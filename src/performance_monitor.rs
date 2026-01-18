use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PerformanceMetric {
    FrameTime,
    RenderTime,
    InputLatency,
    PtyLatency,
    MemoryUsage,
    CpuUsage,
    ScrollbackSize,
    EventQueueSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricUnit {
    Milliseconds,
    Microseconds,
    Nanoseconds,
    Bytes,
    Kilobytes,
    Megabytes,
    Percent,
    Count,
    PerSecond,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendDirection {
    Rising,
    Falling,
    Stable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertCondition {
    Above(f64),
    Below(f64),
    SustainedAbove {
        threshold: f64,
        duration: Duration,
    },
    SustainedBelow {
        threshold: f64,
        duration: Duration,
    },
    RateOfChange(f64),
    StdDevExceeds(f64),
}

// ============================================================================
// Structs
// ============================================================================

#[derive(Debug, Clone)]
pub struct MetricSample {
    pub timestamp: Instant,
    pub value: f64,
    pub unit: MetricUnit,
}

impl MetricSample {
    pub fn new(value: f64, unit: MetricUnit) -> Self {
        Self {
            timestamp: Instant::now(),
            value,
            unit,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub p95: f64,
    pub p99: f64,
    pub std_dev: f64,
    pub sample_count: usize,
}

impl MetricStats {
    pub fn new() -> Self {
        Self {
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            median: 0.0,
            p95: 0.0,
            p99: 0.0,
            std_dev: 0.0,
            sample_count: 0,
        }
    }
}

impl Default for MetricStats {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TimeSeriesBuffer
// ============================================================================

#[derive(Debug)]
pub struct TimeSeriesBuffer<T> {
    capacity: usize,
    samples: VecDeque<T>,
}

impl<T> TimeSeriesBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            samples: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, sample: T) {
        if self.samples.len() >= self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.samples.get(index)
    }

    pub fn recent(&self, count: usize) -> Vec<&T> {
        let start = self.samples.len().saturating_sub(count);
        self.samples.range(start..).collect()
    }

    pub fn is_full(&self) -> bool {
        self.samples.len() >= self.capacity
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.samples.iter()
    }
}

impl TimeSeriesBuffer<MetricSample> {
    pub fn stats(&self) -> MetricStats {
        if self.samples.is_empty() {
            return MetricStats::new();
        }

        let mut values: Vec<f64> = self.samples.iter().map(|s| s.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = values[0];
        let max = values[values.len() - 1];
        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;

        let median = if values.len() % 2 == 0 {
            (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
        } else {
            values[values.len() / 2]
        };

        let p95_idx = ((values.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
        let p95 = values[p95_idx];

        let p99_idx = ((values.len() as f64 * 0.99).ceil() as usize).saturating_sub(1);
        let p99 = values[p99_idx];

        let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        MetricStats {
            min,
            max,
            mean,
            median,
            p95,
            p99,
            std_dev,
            sample_count: values.len(),
        }
    }
}

// ============================================================================
// PerformanceAlert
// ============================================================================

#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub id: String,
    pub metric: PerformanceMetric,
    pub condition: AlertCondition,
    pub message: String,
    pub triggered: bool,
    pub trigger_count: usize,
    pub last_triggered: Option<Instant>,
}

impl PerformanceAlert {
    pub fn new(
        id: impl Into<String>,
        metric: PerformanceMetric,
        condition: AlertCondition,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            metric,
            condition,
            message: message.into(),
            triggered: false,
            trigger_count: 0,
            last_triggered: None,
        }
    }

    pub fn check(&mut self, buffer: &TimeSeriesBuffer<MetricSample>) -> bool {
        let should_trigger = match &self.condition {
            AlertCondition::Above(threshold) => {
                buffer.recent(1).first().is_some_and(|s| s.value > *threshold)
            }
            AlertCondition::Below(threshold) => {
                buffer.recent(1).first().is_some_and(|s| s.value < *threshold)
            }
            AlertCondition::SustainedAbove { threshold, duration } => {
                self.check_sustained_above(buffer, *threshold, *duration)
            }
            AlertCondition::SustainedBelow { threshold, duration } => {
                self.check_sustained_below(buffer, *threshold, *duration)
            }
            AlertCondition::RateOfChange(max_rate) => {
                self.check_rate_of_change(buffer, *max_rate)
            }
            AlertCondition::StdDevExceeds(max_std_dev) => {
                let stats = buffer.stats();
                stats.std_dev > *max_std_dev
            }
        };

        if should_trigger {
            self.triggered = true;
            self.trigger_count += 1;
            self.last_triggered = Some(Instant::now());
        } else {
            self.triggered = false;
        }

        should_trigger
    }

    fn check_sustained_above(
        &self,
        buffer: &TimeSeriesBuffer<MetricSample>,
        threshold: f64,
        duration: Duration,
    ) -> bool {
        if buffer.is_empty() {
            return false;
        }

        let now = Instant::now();
        let cutoff = now.checked_sub(duration);

        buffer
            .iter()
            .rev()
            .take_while(|s| cutoff.map_or(true, |c| s.timestamp >= c))
            .all(|s| s.value > threshold)
    }

    fn check_sustained_below(
        &self,
        buffer: &TimeSeriesBuffer<MetricSample>,
        threshold: f64,
        duration: Duration,
    ) -> bool {
        if buffer.is_empty() {
            return false;
        }

        let now = Instant::now();
        let cutoff = now.checked_sub(duration);

        buffer
            .iter()
            .rev()
            .take_while(|s| cutoff.map_or(true, |c| s.timestamp >= c))
            .all(|s| s.value < threshold)
    }

    fn check_rate_of_change(&self, buffer: &TimeSeriesBuffer<MetricSample>, max_rate: f64) -> bool {
        let recent = buffer.recent(2);
        if recent.len() < 2 {
            return false;
        }

        let prev = recent[0];
        let curr = recent[1];
        let time_diff = curr.timestamp.duration_since(prev.timestamp).as_secs_f64();

        if time_diff == 0.0 {
            return false;
        }

        let rate = (curr.value - prev.value).abs() / time_diff;
        rate > max_rate
    }
}

// ============================================================================
// MonitorConfig
// ============================================================================

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub sample_buffer_size: usize,
    pub sample_interval: Duration,
    pub enable_cpu_monitoring: bool,
    pub enable_memory_monitoring: bool,
    pub alert_check_interval: Duration,
}

impl MonitorConfig {
    pub fn new() -> Self {
        Self {
            sample_buffer_size: 1000,
            sample_interval: Duration::from_millis(16),
            enable_cpu_monitoring: true,
            enable_memory_monitoring: true,
            alert_check_interval: Duration::from_secs(1),
        }
    }

    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.sample_buffer_size = size;
        self
    }

    pub fn with_sample_interval(mut self, interval: Duration) -> Self {
        self.sample_interval = interval;
        self
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PerformanceSummary
// ============================================================================

#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub fps: f64,
    pub frame_time_avg: f64,
    pub frame_time_p99: f64,
    pub input_latency_avg: f64,
    pub memory_usage_mb: f64,
    pub memory_trend: TrendDirection,
    pub active_alerts: Vec<String>,
}

impl PerformanceSummary {
    pub fn new() -> Self {
        Self {
            fps: 0.0,
            frame_time_avg: 0.0,
            frame_time_p99: 0.0,
            input_latency_avg: 0.0,
            memory_usage_mb: 0.0,
            memory_trend: TrendDirection::Unknown,
            active_alerts: Vec::new(),
        }
    }
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PerformanceMonitor
// ============================================================================

pub struct PerformanceMonitor {
    metrics: HashMap<PerformanceMetric, TimeSeriesBuffer<MetricSample>>,
    alerts: Vec<PerformanceAlert>,
    config: MonitorConfig,
    last_sample_time: HashMap<PerformanceMetric, Instant>,
}

impl PerformanceMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        let mut metrics = HashMap::new();

        for metric in [
            PerformanceMetric::FrameTime,
            PerformanceMetric::RenderTime,
            PerformanceMetric::InputLatency,
            PerformanceMetric::PtyLatency,
            PerformanceMetric::MemoryUsage,
            PerformanceMetric::CpuUsage,
            PerformanceMetric::ScrollbackSize,
            PerformanceMetric::EventQueueSize,
        ] {
            metrics.insert(
                metric,
                TimeSeriesBuffer::new(config.sample_buffer_size),
            );
        }

        Self {
            metrics,
            alerts: Vec::new(),
            config,
            last_sample_time: HashMap::new(),
        }
    }

    pub fn record(&mut self, metric: PerformanceMetric, value: f64) {
        let unit = Self::default_unit_for_metric(metric);
        self.record_with_unit(metric, value, unit);
    }

    pub fn record_with_unit(&mut self, metric: PerformanceMetric, value: f64, unit: MetricUnit) {
        let now = Instant::now();

        if let Some(last_time) = self.last_sample_time.get(&metric) {
            if now.duration_since(*last_time) < self.config.sample_interval {
                return;
            }
        }

        self.last_sample_time.insert(metric, now);

        if let Some(buffer) = self.metrics.get_mut(&metric) {
            buffer.push(MetricSample::new(value, unit));
        }
    }

    pub fn get_stats(&self, metric: PerformanceMetric) -> Option<MetricStats> {
        self.metrics.get(&metric).map(|buffer| buffer.stats())
    }

    pub fn get_recent(&self, metric: PerformanceMetric, count: usize) -> Vec<&MetricSample> {
        self.metrics
            .get(&metric)
            .map(|buffer| buffer.recent(count))
            .unwrap_or_default()
    }

    pub fn check_alerts(&mut self) -> Vec<&PerformanceAlert> {
        for alert in &mut self.alerts {
            if let Some(buffer) = self.metrics.get(&alert.metric) {
                alert.check(buffer);
            }
        }

        self.alerts.iter().filter(|a| a.triggered).collect()
    }

    pub fn add_alert(&mut self, alert: PerformanceAlert) {
        self.alerts.push(alert);
    }

    pub fn remove_alert(&mut self, id: &str) {
        self.alerts.retain(|a| a.id != id);
    }

    pub fn get_summary(&mut self) -> PerformanceSummary {
        let frame_stats = self.get_stats(PerformanceMetric::FrameTime);
        let input_stats = self.get_stats(PerformanceMetric::InputLatency);
        let memory_stats = self.get_stats(PerformanceMetric::MemoryUsage);

        let fps = frame_stats.as_ref().map_or(0.0, |s| {
            if s.mean > 0.0 {
                1000.0 / s.mean
            } else {
                0.0
            }
        });

        let frame_time_avg = frame_stats.as_ref().map_or(0.0, |s| s.mean);
        let frame_time_p99 = frame_stats.as_ref().map_or(0.0, |s| s.p99);
        let input_latency_avg = input_stats.as_ref().map_or(0.0, |s| s.mean);
        let memory_usage_mb = memory_stats.as_ref().map_or(0.0, |s| s.mean / 1_048_576.0);
        let memory_trend = self.calculate_trend(PerformanceMetric::MemoryUsage);

        let active_alerts = self.check_alerts()
            .iter()
            .map(|a| a.message.clone())
            .collect();

        PerformanceSummary {
            fps,
            frame_time_avg,
            frame_time_p99,
            input_latency_avg,
            memory_usage_mb,
            memory_trend,
            active_alerts,
        }
    }

    fn default_unit_for_metric(metric: PerformanceMetric) -> MetricUnit {
        match metric {
            PerformanceMetric::FrameTime => MetricUnit::Milliseconds,
            PerformanceMetric::RenderTime => MetricUnit::Milliseconds,
            PerformanceMetric::InputLatency => MetricUnit::Milliseconds,
            PerformanceMetric::PtyLatency => MetricUnit::Milliseconds,
            PerformanceMetric::MemoryUsage => MetricUnit::Bytes,
            PerformanceMetric::CpuUsage => MetricUnit::Percent,
            PerformanceMetric::ScrollbackSize => MetricUnit::Count,
            PerformanceMetric::EventQueueSize => MetricUnit::Count,
        }
    }

    fn calculate_trend(&self, metric: PerformanceMetric) -> TrendDirection {
        let recent = self.get_recent(metric, 10);
        if recent.len() < 5 {
            return TrendDirection::Unknown;
        }

        let mid = recent.len() / 2;
        let first_half: f64 = recent[..mid].iter().map(|s| s.value).sum::<f64>() / mid as f64;
        let second_half: f64 = recent[mid..].iter().map(|s| s.value).sum::<f64>() / (recent.len() - mid) as f64;

        let change_ratio = (second_half - first_half).abs() / first_half.max(1.0);

        if change_ratio < 0.05 {
            TrendDirection::Stable
        } else if second_half > first_half {
            TrendDirection::Rising
        } else {
            TrendDirection::Falling
        }
    }
}

// ============================================================================
// FrameTimer
// ============================================================================

pub struct FrameTimer {
    frame_start: Option<Instant>,
    last_frame_time: Duration,
    frame_count: usize,
    dropped_frames: usize,
    fps_window_start: Instant,
    target_frame_time: Duration,
}

impl FrameTimer {
    pub fn new() -> Self {
        Self {
            frame_start: None,
            last_frame_time: Duration::ZERO,
            frame_count: 0,
            dropped_frames: 0,
            fps_window_start: Instant::now(),
            target_frame_time: Duration::from_millis(16), // 60 FPS
        }
    }

    pub fn with_target_fps(target_fps: u32) -> Self {
        let target_frame_time = Duration::from_secs_f64(1.0 / target_fps as f64);
        Self {
            frame_start: None,
            last_frame_time: Duration::ZERO,
            frame_count: 0,
            dropped_frames: 0,
            fps_window_start: Instant::now(),
            target_frame_time,
        }
    }

    pub fn start_frame(&mut self) {
        self.frame_start = Some(Instant::now());
    }

    pub fn end_frame(&mut self) {
        if let Some(start) = self.frame_start {
            self.last_frame_time = start.elapsed();
            self.frame_count += 1;

            if self.last_frame_time > self.target_frame_time * 2 {
                self.dropped_frames += 1;
            }

            self.frame_start = None;
        }
    }

    pub fn fps(&self) -> f64 {
        let elapsed = self.fps_window_start.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.frame_count as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn frame_time(&self) -> Duration {
        self.last_frame_time
    }

    pub fn frames_dropped(&self) -> usize {
        self.dropped_frames
    }

    pub fn reset(&mut self) {
        self.frame_count = 0;
        self.dropped_frames = 0;
        self.fps_window_start = Instant::now();
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Timing Utilities
// ============================================================================

pub struct TimingGuard {
    start: Instant,
    metric: PerformanceMetric,
    monitor: *mut PerformanceMonitor,
}

impl TimingGuard {
    pub fn new(monitor: &mut PerformanceMonitor, metric: PerformanceMetric) -> Self {
        Self {
            start: Instant::now(),
            metric,
            monitor,
        }
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        unsafe {
            if !self.monitor.is_null() {
                (*self.monitor).record(self.metric, elapsed.as_secs_f64() * 1000.0);
            }
        }
    }
}

pub fn time_block<F, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

pub fn record_timing<F>(monitor: &mut PerformanceMonitor, metric: PerformanceMetric, f: F)
where
    F: FnOnce(),
{
    let start = Instant::now();
    f();
    let elapsed = start.elapsed();
    monitor.record(metric, elapsed.as_secs_f64() * 1000.0);
}

// ============================================================================
// Memory Tracking
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCATED_MEMORY: AtomicUsize = AtomicUsize::new(0);

pub fn estimate_memory_usage() -> usize {
    ALLOCATED_MEMORY.load(Ordering::Relaxed)
}

pub fn track_allocation(size: usize) {
    ALLOCATED_MEMORY.fetch_add(size, Ordering::Relaxed);
}

pub fn track_deallocation(size: usize) {
    ALLOCATED_MEMORY.fetch_sub(size, Ordering::Relaxed);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_metric_sample_creation() {
        let sample = MetricSample::new(42.5, MetricUnit::Milliseconds);
        assert_eq!(sample.value, 42.5);
        assert_eq!(sample.unit, MetricUnit::Milliseconds);
    }

    #[test]
    fn test_time_series_buffer_push() {
        let mut buffer = TimeSeriesBuffer::new(3);
        buffer.push(MetricSample::new(1.0, MetricUnit::Milliseconds));
        buffer.push(MetricSample::new(2.0, MetricUnit::Milliseconds));
        buffer.push(MetricSample::new(3.0, MetricUnit::Milliseconds));

        assert_eq!(buffer.len(), 3);
        assert!(buffer.is_full());
    }

    #[test]
    fn test_time_series_buffer_overflow() {
        let mut buffer = TimeSeriesBuffer::new(2);
        buffer.push(MetricSample::new(1.0, MetricUnit::Milliseconds));
        buffer.push(MetricSample::new(2.0, MetricUnit::Milliseconds));
        buffer.push(MetricSample::new(3.0, MetricUnit::Milliseconds));

        assert_eq!(buffer.len(), 2);
        let recent = buffer.recent(2);
        assert_eq!(recent[0].value, 2.0);
        assert_eq!(recent[1].value, 3.0);
    }

    #[test]
    fn test_time_series_buffer_recent() {
        let mut buffer = TimeSeriesBuffer::new(5);
        for i in 1..=5 {
            buffer.push(MetricSample::new(i as f64, MetricUnit::Milliseconds));
        }

        let recent = buffer.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].value, 3.0);
        assert_eq!(recent[1].value, 4.0);
        assert_eq!(recent[2].value, 5.0);
    }

    #[test]
    fn test_time_series_buffer_clear() {
        let mut buffer = TimeSeriesBuffer::new(3);
        buffer.push(MetricSample::new(1.0, MetricUnit::Milliseconds));
        buffer.push(MetricSample::new(2.0, MetricUnit::Milliseconds));
        buffer.clear();

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_metric_stats_calculation() {
        let mut buffer = TimeSeriesBuffer::new(10);
        for i in 1..=10 {
            buffer.push(MetricSample::new(i as f64, MetricUnit::Milliseconds));
        }

        let stats = buffer.stats();
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 10.0);
        assert_eq!(stats.mean, 5.5);
        assert_eq!(stats.median, 5.5);
        assert_eq!(stats.sample_count, 10);
    }

    #[test]
    fn test_metric_stats_percentiles() {
        let mut buffer = TimeSeriesBuffer::new(100);
        for i in 1..=100 {
            buffer.push(MetricSample::new(i as f64, MetricUnit::Milliseconds));
        }

        let stats = buffer.stats();
        assert!(stats.p95 >= 95.0);
        assert!(stats.p99 >= 99.0);
    }

    #[test]
    fn test_metric_stats_std_dev() {
        let mut buffer = TimeSeriesBuffer::new(5);
        buffer.push(MetricSample::new(2.0, MetricUnit::Milliseconds));
        buffer.push(MetricSample::new(4.0, MetricUnit::Milliseconds));
        buffer.push(MetricSample::new(4.0, MetricUnit::Milliseconds));
        buffer.push(MetricSample::new(4.0, MetricUnit::Milliseconds));
        buffer.push(MetricSample::new(5.0, MetricUnit::Milliseconds));

        let stats = buffer.stats();
        assert!(stats.std_dev > 0.0);
        assert!(stats.std_dev < 2.0);
    }

    #[test]
    fn test_performance_monitor_record() {
        let config = MonitorConfig::new().with_sample_interval(Duration::ZERO);
        let mut monitor = PerformanceMonitor::new(config);

        monitor.record(PerformanceMetric::FrameTime, 16.7);
        monitor.record(PerformanceMetric::FrameTime, 17.2);

        let stats = monitor.get_stats(PerformanceMetric::FrameTime).unwrap();
        assert_eq!(stats.sample_count, 2);
    }

    #[test]
    fn test_performance_monitor_get_recent() {
        let config = MonitorConfig::new().with_sample_interval(Duration::ZERO);
        let mut monitor = PerformanceMonitor::new(config);

        for i in 1..=5 {
            monitor.record(PerformanceMetric::InputLatency, i as f64);
        }

        let recent = monitor.get_recent(PerformanceMetric::InputLatency, 3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[2].value, 5.0);
    }

    #[test]
    fn test_alert_above_condition() {
        let mut alert = PerformanceAlert::new(
            "high_latency",
            PerformanceMetric::InputLatency,
            AlertCondition::Above(50.0),
            "Input latency too high",
        );

        let mut buffer = TimeSeriesBuffer::new(10);
        buffer.push(MetricSample::new(60.0, MetricUnit::Milliseconds));

        let triggered = alert.check(&buffer);
        assert!(triggered);
        assert_eq!(alert.trigger_count, 1);
    }

    #[test]
    fn test_alert_below_condition() {
        let mut alert = PerformanceAlert::new(
            "low_fps",
            PerformanceMetric::FrameTime,
            AlertCondition::Below(30.0),
            "FPS too low",
        );

        let mut buffer = TimeSeriesBuffer::new(10);
        buffer.push(MetricSample::new(20.0, MetricUnit::Milliseconds));

        let triggered = alert.check(&buffer);
        assert!(triggered);
    }

    #[test]
    fn test_alert_std_dev_exceeds() {
        let mut alert = PerformanceAlert::new(
            "unstable",
            PerformanceMetric::FrameTime,
            AlertCondition::StdDevExceeds(5.0),
            "Performance unstable",
        );

        let mut buffer = TimeSeriesBuffer::new(10);
        buffer.push(MetricSample::new(10.0, MetricUnit::Milliseconds));
        buffer.push(MetricSample::new(50.0, MetricUnit::Milliseconds));
        buffer.push(MetricSample::new(15.0, MetricUnit::Milliseconds));

        let triggered = alert.check(&buffer);
        assert!(triggered);
    }

    #[test]
    fn test_performance_monitor_add_remove_alert() {
        let config = MonitorConfig::new();
        let mut monitor = PerformanceMonitor::new(config);

        let alert = PerformanceAlert::new(
            "test_alert",
            PerformanceMetric::FrameTime,
            AlertCondition::Above(100.0),
            "Test alert",
        );

        monitor.add_alert(alert);
        assert_eq!(monitor.alerts.len(), 1);

        monitor.remove_alert("test_alert");
        assert_eq!(monitor.alerts.len(), 0);
    }

    #[test]
    fn test_frame_timer_fps() {
        let mut timer = FrameTimer::new();

        for _ in 0..10 {
            timer.start_frame();
            thread::sleep(Duration::from_millis(16));
            timer.end_frame();
        }

        let fps = timer.fps();
        assert!(fps > 0.0);
        assert!(fps <= 100.0);
    }

    #[test]
    fn test_frame_timer_dropped_frames() {
        let mut timer = FrameTimer::with_target_fps(60);

        timer.start_frame();
        thread::sleep(Duration::from_millis(50));
        timer.end_frame();

        assert_eq!(timer.frames_dropped(), 1);
    }

    #[test]
    fn test_timing_utilities_time_block() {
        let (result, duration) = time_block(|| {
            thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_memory_tracking() {
        let initial = estimate_memory_usage();
        track_allocation(1024);
        assert_eq!(estimate_memory_usage(), initial + 1024);

        track_deallocation(512);
        assert_eq!(estimate_memory_usage(), initial + 512);
    }

    #[test]
    fn test_performance_summary() {
        let config = MonitorConfig::new().with_sample_interval(Duration::ZERO);
        let mut monitor = PerformanceMonitor::new(config);

        monitor.record(PerformanceMetric::FrameTime, 16.7);
        monitor.record(PerformanceMetric::InputLatency, 5.0);
        monitor.record(PerformanceMetric::MemoryUsage, 1_048_576.0);

        let summary = monitor.get_summary();
        assert!(summary.fps > 0.0);
        assert!(summary.frame_time_avg > 0.0);
        assert!(summary.memory_usage_mb > 0.0);
    }

    #[test]
    fn test_trend_detection_rising() {
        let config = MonitorConfig::new().with_sample_interval(Duration::ZERO);
        let mut monitor = PerformanceMonitor::new(config);

        for i in 1..=10 {
            monitor.record(PerformanceMetric::MemoryUsage, i as f64 * 100.0);
        }

        let trend = monitor.calculate_trend(PerformanceMetric::MemoryUsage);
        assert_eq!(trend, TrendDirection::Rising);
    }

    #[test]
    fn test_trend_detection_falling() {
        let config = MonitorConfig::new().with_sample_interval(Duration::ZERO);
        let mut monitor = PerformanceMonitor::new(config);

        for i in (1..=10).rev() {
            monitor.record(PerformanceMetric::MemoryUsage, i as f64 * 100.0);
        }

        let trend = monitor.calculate_trend(PerformanceMetric::MemoryUsage);
        assert_eq!(trend, TrendDirection::Falling);
    }

    #[test]
    fn test_trend_detection_stable() {
        let config = MonitorConfig::new().with_sample_interval(Duration::ZERO);
        let mut monitor = PerformanceMonitor::new(config);

        for _ in 1..=10 {
            monitor.record(PerformanceMetric::MemoryUsage, 1000.0);
        }

        let trend = monitor.calculate_trend(PerformanceMetric::MemoryUsage);
        assert_eq!(trend, TrendDirection::Stable);
    }

    #[test]
    fn test_alert_sustained_above() {
        let mut alert = PerformanceAlert::new(
            "sustained_high",
            PerformanceMetric::CpuUsage,
            AlertCondition::SustainedAbove {
                threshold: 80.0,
                duration: Duration::from_millis(100),
            },
            "CPU sustained high",
        );

        let mut buffer = TimeSeriesBuffer::new(10);

        for _ in 0..5 {
            buffer.push(MetricSample::new(90.0, MetricUnit::Percent));
            thread::sleep(Duration::from_millis(25));
        }

        let triggered = alert.check(&buffer);
        assert!(triggered);
    }

    #[test]
    fn test_monitor_config_builder() {
        let config = MonitorConfig::new()
            .with_buffer_size(500)
            .with_sample_interval(Duration::from_millis(33));

        assert_eq!(config.sample_buffer_size, 500);
        assert_eq!(config.sample_interval, Duration::from_millis(33));
    }
}
