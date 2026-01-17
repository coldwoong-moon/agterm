//! Debug module for development and troubleshooting
//!
//! Provides:
//! - Debug UI panel with runtime information
//! - Performance metrics collection
//! - State inspection tools

pub mod panel;

pub use panel::{DebugPanel, DebugPanelMessage};

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Performance metrics for the application
#[derive(Debug, Clone)]
pub struct Metrics {
    /// Frame times for FPS calculation
    frame_times: VecDeque<Duration>,
    /// Last frame timestamp
    last_frame: Instant,
    /// Message processing times
    message_times: VecDeque<Duration>,
    /// PTY read byte counts
    pty_bytes_read: VecDeque<usize>,
    /// PTY write byte counts
    pty_bytes_written: VecDeque<usize>,
    /// Maximum samples to keep
    max_samples: usize,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new(100)
    }
}

impl Metrics {
    /// Create a new metrics collector with the specified sample size
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(max_samples),
            last_frame: Instant::now(),
            message_times: VecDeque::with_capacity(max_samples),
            pty_bytes_read: VecDeque::with_capacity(max_samples),
            pty_bytes_written: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    /// Record a frame completion
    pub fn record_frame(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame);
        self.last_frame = now;

        if self.frame_times.len() >= self.max_samples {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(frame_time);
    }

    /// Record message processing time
    pub fn record_message_time(&mut self, duration: Duration) {
        if self.message_times.len() >= self.max_samples {
            self.message_times.pop_front();
        }
        self.message_times.push_back(duration);
    }

    /// Record PTY read bytes
    pub fn record_pty_read(&mut self, bytes: usize) {
        if self.pty_bytes_read.len() >= self.max_samples {
            self.pty_bytes_read.pop_front();
        }
        self.pty_bytes_read.push_back(bytes);
    }

    /// Record PTY write bytes
    pub fn record_pty_write(&mut self, bytes: usize) {
        if self.pty_bytes_written.len() >= self.max_samples {
            self.pty_bytes_written.pop_front();
        }
        self.pty_bytes_written.push_back(bytes);
    }

    /// Calculate average FPS
    pub fn fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let total: Duration = self.frame_times.iter().sum();
        let avg_frame_time = total.as_secs_f64() / self.frame_times.len() as f64;
        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }

    /// Calculate average frame time in milliseconds
    pub fn avg_frame_time_ms(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let total: Duration = self.frame_times.iter().sum();
        (total.as_secs_f64() * 1000.0) / self.frame_times.len() as f64
    }

    /// Calculate average message processing time in microseconds
    pub fn avg_message_time_us(&self) -> f64 {
        if self.message_times.is_empty() {
            return 0.0;
        }
        let total: Duration = self.message_times.iter().sum();
        (total.as_secs_f64() * 1_000_000.0) / self.message_times.len() as f64
    }

    /// Calculate total PTY bytes read in the sample window
    pub fn total_pty_bytes_read(&self) -> usize {
        self.pty_bytes_read.iter().sum()
    }

    /// Calculate total PTY bytes written in the sample window
    pub fn total_pty_bytes_written(&self) -> usize {
        self.pty_bytes_written.iter().sum()
    }

    /// Calculate PTY read bytes per second (approximate)
    pub fn pty_read_bytes_per_sec(&self) -> f64 {
        if self.pty_bytes_read.is_empty() {
            return 0.0;
        }
        // Assuming samples are taken at roughly 100ms intervals (tick rate)
        let total: usize = self.pty_bytes_read.iter().sum();
        let sample_duration_secs = self.pty_bytes_read.len() as f64 * 0.1;
        if sample_duration_secs > 0.0 {
            total as f64 / sample_duration_secs
        } else {
            0.0
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.message_times.clear();
        self.pty_bytes_read.clear();
        self.pty_bytes_written.clear();
        self.last_frame = Instant::now();
    }
}

/// Input debug state
#[derive(Debug, Clone, Default)]
pub struct InputDebugState {
    /// Last key pressed (keycode representation)
    pub last_key: Option<String>,
    /// Last key modifiers
    pub last_modifiers: Option<String>,
    /// IME composing state
    pub ime_composing: bool,
    /// IME preedit text
    pub ime_preedit: String,
    /// Raw mode active
    pub raw_mode: bool,
}

impl InputDebugState {
    /// Update with a new key press
    pub fn record_key(&mut self, key: &str, modifiers: &str) {
        self.last_key = Some(key.to_string());
        self.last_modifiers = Some(modifiers.to_string());
    }

    /// Update IME state
    pub fn update_ime(&mut self, composing: bool, preedit: &str) {
        self.ime_composing = composing;
        self.ime_preedit = preedit.to_string();
    }
}

/// PTY session debug info
#[derive(Debug, Clone)]
pub struct PtyDebugInfo {
    /// Session ID
    pub session_id: String,
    /// Total bytes read
    pub bytes_read: usize,
    /// Total bytes written
    pub bytes_written: usize,
    /// Output buffer size
    pub buffer_size: usize,
    /// Session start time
    pub started_at: Instant,
    /// Is session active
    pub active: bool,
}

impl PtyDebugInfo {
    /// Create a new PTY debug info
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            bytes_read: 0,
            bytes_written: 0,
            buffer_size: 0,
            started_at: Instant::now(),
            active: true,
        }
    }

    /// Get session uptime
    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_fps() {
        let mut metrics = Metrics::new(10);

        // Simulate frames at 60 FPS (16.67ms per frame)
        for _ in 0..10 {
            metrics.frame_times.push_back(Duration::from_micros(16667));
        }

        let fps = metrics.fps();
        assert!((fps - 60.0).abs() < 1.0); // Allow small error
    }

    #[test]
    fn test_metrics_pty_stats() {
        let mut metrics = Metrics::new(10);

        metrics.record_pty_read(100);
        metrics.record_pty_read(200);
        metrics.record_pty_write(50);

        assert_eq!(metrics.total_pty_bytes_read(), 300);
        assert_eq!(metrics.total_pty_bytes_written(), 50);
    }

    #[test]
    fn test_input_debug_state() {
        let mut state = InputDebugState::default();

        state.record_key("A", "Ctrl");
        assert_eq!(state.last_key, Some("A".to_string()));
        assert_eq!(state.last_modifiers, Some("Ctrl".to_string()));

        state.update_ime(true, "한");
        assert!(state.ime_composing);
        assert_eq!(state.ime_preedit, "한");
    }

    #[test]
    fn test_pty_debug_info() {
        let info = PtyDebugInfo::new("test-session".to_string());
        assert!(info.active);
        assert_eq!(info.bytes_read, 0);
        assert!(info.uptime() < Duration::from_secs(1));
    }
}
