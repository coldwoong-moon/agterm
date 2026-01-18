//! Timer and alarm system for AgTerm
//!
//! This module provides a comprehensive timer and alarm system with features including:
//! - Countdown timers with pause/resume functionality
//! - Recurring alarms with weekday scheduling
//! - Multiple action types (notifications, sounds, commands, bell)
//! - Timer state management and persistence

use chrono::{Datelike, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Unique identifier for a timer
pub type TimerId = Uuid;

/// Unique identifier for an alarm
pub type AlarmId = Uuid;

/// State of a timer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerState {
    /// Timer is created but not started
    Idle,
    /// Timer is actively counting down
    Running,
    /// Timer is paused (with remaining duration preserved)
    Paused,
    /// Timer has completed
    Completed,
}

/// Actions to perform when a timer completes or alarm triggers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimerAction {
    /// Show desktop notification
    Notify {
        /// Notification title
        title: String,
        /// Notification body text
        body: String,
    },
    /// Play audio file (or default beep if None)
    Sound {
        /// Optional path to audio file (uses default beep if None)
        file: Option<PathBuf>,
    },
    /// Execute shell command
    Command {
        /// Command to execute
        cmd: String,
    },
    /// Ring terminal bell
    Bell,
    /// Execute multiple actions in sequence
    Multiple(Vec<TimerAction>),
}

impl Default for TimerAction {
    fn default() -> Self {
        Self::Bell
    }
}

/// A countdown timer with pause/resume functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    /// Unique identifier
    pub id: TimerId,
    /// Timer name/description
    pub name: String,
    /// Total duration of the timer
    pub duration: Duration,
    /// Time when the timer was started (None if never started)
    #[serde(skip)]
    started_at: Option<Instant>,
    /// Remaining duration when paused (None if not paused)
    paused_at: Option<Duration>,
    /// Current state of the timer
    pub state: TimerState,
    /// Action to perform on completion
    pub on_complete: TimerAction,
    /// Whether to repeat the timer automatically
    pub repeat: bool,
}

impl Timer {
    /// Create a new timer
    ///
    /// # Arguments
    /// * `name` - Timer name/description
    /// * `duration` - How long the timer should run
    /// * `action` - Action to perform when timer completes
    pub fn new(name: String, duration: Duration, action: TimerAction) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            duration,
            started_at: None,
            paused_at: None,
            state: TimerState::Idle,
            on_complete: action,
            repeat: false,
        }
    }

    /// Create a new repeating timer
    pub fn new_repeating(name: String, duration: Duration, action: TimerAction) -> Self {
        let mut timer = Self::new(name, duration, action);
        timer.repeat = true;
        timer
    }

    /// Start the timer
    pub fn start(&mut self) -> Result<(), String> {
        match self.state {
            TimerState::Idle => {
                self.started_at = Some(Instant::now());
                self.state = TimerState::Running;
                Ok(())
            }
            TimerState::Paused => {
                // Resume from paused state
                self.resume()
            }
            TimerState::Running => Err("Timer is already running".to_string()),
            TimerState::Completed => Err("Timer has already completed".to_string()),
        }
    }

    /// Pause the timer
    pub fn pause(&mut self) -> Result<(), String> {
        if self.state != TimerState::Running {
            return Err("Timer is not running".to_string());
        }

        if let Some(started) = self.started_at {
            let elapsed = started.elapsed();
            let remaining = self.duration.checked_sub(elapsed).unwrap_or(Duration::ZERO);
            self.paused_at = Some(remaining);
            self.state = TimerState::Paused;
            self.started_at = None;
            Ok(())
        } else {
            Err("Timer has no start time".to_string())
        }
    }

    /// Resume the timer from paused state
    pub fn resume(&mut self) -> Result<(), String> {
        if self.state != TimerState::Paused {
            return Err("Timer is not paused".to_string());
        }

        if let Some(remaining) = self.paused_at {
            // Restart with the remaining duration
            self.duration = remaining;
            self.started_at = Some(Instant::now());
            self.paused_at = None;
            self.state = TimerState::Running;
            Ok(())
        } else {
            Err("Timer has no paused duration".to_string())
        }
    }

    /// Stop the timer and reset to idle state
    pub fn stop(&mut self) -> Result<(), String> {
        if self.state == TimerState::Idle {
            return Err("Timer is not started".to_string());
        }

        self.started_at = None;
        self.paused_at = None;
        self.state = TimerState::Idle;
        Ok(())
    }

    /// Get remaining duration (None if not running/paused)
    pub fn remaining(&self) -> Option<Duration> {
        match self.state {
            TimerState::Running => {
                if let Some(started) = self.started_at {
                    let elapsed = started.elapsed();
                    self.duration.checked_sub(elapsed)
                } else {
                    None
                }
            }
            TimerState::Paused => self.paused_at,
            _ => None,
        }
    }

    /// Check if the timer has completed
    pub fn check_completion(&mut self) -> bool {
        if self.state == TimerState::Running {
            if let Some(started) = self.started_at {
                if started.elapsed() >= self.duration {
                    self.state = TimerState::Completed;
                    self.started_at = None;
                    return true;
                }
            }
        }
        false
    }

    /// Reset timer after completion (for repeating timers)
    pub fn reset(&mut self) {
        self.state = TimerState::Idle;
        self.started_at = None;
        self.paused_at = None;
    }
}

/// Event emitted when a timer completes
#[derive(Debug, Clone)]
pub struct TimerEvent {
    /// Timer that completed
    pub timer: Timer,
}

/// Manages multiple timers
pub struct TimerManager {
    timers: HashMap<TimerId, Timer>,
}

impl TimerManager {
    /// Create a new timer manager
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
        }
    }

    /// Create and register a new timer
    ///
    /// # Arguments
    /// * `name` - Timer name/description
    /// * `duration` - How long the timer should run
    /// * `action` - Action to perform when timer completes
    ///
    /// # Returns
    /// The ID of the created timer
    pub fn create_timer(
        &mut self,
        name: String,
        duration: Duration,
        action: TimerAction,
    ) -> TimerId {
        let timer = Timer::new(name, duration, action);
        let id = timer.id;
        self.timers.insert(id, timer);
        id
    }

    /// Create and register a new repeating timer
    pub fn create_repeating_timer(
        &mut self,
        name: String,
        duration: Duration,
        action: TimerAction,
    ) -> TimerId {
        let timer = Timer::new_repeating(name, duration, action);
        let id = timer.id;
        self.timers.insert(id, timer);
        id
    }

    /// Start a timer by ID
    pub fn start_timer(&mut self, id: TimerId) -> Result<(), String> {
        self.timers
            .get_mut(&id)
            .ok_or_else(|| "Timer not found".to_string())
            .and_then(|timer| timer.start())
    }

    /// Pause a timer by ID
    pub fn pause_timer(&mut self, id: TimerId) -> Result<(), String> {
        self.timers
            .get_mut(&id)
            .ok_or_else(|| "Timer not found".to_string())
            .and_then(|timer| timer.pause())
    }

    /// Resume a paused timer
    pub fn resume_timer(&mut self, id: TimerId) -> Result<(), String> {
        self.timers
            .get_mut(&id)
            .ok_or_else(|| "Timer not found".to_string())
            .and_then(|timer| timer.resume())
    }

    /// Stop a timer by ID
    pub fn stop_timer(&mut self, id: TimerId) -> Result<(), String> {
        self.timers
            .get_mut(&id)
            .ok_or_else(|| "Timer not found".to_string())
            .and_then(|timer| timer.stop())
    }

    /// Get remaining duration for a timer
    pub fn get_remaining(&self, id: TimerId) -> Option<Duration> {
        self.timers.get(&id).and_then(|timer| timer.remaining())
    }

    /// Get a reference to a timer by ID
    pub fn get_timer(&self, id: TimerId) -> Option<&Timer> {
        self.timers.get(&id)
    }

    /// List all timers
    pub fn list_timers(&self) -> Vec<&Timer> {
        self.timers.values().collect()
    }

    /// Remove a timer by ID
    pub fn remove_timer(&mut self, id: TimerId) -> Option<Timer> {
        self.timers.remove(&id)
    }

    /// Check all timers and return completed ones
    ///
    /// This should be called regularly (e.g., every frame or on a timer)
    /// to detect when timers complete.
    pub fn tick(&mut self) -> Vec<TimerEvent> {
        let mut events = Vec::new();

        for timer in self.timers.values_mut() {
            if timer.check_completion() {
                events.push(TimerEvent {
                    timer: timer.clone(),
                });

                // Handle repeating timers
                if timer.repeat {
                    timer.reset();
                    let _ = timer.start(); // Auto-restart
                }
            }
        }

        events
    }

    /// Get count of active timers (running or paused)
    pub fn active_count(&self) -> usize {
        self.timers
            .values()
            .filter(|t| t.state == TimerState::Running || t.state == TimerState::Paused)
            .count()
    }
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A scheduled alarm with optional recurring schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    /// Unique identifier
    pub id: AlarmId,
    /// Alarm name/description
    pub name: String,
    /// Time of day for the alarm (24-hour format)
    pub time: NaiveTime,
    /// Days of the week to repeat (empty = one-time alarm)
    pub days: Vec<Weekday>,
    /// Whether the alarm is enabled
    pub enabled: bool,
    /// Action to perform when alarm triggers
    pub action: TimerAction,
    /// Last date this alarm was triggered (to prevent duplicate triggers)
    #[serde(skip)]
    last_triggered: Option<chrono::NaiveDate>,
}

impl Alarm {
    /// Create a new one-time alarm
    pub fn new(name: String, time: NaiveTime, action: TimerAction) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            time,
            days: Vec::new(),
            enabled: true,
            action,
            last_triggered: None,
        }
    }

    /// Create a new recurring alarm
    pub fn new_recurring(
        name: String,
        time: NaiveTime,
        days: Vec<Weekday>,
        action: TimerAction,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            time,
            days,
            enabled: true,
            action,
            last_triggered: None,
        }
    }

    /// Check if this alarm should trigger at the current time
    pub fn should_trigger(&self, now: chrono::DateTime<chrono::Local>) -> bool {
        if !self.enabled {
            return false;
        }

        let current_time = now.time();
        let current_date = now.date_naive();
        let current_weekday = now.weekday();

        // Check if we already triggered today
        if let Some(last) = self.last_triggered {
            if last == current_date {
                return false;
            }
        }

        // Check if time matches (within a 1-minute window)
        let time_diff = if current_time >= self.time {
            current_time - self.time
        } else {
            return false;
        };

        if time_diff > chrono::Duration::minutes(1) {
            return false;
        }

        // Check day of week for recurring alarms
        if self.days.is_empty() {
            // One-time alarm
            true
        } else {
            // Recurring alarm - check if today is a scheduled day
            self.days.contains(&current_weekday)
        }
    }

    /// Mark this alarm as triggered
    pub fn mark_triggered(&mut self, date: chrono::NaiveDate) {
        self.last_triggered = Some(date);
    }

    /// Check if this is a recurring alarm
    pub fn is_recurring(&self) -> bool {
        !self.days.is_empty()
    }
}

/// Event emitted when an alarm triggers
#[derive(Debug, Clone)]
pub struct AlarmEvent {
    /// Alarm that triggered
    pub alarm: Alarm,
}

/// Manages multiple alarms
pub struct AlarmManager {
    alarms: HashMap<AlarmId, Alarm>,
}

impl AlarmManager {
    /// Create a new alarm manager
    pub fn new() -> Self {
        Self {
            alarms: HashMap::new(),
        }
    }

    /// Create and register a new one-time alarm
    ///
    /// # Arguments
    /// * `name` - Alarm name/description
    /// * `time` - Time of day to trigger
    /// * `action` - Action to perform when alarm triggers
    ///
    /// # Returns
    /// The ID of the created alarm
    pub fn create_alarm(&mut self, name: String, time: NaiveTime, action: TimerAction) -> AlarmId {
        let alarm = Alarm::new(name, time, action);
        let id = alarm.id;
        self.alarms.insert(id, alarm);
        id
    }

    /// Create and register a new recurring alarm
    pub fn create_recurring_alarm(
        &mut self,
        name: String,
        time: NaiveTime,
        days: Vec<Weekday>,
        action: TimerAction,
    ) -> AlarmId {
        let alarm = Alarm::new_recurring(name, time, days, action);
        let id = alarm.id;
        self.alarms.insert(id, alarm);
        id
    }

    /// Enable an alarm by ID
    pub fn enable_alarm(&mut self, id: AlarmId) -> Result<(), String> {
        self.alarms
            .get_mut(&id)
            .ok_or_else(|| "Alarm not found".to_string())
            .map(|alarm| {
                alarm.enabled = true;
            })
    }

    /// Disable an alarm by ID
    pub fn disable_alarm(&mut self, id: AlarmId) -> Result<(), String> {
        self.alarms
            .get_mut(&id)
            .ok_or_else(|| "Alarm not found".to_string())
            .map(|alarm| {
                alarm.enabled = false;
            })
    }

    /// Toggle an alarm's enabled state
    pub fn toggle_alarm(&mut self, id: AlarmId) -> Result<bool, String> {
        self.alarms
            .get_mut(&id)
            .ok_or_else(|| "Alarm not found".to_string())
            .map(|alarm| {
                alarm.enabled = !alarm.enabled;
                alarm.enabled
            })
    }

    /// Get a reference to an alarm by ID
    pub fn get_alarm(&self, id: AlarmId) -> Option<&Alarm> {
        self.alarms.get(&id)
    }

    /// List all alarms
    pub fn list_alarms(&self) -> Vec<&Alarm> {
        self.alarms.values().collect()
    }

    /// Remove an alarm by ID
    pub fn remove_alarm(&mut self, id: AlarmId) -> Option<Alarm> {
        self.alarms.remove(&id)
    }

    /// Check all alarms and return those that should trigger
    ///
    /// This should be called regularly (e.g., every minute) to detect
    /// when alarms should trigger.
    pub fn check_alarms(&mut self) -> Vec<AlarmEvent> {
        let now = chrono::Local::now();
        let today = now.date_naive();
        let mut events = Vec::new();

        for alarm in self.alarms.values_mut() {
            if alarm.should_trigger(now) {
                events.push(AlarmEvent {
                    alarm: alarm.clone(),
                });
                alarm.mark_triggered(today);

                // Disable one-time alarms after triggering
                if !alarm.is_recurring() {
                    alarm.enabled = false;
                }
            }
        }

        events
    }

    /// Get count of enabled alarms
    pub fn enabled_count(&self) -> usize {
        self.alarms.values().filter(|a| a.enabled).count()
    }
}

impl Default for AlarmManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::new(
            "Test Timer".to_string(),
            Duration::from_secs(10),
            TimerAction::Bell,
        );
        assert_eq!(timer.name, "Test Timer");
        assert_eq!(timer.duration, Duration::from_secs(10));
        assert_eq!(timer.state, TimerState::Idle);
        assert!(!timer.repeat);
    }

    #[test]
    fn test_timer_start() {
        let mut timer = Timer::new(
            "Test".to_string(),
            Duration::from_secs(10),
            TimerAction::Bell,
        );
        assert!(timer.start().is_ok());
        assert_eq!(timer.state, TimerState::Running);
        assert!(timer.started_at.is_some());
    }

    #[test]
    fn test_timer_pause_resume() {
        let mut timer = Timer::new(
            "Test".to_string(),
            Duration::from_secs(10),
            TimerAction::Bell,
        );

        // Start timer
        timer.start().unwrap();
        assert_eq!(timer.state, TimerState::Running);

        // Wait a bit
        std::thread::sleep(Duration::from_millis(100));

        // Pause timer
        timer.pause().unwrap();
        assert_eq!(timer.state, TimerState::Paused);
        assert!(timer.paused_at.is_some());
        let remaining = timer.paused_at.unwrap();
        assert!(remaining < Duration::from_secs(10));

        // Resume timer
        timer.resume().unwrap();
        assert_eq!(timer.state, TimerState::Running);
        assert!(timer.started_at.is_some());
    }

    #[test]
    fn test_timer_stop() {
        let mut timer = Timer::new(
            "Test".to_string(),
            Duration::from_secs(10),
            TimerAction::Bell,
        );

        timer.start().unwrap();
        assert_eq!(timer.state, TimerState::Running);

        timer.stop().unwrap();
        assert_eq!(timer.state, TimerState::Idle);
        assert!(timer.started_at.is_none());
    }

    #[test]
    fn test_timer_remaining() {
        let mut timer = Timer::new(
            "Test".to_string(),
            Duration::from_secs(5),
            TimerAction::Bell,
        );

        // Not started - no remaining time
        assert!(timer.remaining().is_none());

        // Start timer
        timer.start().unwrap();
        let remaining = timer.remaining().unwrap();
        assert!(remaining <= Duration::from_secs(5));

        // Pause and check remaining
        std::thread::sleep(Duration::from_millis(100));
        timer.pause().unwrap();
        let remaining = timer.remaining().unwrap();
        assert!(remaining < Duration::from_secs(5));
    }

    #[test]
    fn test_timer_completion() {
        let mut timer = Timer::new(
            "Test".to_string(),
            Duration::from_millis(50),
            TimerAction::Bell,
        );

        timer.start().unwrap();
        assert_eq!(timer.state, TimerState::Running);

        // Should not be complete immediately
        assert!(!timer.check_completion());

        // Wait for completion
        std::thread::sleep(Duration::from_millis(100));
        assert!(timer.check_completion());
        assert_eq!(timer.state, TimerState::Completed);
    }

    #[test]
    fn test_timer_manager_create() {
        let mut manager = TimerManager::new();
        let id = manager.create_timer(
            "Test".to_string(),
            Duration::from_secs(10),
            TimerAction::Bell,
        );

        let timer = manager.get_timer(id).unwrap();
        assert_eq!(timer.name, "Test");
        assert_eq!(timer.duration, Duration::from_secs(10));
    }

    #[test]
    fn test_timer_manager_operations() {
        let mut manager = TimerManager::new();
        let id = manager.create_timer(
            "Test".to_string(),
            Duration::from_secs(10),
            TimerAction::Bell,
        );

        // Start timer
        assert!(manager.start_timer(id).is_ok());
        let timer = manager.get_timer(id).unwrap();
        assert_eq!(timer.state, TimerState::Running);

        // Pause timer
        assert!(manager.pause_timer(id).is_ok());
        let timer = manager.get_timer(id).unwrap();
        assert_eq!(timer.state, TimerState::Paused);

        // Resume timer
        assert!(manager.resume_timer(id).is_ok());
        let timer = manager.get_timer(id).unwrap();
        assert_eq!(timer.state, TimerState::Running);

        // Stop timer
        assert!(manager.stop_timer(id).is_ok());
        let timer = manager.get_timer(id).unwrap();
        assert_eq!(timer.state, TimerState::Idle);
    }

    #[test]
    fn test_timer_manager_list() {
        let mut manager = TimerManager::new();
        manager.create_timer("Timer 1".to_string(), Duration::from_secs(10), TimerAction::Bell);
        manager.create_timer("Timer 2".to_string(), Duration::from_secs(20), TimerAction::Bell);

        let timers = manager.list_timers();
        assert_eq!(timers.len(), 2);
    }

    #[test]
    fn test_timer_manager_tick() {
        let mut manager = TimerManager::new();
        let id = manager.create_timer(
            "Test".to_string(),
            Duration::from_millis(50),
            TimerAction::Bell,
        );
        manager.start_timer(id).unwrap();

        // Should not complete immediately
        let events = manager.tick();
        assert_eq!(events.len(), 0);

        // Wait for completion
        std::thread::sleep(Duration::from_millis(100));
        let events = manager.tick();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].timer.name, "Test");
    }

    #[test]
    fn test_repeating_timer() {
        let mut manager = TimerManager::new();
        let id = manager.create_repeating_timer(
            "Repeat".to_string(),
            Duration::from_millis(50),
            TimerAction::Bell,
        );
        manager.start_timer(id).unwrap();

        // Wait for first completion
        std::thread::sleep(Duration::from_millis(100));
        let events = manager.tick();
        assert_eq!(events.len(), 1);

        // Timer should auto-restart
        let timer = manager.get_timer(id).unwrap();
        assert_eq!(timer.state, TimerState::Running);
    }

    #[test]
    fn test_timer_action_types() {
        let notify = TimerAction::Notify {
            title: "Test".to_string(),
            body: "Body".to_string(),
        };
        assert!(matches!(notify, TimerAction::Notify { .. }));

        let sound = TimerAction::Sound {
            file: Some(PathBuf::from("/test.wav")),
        };
        assert!(matches!(sound, TimerAction::Sound { .. }));

        let command = TimerAction::Command {
            cmd: "echo test".to_string(),
        };
        assert!(matches!(command, TimerAction::Command { .. }));

        let bell = TimerAction::Bell;
        assert!(matches!(bell, TimerAction::Bell));

        let multiple = TimerAction::Multiple(vec![TimerAction::Bell, notify]);
        assert!(matches!(multiple, TimerAction::Multiple(_)));
    }

    #[test]
    fn test_alarm_creation() {
        let time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();
        let alarm = Alarm::new("Morning".to_string(), time, TimerAction::Bell);

        assert_eq!(alarm.name, "Morning");
        assert_eq!(alarm.time, time);
        assert!(alarm.days.is_empty());
        assert!(alarm.enabled);
        assert!(!alarm.is_recurring());
    }

    #[test]
    fn test_recurring_alarm() {
        let time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let days = vec![Weekday::Mon, Weekday::Wed, Weekday::Fri];
        let alarm = Alarm::new_recurring("Work".to_string(), time, days.clone(), TimerAction::Bell);

        assert_eq!(alarm.days, days);
        assert!(alarm.is_recurring());
    }

    #[test]
    fn test_alarm_manager_create() {
        let mut manager = AlarmManager::new();
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let id = manager.create_alarm("Test".to_string(), time, TimerAction::Bell);

        let alarm = manager.get_alarm(id).unwrap();
        assert_eq!(alarm.name, "Test");
        assert_eq!(alarm.time, time);
    }

    #[test]
    fn test_alarm_enable_disable() {
        let mut manager = AlarmManager::new();
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let id = manager.create_alarm("Test".to_string(), time, TimerAction::Bell);

        // Disable alarm
        assert!(manager.disable_alarm(id).is_ok());
        let alarm = manager.get_alarm(id).unwrap();
        assert!(!alarm.enabled);

        // Enable alarm
        assert!(manager.enable_alarm(id).is_ok());
        let alarm = manager.get_alarm(id).unwrap();
        assert!(alarm.enabled);
    }

    #[test]
    fn test_alarm_toggle() {
        let mut manager = AlarmManager::new();
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let id = manager.create_alarm("Test".to_string(), time, TimerAction::Bell);

        let alarm = manager.get_alarm(id).unwrap();
        let initial_state = alarm.enabled;

        let new_state = manager.toggle_alarm(id).unwrap();
        assert_eq!(new_state, !initial_state);
    }

    #[test]
    fn test_alarm_manager_list() {
        let mut manager = AlarmManager::new();
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        manager.create_alarm("Alarm 1".to_string(), time, TimerAction::Bell);
        manager.create_alarm("Alarm 2".to_string(), time, TimerAction::Bell);

        let alarms = manager.list_alarms();
        assert_eq!(alarms.len(), 2);
    }

    #[test]
    fn test_alarm_should_trigger_disabled() {
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let mut alarm = Alarm::new("Test".to_string(), time, TimerAction::Bell);
        alarm.enabled = false;

        let now = chrono::Local::now();
        assert!(!alarm.should_trigger(now));
    }

    #[test]
    fn test_alarm_manager_counts() {
        let mut manager = AlarmManager::new();
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let id1 = manager.create_alarm("Alarm 1".to_string(), time, TimerAction::Bell);
        let id2 = manager.create_alarm("Alarm 2".to_string(), time, TimerAction::Bell);

        assert_eq!(manager.enabled_count(), 2);

        manager.disable_alarm(id1).unwrap();
        assert_eq!(manager.enabled_count(), 1);

        manager.disable_alarm(id2).unwrap();
        assert_eq!(manager.enabled_count(), 0);
    }

    #[test]
    fn test_timer_manager_active_count() {
        let mut manager = TimerManager::new();
        let id1 = manager.create_timer(
            "Timer 1".to_string(),
            Duration::from_secs(10),
            TimerAction::Bell,
        );
        let id2 = manager.create_timer(
            "Timer 2".to_string(),
            Duration::from_secs(20),
            TimerAction::Bell,
        );

        assert_eq!(manager.active_count(), 0);

        manager.start_timer(id1).unwrap();
        assert_eq!(manager.active_count(), 1);

        manager.start_timer(id2).unwrap();
        assert_eq!(manager.active_count(), 2);

        manager.pause_timer(id1).unwrap();
        assert_eq!(manager.active_count(), 2); // Paused timers are still active

        manager.stop_timer(id2).unwrap();
        assert_eq!(manager.active_count(), 1);
    }

    #[test]
    fn test_timer_manager_remove() {
        let mut manager = TimerManager::new();
        let id = manager.create_timer(
            "Test".to_string(),
            Duration::from_secs(10),
            TimerAction::Bell,
        );

        assert!(manager.get_timer(id).is_some());

        let removed = manager.remove_timer(id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "Test");
        assert!(manager.get_timer(id).is_none());
    }

    #[test]
    fn test_alarm_manager_remove() {
        let mut manager = AlarmManager::new();
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let id = manager.create_alarm("Test".to_string(), time, TimerAction::Bell);

        assert!(manager.get_alarm(id).is_some());

        let removed = manager.remove_alarm(id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "Test");
        assert!(manager.get_alarm(id).is_none());
    }

    #[test]
    fn test_timer_error_conditions() {
        let mut timer = Timer::new(
            "Test".to_string(),
            Duration::from_secs(10),
            TimerAction::Bell,
        );

        // Cannot pause when not running
        assert!(timer.pause().is_err());

        // Cannot resume when not paused
        assert!(timer.resume().is_err());

        // Cannot stop when idle
        assert!(timer.stop().is_err());

        // Start timer
        timer.start().unwrap();

        // Cannot start again when running
        assert!(timer.start().is_err());
    }

    #[test]
    fn test_manager_error_conditions() {
        let mut manager = TimerManager::new();
        let invalid_id = Uuid::new_v4();

        // Operations on non-existent timer should fail
        assert!(manager.start_timer(invalid_id).is_err());
        assert!(manager.pause_timer(invalid_id).is_err());
        assert!(manager.resume_timer(invalid_id).is_err());
        assert!(manager.stop_timer(invalid_id).is_err());
        assert!(manager.get_remaining(invalid_id).is_none());
        assert!(manager.get_timer(invalid_id).is_none());
    }
}
