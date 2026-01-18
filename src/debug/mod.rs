//! Debug module for development and troubleshooting
//!
//! Provides:
//! - Debug UI panel with runtime information
//! - Performance metrics collection
//! - State inspection tools
//! - Event logging for debugging

pub mod event_log;
pub mod panel;

pub use event_log::{EventLog, EventType, LogEntry};
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
    /// Render times (frame render duration)
    render_times: VecDeque<Duration>,
    /// PTY read byte counts
    pty_bytes_read: VecDeque<usize>,
    /// PTY write byte counts
    pty_bytes_written: VecDeque<usize>,
    /// Maximum samples to keep
    max_samples: usize,
    /// Last render start time (for measuring render duration)
    last_render_start: Option<Instant>,
    /// FPS history for graphing (sampled every second)
    fps_history: VecDeque<f64>,
    /// Last FPS sample timestamp
    last_fps_sample: Instant,
    /// Memory usage history in MB (sampled every second)
    memory_history: VecDeque<f64>,
    /// PTY I/O rate history in bytes/sec (sampled every second)
    pty_io_history: VecDeque<f64>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new(100, 60) // 100 samples, 60 seconds of history
    }
}

#[allow(dead_code)]
impl Metrics {
    /// Create a new metrics collector with the specified sample size and history length
    pub fn new(max_samples: usize, history_seconds: usize) -> Self {
        let now = Instant::now();
        Self {
            frame_times: VecDeque::with_capacity(max_samples),
            last_frame: now,
            message_times: VecDeque::with_capacity(max_samples),
            render_times: VecDeque::with_capacity(max_samples),
            pty_bytes_read: VecDeque::with_capacity(max_samples),
            pty_bytes_written: VecDeque::with_capacity(max_samples),
            max_samples,
            last_render_start: None,
            fps_history: VecDeque::with_capacity(history_seconds),
            last_fps_sample: now,
            memory_history: VecDeque::with_capacity(history_seconds),
            pty_io_history: VecDeque::with_capacity(history_seconds),
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

        // Sample FPS history every second
        if now.duration_since(self.last_fps_sample).as_secs() >= 1 {
            let current_fps = self.fps();
            if self.fps_history.len() >= self.fps_history.capacity() {
                self.fps_history.pop_front();
            }
            self.fps_history.push_back(current_fps);
            self.last_fps_sample = now;
        }
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

    /// Start render time measurement
    pub fn start_render(&mut self) {
        self.last_render_start = Some(Instant::now());
    }

    /// Complete render time measurement
    pub fn end_render(&mut self) {
        if let Some(start) = self.last_render_start.take() {
            let duration = start.elapsed();
            if self.render_times.len() >= self.max_samples {
                self.render_times.pop_front();
            }
            self.render_times.push_back(duration);
        }
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

    /// Calculate average render time in milliseconds
    pub fn avg_render_time_ms(&self) -> f64 {
        if self.render_times.is_empty() {
            return 0.0;
        }
        let total: Duration = self.render_times.iter().sum();
        (total.as_secs_f64() * 1000.0) / self.render_times.len() as f64
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

    /// Update memory usage history (call periodically, e.g., every second)
    pub fn sample_memory(&mut self, memory_mb: f64) {
        if self.memory_history.len() >= self.memory_history.capacity() {
            self.memory_history.pop_front();
        }
        self.memory_history.push_back(memory_mb);
    }

    /// Update PTY I/O rate history (call periodically, e.g., every second)
    pub fn sample_pty_io(&mut self, bytes_per_sec: f64) {
        if self.pty_io_history.len() >= self.pty_io_history.capacity() {
            self.pty_io_history.pop_front();
        }
        self.pty_io_history.push_back(bytes_per_sec);
    }

    /// Get FPS history for graphing
    pub fn fps_history(&self) -> &VecDeque<f64> {
        &self.fps_history
    }

    /// Get memory history for graphing
    pub fn memory_history(&self) -> &VecDeque<f64> {
        &self.memory_history
    }

    /// Get PTY I/O history for graphing
    pub fn pty_io_history(&self) -> &VecDeque<f64> {
        &self.pty_io_history
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.frame_times.clear();
        self.message_times.clear();
        self.render_times.clear();
        self.pty_bytes_read.clear();
        self.pty_bytes_written.clear();
        self.fps_history.clear();
        self.memory_history.clear();
        self.pty_io_history.clear();
        self.last_frame = now;
        self.last_fps_sample = now;
        self.last_render_start = None;
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

#[allow(dead_code)]
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
#[allow(dead_code)]
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

#[allow(dead_code)]
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
        let mut metrics = Metrics::new(10, 60);

        // Simulate frames at 60 FPS (16.67ms per frame)
        for _ in 0..10 {
            metrics.frame_times.push_back(Duration::from_micros(16667));
        }

        let fps = metrics.fps();
        assert!((fps - 60.0).abs() < 1.0); // Allow small error
    }

    #[test]
    fn test_metrics_pty_stats() {
        let mut metrics = Metrics::new(10, 60);

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
