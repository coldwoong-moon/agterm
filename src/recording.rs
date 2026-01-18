//! Terminal session recording and playback
//!
//! This module provides terminal session recording, playback, and export functionality
//! compatible with the asciicast v2 format.
//!
//! # Features
//!
//! - Record terminal input/output with timestamps
//! - Record terminal resize events
//! - Playback with speed control (0.5x - 4x)
//! - Seek to specific timestamps
//! - Compression support (gzip)
//! - asciicast v2 format compatibility
//!
//! # Examples
//!
//! ```no_run
//! use agterm::recording::{Recording, RecordingEvent, RecordingPlayer, PlayerState};
//! use std::time::Duration;
//!
//! // Start recording
//! let mut recording = Recording::new(80, 24);
//! recording.add_output(Duration::from_secs(0), b"Hello, world!\n");
//! recording.add_input(Duration::from_millis(100), b"ls\n");
//!
//! // Save recording
//! recording.save_to_file("session.cast").unwrap();
//!
//! // Load and play back
//! let recording = Recording::load_from_file("session.cast").unwrap();
//! let mut player = RecordingPlayer::new(recording);
//! player.play();
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Errors that can occur during recording operations
#[derive(Debug, Error)]
pub enum RecordingError {
    /// I/O error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Invalid format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    /// Compression error
    #[error("Compression error: {0}")]
    Compression(String),
}

/// Recording event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RecordingEvent {
    /// Output data from terminal
    #[serde(rename = "o")]
    Output {
        /// Time offset from recording start
        #[serde(rename = "time")]
        timestamp: f64,
        /// Output data (UTF-8 or bytes)
        data: String,
    },
    /// Input data to terminal
    #[serde(rename = "i")]
    Input {
        /// Time offset from recording start
        #[serde(rename = "time")]
        timestamp: f64,
        /// Input data (UTF-8 or bytes)
        data: String,
    },
    /// Terminal resize event
    #[serde(rename = "r")]
    Resize {
        /// Time offset from recording start
        #[serde(rename = "time")]
        timestamp: f64,
        /// Number of columns
        cols: u16,
        /// Number of rows
        rows: u16,
    },
}

impl RecordingEvent {
    /// Get the timestamp of this event
    pub fn timestamp(&self) -> Duration {
        let secs = match self {
            RecordingEvent::Output { timestamp, .. } => *timestamp,
            RecordingEvent::Input { timestamp, .. } => *timestamp,
            RecordingEvent::Resize { timestamp, .. } => *timestamp,
        };
        Duration::from_secs_f64(secs)
    }

    /// Check if this is an output event
    pub fn is_output(&self) -> bool {
        matches!(self, RecordingEvent::Output { .. })
    }

    /// Check if this is an input event
    pub fn is_input(&self) -> bool {
        matches!(self, RecordingEvent::Input { .. })
    }

    /// Check if this is a resize event
    pub fn is_resize(&self) -> bool {
        matches!(self, RecordingEvent::Resize { .. })
    }

    /// Get the data if this is an output or input event
    pub fn data(&self) -> Option<&str> {
        match self {
            RecordingEvent::Output { data, .. } | RecordingEvent::Input { data, .. } => {
                Some(data)
            }
            RecordingEvent::Resize { .. } => None,
        }
    }
}

/// Recording metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    /// asciicast format version (always 2)
    pub version: u32,
    /// Terminal width in columns
    pub width: u16,
    /// Terminal height in rows
    pub height: u16,
    /// Recording start time (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    /// Total duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    /// Recording title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Shell command that was executed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Idle time limit (seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_time_limit: Option<f64>,
    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Theme
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<RecordingTheme>,
}

/// Recording theme (for asciicast compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingTheme {
    /// Foreground color
    pub fg: String,
    /// Background color
    pub bg: String,
    /// Color palette (16 colors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palette: Option<Vec<String>>,
}

/// Terminal session recording
#[derive(Debug, Clone)]
pub struct Recording {
    /// Recording metadata
    pub metadata: RecordingMetadata,
    /// List of recording events
    events: Vec<RecordingEvent>,
    /// Recording start time (for relative timestamps)
    start_time: Option<Instant>,
    /// Whether recording is currently active
    is_recording: bool,
}

impl Recording {
    /// Create a new recording with the specified terminal size
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            metadata: RecordingMetadata {
                version: 2,
                width: cols,
                height: rows,
                timestamp: Some(Utc::now()),
                duration: None,
                title: None,
                command: None,
                idle_time_limit: None,
                env: None,
                theme: None,
            },
            events: Vec::new(),
            start_time: None,
            is_recording: false,
        }
    }

    /// Start recording
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.is_recording = true;
        self.metadata.timestamp = Some(Utc::now());
        info!("Recording started");
    }

    /// Stop recording
    pub fn stop(&mut self) {
        self.is_recording = false;
        // Use the last event timestamp as duration, or elapsed time if no events
        let duration = self
            .events
            .last()
            .map(|e| match e {
                RecordingEvent::Output { timestamp, .. } => *timestamp,
                RecordingEvent::Input { timestamp, .. } => *timestamp,
                RecordingEvent::Resize { timestamp, .. } => *timestamp,
            })
            .or_else(|| self.start_time.map(|s| s.elapsed().as_secs_f64()))
            .unwrap_or(0.0);
        self.metadata.duration = Some(duration);
        info!(duration = %duration, "Recording stopped");
    }

    /// Check if recording is active
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Get elapsed time since recording started
    fn elapsed(&self) -> Duration {
        self.start_time
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Add an output event
    pub fn add_output(&mut self, timestamp: Duration, data: &[u8]) {
        if !self.is_recording {
            return;
        }

        let data_str = String::from_utf8_lossy(data).to_string();
        let event = RecordingEvent::Output {
            timestamp: timestamp.as_secs_f64(),
            data: data_str,
        };

        self.events.push(event);
        debug!(timestamp = %timestamp.as_secs_f64(), size = data.len(), "Output recorded");
    }

    /// Add an input event
    pub fn add_input(&mut self, timestamp: Duration, data: &[u8]) {
        if !self.is_recording {
            return;
        }

        let data_str = String::from_utf8_lossy(data).to_string();
        let event = RecordingEvent::Input {
            timestamp: timestamp.as_secs_f64(),
            data: data_str,
        };

        self.events.push(event);
        debug!(timestamp = %timestamp.as_secs_f64(), size = data.len(), "Input recorded");
    }

    /// Add a resize event
    pub fn add_resize(&mut self, timestamp: Duration, cols: u16, rows: u16) {
        if !self.is_recording {
            return;
        }

        let event = RecordingEvent::Resize {
            timestamp: timestamp.as_secs_f64(),
            cols,
            rows,
        };

        self.events.push(event);
        debug!(timestamp = %timestamp.as_secs_f64(), cols, rows, "Resize recorded");
    }

    /// Record output with automatic timestamp
    pub fn record_output(&mut self, data: &[u8]) {
        let timestamp = self.elapsed();
        self.add_output(timestamp, data);
    }

    /// Record input with automatic timestamp
    pub fn record_input(&mut self, data: &[u8]) {
        let timestamp = self.elapsed();
        self.add_input(timestamp, data);
    }

    /// Record resize with automatic timestamp
    pub fn record_resize(&mut self, cols: u16, rows: u16) {
        let timestamp = self.elapsed();
        self.add_resize(timestamp, cols, rows);
    }

    /// Get all events
    pub fn events(&self) -> &[RecordingEvent] {
        &self.events
    }

    /// Get the number of events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if recording is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get recording duration
    pub fn duration(&self) -> Duration {
        self.metadata
            .duration
            .map(Duration::from_secs_f64)
            .or_else(|| {
                self.events
                    .last()
                    .map(|e| Duration::from_secs_f64(match e {
                        RecordingEvent::Output { timestamp, .. } => *timestamp,
                        RecordingEvent::Input { timestamp, .. } => *timestamp,
                        RecordingEvent::Resize { timestamp, .. } => *timestamp,
                    }))
            })
            .unwrap_or(Duration::ZERO)
    }

    /// Save recording to a file in asciicast v2 format
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), RecordingError> {
        let file = File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);

        // Write header (metadata) as first line
        let header_json = serde_json::to_string(&self.metadata)?;
        writeln!(writer, "{header_json}")?;

        // Write events as subsequent lines
        for event in &self.events {
            let event_json = match event {
                RecordingEvent::Output { timestamp, data } => {
                    format!("[{},\"o\",{}]", timestamp, serde_json::to_string(data)?)
                }
                RecordingEvent::Input { timestamp, data } => {
                    format!("[{},\"i\",{}]", timestamp, serde_json::to_string(data)?)
                }
                RecordingEvent::Resize {
                    timestamp,
                    cols,
                    rows,
                } => {
                    let size_str = format!("{cols}x{rows}");
                    format!("[{},\"r\",{}]", timestamp, serde_json::to_string(&size_str)?)
                }
            };
            writeln!(writer, "{event_json}")?;
        }

        writer.flush()?;
        info!(path = %path.as_ref().display(), events = self.events.len(), "Recording saved");
        Ok(())
    }

    /// Load recording from a file in asciicast v2 format
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, RecordingError> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Read header (metadata) from first line
        let header_line = lines
            .next()
            .ok_or_else(|| RecordingError::InvalidFormat("Empty file".to_string()))??;
        let metadata: RecordingMetadata = serde_json::from_str(&header_line)?;

        // Read events from subsequent lines
        let mut events = Vec::new();
        for (line_num, line_result) in lines.enumerate() {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }

            // Parse event line: [timestamp, type, data]
            let values: Vec<serde_json::Value> = serde_json::from_str(&line).map_err(|e| {
                RecordingError::InvalidFormat(format!("Line {}: {}", line_num + 2, e))
            })?;

            if values.len() < 3 {
                warn!(line = line_num + 2, "Skipping invalid event line");
                continue;
            }

            let timestamp = values[0]
                .as_f64()
                .ok_or_else(|| RecordingError::InvalidFormat("Invalid timestamp".to_string()))?;

            let event_type = values[1].as_str().ok_or_else(|| {
                RecordingError::InvalidFormat("Invalid event type".to_string())
            })?;

            let event = match event_type {
                "o" => {
                    let data = values[2]
                        .as_str()
                        .ok_or_else(|| RecordingError::InvalidFormat("Invalid data".to_string()))?
                        .to_string();
                    RecordingEvent::Output { timestamp, data }
                }
                "i" => {
                    let data = values[2]
                        .as_str()
                        .ok_or_else(|| RecordingError::InvalidFormat("Invalid data".to_string()))?
                        .to_string();
                    RecordingEvent::Input { timestamp, data }
                }
                "r" => {
                    let size_str = values[2].as_str().ok_or_else(|| {
                        RecordingError::InvalidFormat("Invalid size".to_string())
                    })?;
                    let parts: Vec<&str> = size_str.split('x').collect();
                    if parts.len() != 2 {
                        return Err(RecordingError::InvalidFormat(
                            "Invalid resize format".to_string(),
                        ));
                    }
                    let cols = parts[0].parse().map_err(|_| {
                        RecordingError::InvalidFormat("Invalid columns".to_string())
                    })?;
                    let rows = parts[1]
                        .parse()
                        .map_err(|_| RecordingError::InvalidFormat("Invalid rows".to_string()))?;
                    RecordingEvent::Resize {
                        timestamp,
                        cols,
                        rows,
                    }
                }
                _ => {
                    warn!(line = line_num + 2, event_type, "Unknown event type");
                    continue;
                }
            };

            events.push(event);
        }

        info!(path = %path.as_ref().display(), events = events.len(), "Recording loaded");

        Ok(Self {
            metadata,
            events,
            start_time: None,
            is_recording: false,
        })
    }

    /// Compress recording events (removes redundant data)
    pub fn compress(&mut self) {
        // Remove consecutive duplicate resize events
        let mut compressed = Vec::new();
        let mut last_resize: Option<(u16, u16)> = None;

        for event in &self.events {
            match event {
                RecordingEvent::Resize { cols, rows, .. } => {
                    if last_resize != Some((*cols, *rows)) {
                        compressed.push(event.clone());
                        last_resize = Some((*cols, *rows));
                    }
                }
                _ => {
                    compressed.push(event.clone());
                }
            }
        }

        let before = self.events.len();
        self.events = compressed;
        let after = self.events.len();

        if before != after {
            info!(before, after, saved = before - after, "Recording compressed");
        }
    }

    /// Get statistics about the recording
    pub fn stats(&self) -> RecordingStats {
        let mut stats = RecordingStats::default();

        for event in &self.events {
            match event {
                RecordingEvent::Output { data, .. } => {
                    stats.output_events += 1;
                    stats.output_bytes += data.len();
                }
                RecordingEvent::Input { data, .. } => {
                    stats.input_events += 1;
                    stats.input_bytes += data.len();
                }
                RecordingEvent::Resize { .. } => {
                    stats.resize_events += 1;
                }
            }
        }

        stats.duration = self.duration();
        stats
    }
}

/// Recording statistics
#[derive(Debug, Clone, Default)]
pub struct RecordingStats {
    /// Total duration
    pub duration: Duration,
    /// Number of output events
    pub output_events: usize,
    /// Total output bytes
    pub output_bytes: usize,
    /// Number of input events
    pub input_events: usize,
    /// Total input bytes
    pub input_bytes: usize,
    /// Number of resize events
    pub resize_events: usize,
}

/// Player state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    /// Player is stopped
    Stopped,
    /// Player is playing
    Playing,
    /// Player is paused
    Paused,
}

/// Recording player with playback controls
#[derive(Debug, Clone)]
pub struct RecordingPlayer {
    /// The recording being played
    recording: Recording,
    /// Current playback position (event index)
    position: usize,
    /// Current playback time
    current_time: Duration,
    /// Playback speed multiplier
    speed: f64,
    /// Player state
    state: PlayerState,
    /// Event queue for events that should be emitted
    event_queue: VecDeque<RecordingEvent>,
    /// Last update time
    last_update: Option<Instant>,
}

impl RecordingPlayer {
    /// Create a new player for the given recording
    pub fn new(recording: Recording) -> Self {
        Self {
            recording,
            position: 0,
            current_time: Duration::ZERO,
            speed: 1.0,
            state: PlayerState::Stopped,
            event_queue: VecDeque::new(),
            last_update: None,
        }
    }

    /// Start or resume playback
    pub fn play(&mut self) {
        if self.state == PlayerState::Stopped {
            self.position = 0;
            self.current_time = Duration::ZERO;
        }
        self.state = PlayerState::Playing;
        self.last_update = Some(Instant::now());
        debug!(speed = %self.speed, "Playback started");
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if self.state == PlayerState::Playing {
            self.state = PlayerState::Paused;
            self.last_update = None;
            debug!(position = self.position, "Playback paused");
        }
    }

    /// Stop playback and reset to beginning
    pub fn stop(&mut self) {
        self.state = PlayerState::Stopped;
        self.position = 0;
        self.current_time = Duration::ZERO;
        self.event_queue.clear();
        self.last_update = None;
        debug!("Playback stopped");
    }

    /// Get current player state
    pub fn state(&self) -> PlayerState {
        self.state
    }

    /// Set playback speed (0.5x to 4x)
    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.clamp(0.5, 4.0);
        debug!(speed = %self.speed, "Playback speed changed");
    }

    /// Get current playback speed
    pub fn speed(&self) -> f64 {
        self.speed
    }

    /// Seek to a specific time
    pub fn seek(&mut self, time: Duration) {
        let target_secs = time.as_secs_f64();

        // Find the position for this time
        self.position = self
            .recording
            .events
            .iter()
            .position(|e| {
                let ts = match e {
                    RecordingEvent::Output { timestamp, .. } => *timestamp,
                    RecordingEvent::Input { timestamp, .. } => *timestamp,
                    RecordingEvent::Resize { timestamp, .. } => *timestamp,
                };
                ts >= target_secs
            })
            .unwrap_or(self.recording.events.len());

        self.current_time = time;
        self.event_queue.clear();
        debug!(position = self.position, time = %time.as_secs_f64(), "Seeked to time");
    }

    /// Get current playback time
    pub fn current_time(&self) -> Duration {
        self.current_time
    }

    /// Get total duration
    pub fn duration(&self) -> Duration {
        self.recording.duration()
    }

    /// Get current position as percentage (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        let duration = self.duration().as_secs_f64();
        if duration == 0.0 {
            0.0
        } else {
            (self.current_time.as_secs_f64() / duration).min(1.0)
        }
    }

    /// Check if playback has finished
    pub fn is_finished(&self) -> bool {
        self.position >= self.recording.events.len()
    }

    /// Update playback state and return events that should be emitted now
    pub fn update(&mut self) -> Vec<RecordingEvent> {
        if self.state != PlayerState::Playing {
            return Vec::new();
        }

        // Calculate time delta
        let now = Instant::now();
        let delta = if let Some(last) = self.last_update {
            now.duration_since(last)
        } else {
            Duration::ZERO
        };
        self.last_update = Some(now);

        // Advance current time by delta * speed
        let scaled_delta = delta.mul_f64(self.speed);
        self.current_time += scaled_delta;

        // Collect events that should be emitted now
        let mut events = Vec::new();
        let current_secs = self.current_time.as_secs_f64();

        while self.position < self.recording.events.len() {
            let event = &self.recording.events[self.position];
            let event_time = match event {
                RecordingEvent::Output { timestamp, .. } => *timestamp,
                RecordingEvent::Input { timestamp, .. } => *timestamp,
                RecordingEvent::Resize { timestamp, .. } => *timestamp,
            };

            if event_time <= current_secs {
                events.push(event.clone());
                self.position += 1;
            } else {
                break;
            }
        }

        // Check if finished
        if self.is_finished() {
            debug!("Playback finished");
            self.stop();
        }

        events
    }

    /// Skip forward by the specified duration
    pub fn skip_forward(&mut self, duration: Duration) {
        let new_time = self.current_time + duration;
        self.seek(new_time.min(self.duration()));
    }

    /// Skip backward by the specified duration
    pub fn skip_backward(&mut self, duration: Duration) {
        let new_time = self.current_time.saturating_sub(duration);
        self.seek(new_time);
    }

    /// Get the underlying recording
    pub fn recording(&self) -> &Recording {
        &self.recording
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_recording_new() {
        let recording = Recording::new(80, 24);
        assert_eq!(recording.metadata.width, 80);
        assert_eq!(recording.metadata.height, 24);
        assert_eq!(recording.metadata.version, 2);
        assert!(recording.is_empty());
    }

    #[test]
    fn test_recording_events() {
        let mut recording = Recording::new(80, 24);
        recording.start();

        recording.add_output(Duration::from_secs(0), b"Hello");
        recording.add_input(Duration::from_millis(100), b"ls\n");
        recording.add_resize(Duration::from_millis(200), 120, 40);

        assert_eq!(recording.len(), 3);
        assert!(recording.events()[0].is_output());
        assert!(recording.events()[1].is_input());
        assert!(recording.events()[2].is_resize());

        recording.stop();
        assert!(!recording.is_recording());
    }

    #[test]
    fn test_recording_auto_timestamp() {
        let mut recording = Recording::new(80, 24);
        recording.start();

        thread::sleep(Duration::from_millis(50));
        recording.record_output(b"test");

        assert_eq!(recording.len(), 1);
        assert!(recording.events()[0].timestamp() >= Duration::from_millis(50));

        recording.stop();
    }

    #[test]
    fn test_recording_save_load() {
        let mut recording = Recording::new(80, 24);
        recording.start();
        recording.add_output(Duration::from_secs(0), b"Hello, world!");
        recording.add_input(Duration::from_millis(100), b"test\n");
        recording.add_resize(Duration::from_millis(200), 120, 40);
        recording.stop();

        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.cast");

        recording.save_to_file(&path).unwrap();
        let loaded = Recording::load_from_file(&path).unwrap();

        assert_eq!(loaded.metadata.width, 80);
        assert_eq!(loaded.metadata.height, 24);
        assert_eq!(loaded.len(), 3);
        assert!(loaded.events()[0].is_output());
        assert_eq!(loaded.events()[0].data(), Some("Hello, world!"));
    }

    #[test]
    fn test_recording_compression() {
        let mut recording = Recording::new(80, 24);
        recording.start();
        recording.add_resize(Duration::from_secs(0), 80, 24);
        recording.add_resize(Duration::from_millis(100), 80, 24); // Duplicate
        recording.add_resize(Duration::from_millis(200), 120, 40); // Different
        recording.add_resize(Duration::from_millis(300), 120, 40); // Duplicate
        recording.stop();

        assert_eq!(recording.len(), 4);
        recording.compress();
        assert_eq!(recording.len(), 2); // Only unique resize events remain
    }

    #[test]
    fn test_recording_stats() {
        let mut recording = Recording::new(80, 24);
        recording.start();
        recording.add_output(Duration::from_secs(0), b"Hello");
        recording.add_output(Duration::from_millis(100), b"World");
        recording.add_input(Duration::from_millis(200), b"test\n");
        recording.add_resize(Duration::from_millis(300), 120, 40);
        recording.stop();

        let stats = recording.stats();
        assert_eq!(stats.output_events, 2);
        assert_eq!(stats.output_bytes, 10); // "Hello" + "World"
        assert_eq!(stats.input_events, 1);
        assert_eq!(stats.input_bytes, 5); // "test\n"
        assert_eq!(stats.resize_events, 1);
    }

    #[test]
    fn test_player_basic() {
        let mut recording = Recording::new(80, 24);
        recording.start();
        recording.add_output(Duration::from_secs(0), b"Event 1");
        recording.add_output(Duration::from_millis(100), b"Event 2");
        recording.stop();

        let mut player = RecordingPlayer::new(recording);
        assert_eq!(player.state(), PlayerState::Stopped);

        player.play();
        assert_eq!(player.state(), PlayerState::Playing);

        player.pause();
        assert_eq!(player.state(), PlayerState::Paused);

        player.stop();
        assert_eq!(player.state(), PlayerState::Stopped);
    }

    #[test]
    fn test_player_speed() {
        let mut recording = Recording::new(80, 24);
        recording.start();
        recording.add_output(Duration::from_secs(0), b"test");
        recording.stop();

        let mut player = RecordingPlayer::new(recording);

        player.set_speed(2.0);
        assert_eq!(player.speed(), 2.0);

        player.set_speed(0.25); // Below minimum
        assert_eq!(player.speed(), 0.5); // Clamped to minimum

        player.set_speed(10.0); // Above maximum
        assert_eq!(player.speed(), 4.0); // Clamped to maximum
    }

    #[test]
    fn test_player_seek() {
        let mut recording = Recording::new(80, 24);
        recording.start();
        recording.add_output(Duration::from_secs(0), b"Event 1");
        recording.add_output(Duration::from_secs(1), b"Event 2");
        recording.add_output(Duration::from_secs(2), b"Event 3");
        recording.stop();

        let mut player = RecordingPlayer::new(recording);

        player.seek(Duration::from_millis(1500));
        assert!(player.current_time() >= Duration::from_secs(1));

        player.seek(Duration::ZERO);
        assert_eq!(player.current_time(), Duration::ZERO);
    }

    #[test]
    fn test_player_progress() {
        let mut recording = Recording::new(80, 24);
        recording.start();
        recording.add_output(Duration::from_secs(0), b"Start");
        recording.add_output(Duration::from_secs(5), b"Middle");
        recording.add_output(Duration::from_secs(10), b"End");
        recording.stop();

        let mut player = RecordingPlayer::new(recording);

        assert_eq!(player.progress(), 0.0);

        player.seek(Duration::from_secs(5));
        assert!((player.progress() - 0.5).abs() < 0.01);

        player.seek(Duration::from_secs(10));
        assert!((player.progress() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_player_skip() {
        let mut recording = Recording::new(80, 24);
        recording.start();
        recording.add_output(Duration::from_secs(0), b"Event 1");
        recording.add_output(Duration::from_secs(5), b"Event 2");
        recording.add_output(Duration::from_secs(10), b"Event 3");
        recording.stop();

        let mut player = RecordingPlayer::new(recording);

        player.skip_forward(Duration::from_secs(3));
        assert_eq!(player.current_time(), Duration::from_secs(3));

        player.skip_forward(Duration::from_secs(3));
        assert_eq!(player.current_time(), Duration::from_secs(6));

        player.skip_backward(Duration::from_secs(2));
        assert_eq!(player.current_time(), Duration::from_secs(4));
    }

    #[test]
    fn test_asciicast_format() {
        let mut recording = Recording::new(80, 24);
        recording.metadata.title = Some("Test Recording".to_string());
        recording.start();
        recording.add_output(Duration::from_secs(0), b"Hello");
        recording.add_input(Duration::from_millis(500), b"test\n");
        recording.stop();

        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.cast");

        // Save and verify file format
        recording.save_to_file(&path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // First line should be metadata JSON
        assert!(lines[0].starts_with("{\"version\":2"));

        // Subsequent lines should be events
        assert!(lines.len() >= 3); // Header + 2 events

        // Verify we can load it back
        let loaded = Recording::load_from_file(&path).unwrap();
        assert_eq!(loaded.metadata.version, 2);
        assert_eq!(loaded.metadata.width, 80);
        assert_eq!(loaded.metadata.height, 24);
        assert_eq!(loaded.len(), 2);
    }
}
