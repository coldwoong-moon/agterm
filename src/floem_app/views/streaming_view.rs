//! Streaming Response View
//!
//! This module provides a real-time streaming view for AI agent responses with
//! animated cursor, token count, and elapsed time tracking.

use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use floem::views::{container, h_stack, label, scroll, v_stack, Decorators};
use floem::peniko::Color;
use floem::text::Weight;
use floem::style::CursorStyle;
use chrono::{DateTime, Utc};

use crate::floem_app::theme;

/// State management for streaming AI responses
#[derive(Clone, Copy)]
pub struct StreamingState {
    /// Whether streaming is currently active
    pub is_streaming: RwSignal<bool>,
    /// Name of the agent providing the response
    pub agent_name: RwSignal<String>,
    /// Accumulated content text
    pub content: RwSignal<String>,
    /// Token count
    pub token_count: RwSignal<usize>,
    /// Start time of the stream
    pub start_time: RwSignal<Option<DateTime<Utc>>>,
    /// Whether the stream can be stopped
    pub can_stop: RwSignal<bool>,
}

impl StreamingState {
    /// Create a new streaming state
    pub fn new() -> Self {
        Self {
            is_streaming: RwSignal::new(false),
            agent_name: RwSignal::new(String::new()),
            content: RwSignal::new(String::new()),
            token_count: RwSignal::new(0),
            start_time: RwSignal::new(None),
            can_stop: RwSignal::new(true),
        }
    }

    /// Start streaming from a new agent
    pub fn start(&self, agent_name: &str) {
        self.is_streaming.set(true);
        self.agent_name.set(agent_name.to_string());
        self.content.set(String::new());
        self.token_count.set(0);
        self.start_time.set(Some(Utc::now()));
        self.can_stop.set(true);
    }

    /// Append a chunk to the content
    pub fn append(&self, chunk: &str) {
        self.content.update(|content| {
            content.push_str(chunk);
        });

        // Rough token estimation (words * 1.3)
        let words = chunk.split_whitespace().count();
        let estimated_tokens = (words as f64 * 1.3) as usize;
        self.token_count.update(|count| {
            *count += estimated_tokens.max(1);
        });
    }

    /// Finish streaming
    pub fn finish(&self) {
        self.is_streaming.set(false);
        self.can_stop.set(false);
    }

    /// Stop streaming (user-initiated)
    pub fn stop(&self) {
        self.is_streaming.set(false);
        self.can_stop.set(false);
    }

    /// Calculate elapsed seconds since start
    pub fn elapsed_seconds(&self) -> f64 {
        if let Some(start) = self.start_time.get() {
            let elapsed = Utc::now().signed_duration_since(start);
            elapsed.num_milliseconds() as f64 / 1000.0
        } else {
            0.0
        }
    }
}

impl Default for StreamingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Main streaming response view
pub fn streaming_view<F>(
    state: &StreamingState,
    on_stop: F,
) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
    let is_streaming = state.is_streaming;
    let agent_name = state.agent_name;
    let content = state.content;
    let token_count = state.token_count;
    let can_stop = state.can_stop;
    let state_for_elapsed = *state;

    container(
        v_stack((
            // Header with agent name and stop button
            streaming_header(agent_name, is_streaming, can_stop, on_stop),

            // Content area with scrolling
            streaming_content(content, is_streaming),

            // Footer with stats
            streaming_footer(token_count, move || state_for_elapsed.elapsed_seconds()),
        ))
        .style(|s| s.flex_col().width_full().height_full()),
    )
    .style(|s| {
        s.width_full()
            .height_full()
            .background(theme::colors::BG_PRIMARY)
            .border_radius(8.0)
            .border(1.0)
            .border_color(theme::colors::BORDER)
    })
}

/// Header with agent name and stop button
fn streaming_header<F>(
    agent_name: RwSignal<String>,
    is_streaming: RwSignal<bool>,
    can_stop: RwSignal<bool>,
    on_stop: F,
) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
    container(
        h_stack((
            // Agent name with thinking indicator
            h_stack((
                label(|| "🤖")
                    .style(|s| {
                        s.font_size(16.0)
                            .margin_right(8.0)
                    }),

                label(move || {
                    let name = agent_name.get();
                    let streaming = is_streaming.get();
                    if streaming {
                        format!("{} is thinking...", name)
                    } else {
                        format!("{} (finished)", name)
                    }
                })
                .style(move |s| {
                    s.font_size(13.0)
                        .font_weight(Weight::SEMIBOLD)
                        .color(if is_streaming.get() {
                            theme::colors::ACCENT_BLUE
                        } else {
                            theme::colors::TEXT_SECONDARY
                        })
                }),
            ))
            .style(|s| s.items_center().flex_grow(1.0)),

            // Stop button (only when streaming and can stop)
            container(
                label(|| "Stop")
                    .style(|s| {
                        s.font_size(11.0)
                            .color(Color::WHITE)
                    })
            )
            .on_click_stop(move |_| {
                if is_streaming.get() && can_stop.get() {
                    tracing::info!("Stop button clicked");
                    on_stop();
                }
            })
            .style(move |s| {
                let active = is_streaming.get() && can_stop.get();
                s.padding_horiz(12.0)
                    .padding_vert(6.0)
                    .border_radius(4.0)
                    .background(if active {
                        theme::colors::ACCENT_RED
                    } else {
                        theme::colors::TEXT_DISABLED
                    })
                    .cursor(if active {
                        CursorStyle::Pointer
                    } else {
                        CursorStyle::Default
                    })
                    .apply_if(active, |s| {
                        s.hover(|s| s.background(Color::rgb8(210, 70, 85)))
                    })
            }),
        ))
        .style(|s| s.items_center().justify_between().width_full()),
    )
    .style(|s| {
        s.width_full()
            .padding(12.0)
            .background(theme::colors::BG_SECONDARY)
            .border_bottom(1.0)
            .border_color(theme::colors::BORDER_SUBTLE)
    })
}

/// Content area with markdown-style rendering and animated cursor
fn streaming_content(
    content: RwSignal<String>,
    is_streaming: RwSignal<bool>,
) -> impl IntoView {
    container(
        scroll(
            container(
                h_stack((
                    // Main content text
                    label(move || {
                        let text = content.get();
                        if text.is_empty() && is_streaming.get() {
                            "Starting...".to_string()
                        } else {
                            text
                        }
                    })
                    .style(move |s| {
                        s.font_size(13.0)
                            .color(if content.get().is_empty() && is_streaming.get() {
                                theme::colors::TEXT_MUTED
                            } else {
                                theme::colors::TEXT_PRIMARY
                            })
                            .line_height(1.6)
                            .font_family("SF Mono, JetBrains Mono, Menlo, monospace".to_string())
                    }),

                    // Animated cursor (only when streaming)
                    container(
                        label(|| "█")
                            .style(|s| {
                                s.font_size(13.0)
                                    .color(theme::colors::ACCENT_BLUE)
                            })
                    )
                    .style(move |s| {
                        s.display(if is_streaming.get() {
                            floem::style::Display::Flex
                        } else {
                            floem::style::Display::None
                        })
                        .margin_left(2.0)
                    }),
                ))
                .style(|s| s.items_start()),
            )
            .style(|s| {
                s.width_full()
                    .padding(16.0)
            }),
        )
        .style(|s| {
            s.width_full()
                .flex_grow(1.0)
        }),
    )
    .style(|s| {
        s.width_full()
            .flex_grow(1.0)
            .background(theme::colors::BG_PRIMARY)
    })
}

/// Footer with token count and elapsed time
fn streaming_footer<F>(
    token_count: RwSignal<usize>,
    elapsed_fn: F,
) -> impl IntoView
where
    F: Fn() -> f64 + 'static,
{
    container(
        h_stack((
            // Token count
            label(move || {
                let count = token_count.get();
                format!("Tokens: {}", count)
            })
            .style(|s| {
                s.font_size(11.0)
                    .color(theme::colors::TEXT_SECONDARY)
            }),

            // Separator
            label(|| "|")
                .style(|s| {
                    s.font_size(11.0)
                        .color(theme::colors::TEXT_MUTED)
                        .margin_horiz(8.0)
                }),

            // Elapsed time
            label(move || {
                let seconds = elapsed_fn();
                format!("Elapsed: {:.1}s", seconds)
            })
            .style(|s| {
                s.font_size(11.0)
                    .color(theme::colors::TEXT_SECONDARY)
            }),
        ))
        .style(|s| s.items_center()),
    )
    .style(|s| {
        s.width_full()
            .padding(8.0)
            .background(theme::colors::BG_SECONDARY)
            .border_top(1.0)
            .border_color(theme::colors::BORDER_SUBTLE)
    })
}

/// Animated blinking cursor (for future use with timer)
#[allow(dead_code)]
pub fn cursor_animation() -> impl IntoView {
    let visible = RwSignal::new(true);

    container(
        label(move || if visible.get() { "█" } else { " " })
            .style(|s| {
                s.font_size(13.0)
                    .color(theme::colors::ACCENT_BLUE)
                    .font_family("SF Mono, JetBrains Mono, Menlo, monospace".to_string())
            })
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_state_creation() {
        let state = StreamingState::new();
        assert!(!state.is_streaming.get());
        assert_eq!(state.agent_name.get(), "");
        assert_eq!(state.content.get(), "");
        assert_eq!(state.token_count.get(), 0);
        assert!(state.start_time.get().is_none());
        assert!(state.can_stop.get());
    }

    #[test]
    fn test_streaming_state_start() {
        let state = StreamingState::new();
        state.start("Claude");

        assert!(state.is_streaming.get());
        assert_eq!(state.agent_name.get(), "Claude");
        assert_eq!(state.content.get(), "");
        assert_eq!(state.token_count.get(), 0);
        assert!(state.start_time.get().is_some());
        assert!(state.can_stop.get());
    }

    #[test]
    fn test_streaming_state_append() {
        let state = StreamingState::new();
        state.start("Claude");

        state.append("Hello");
        assert_eq!(state.content.get(), "Hello");
        assert!(state.token_count.get() > 0);

        state.append(" world");
        assert_eq!(state.content.get(), "Hello world");
        assert!(state.token_count.get() > 1);
    }

    #[test]
    fn test_streaming_state_finish() {
        let state = StreamingState::new();
        state.start("Claude");
        state.append("Test");

        state.finish();
        assert!(!state.is_streaming.get());
        assert!(!state.can_stop.get());
        assert_eq!(state.content.get(), "Test");
    }

    #[test]
    fn test_streaming_state_stop() {
        let state = StreamingState::new();
        state.start("Claude");

        state.stop();
        assert!(!state.is_streaming.get());
        assert!(!state.can_stop.get());
    }

    #[test]
    fn test_streaming_state_elapsed_seconds() {
        let state = StreamingState::new();

        // Before start
        assert_eq!(state.elapsed_seconds(), 0.0);

        // After start
        state.start("Claude");
        std::thread::sleep(std::time::Duration::from_millis(100));
        let elapsed = state.elapsed_seconds();
        assert!(elapsed >= 0.1);
        assert!(elapsed < 0.3); // Should be around 0.1s with some margin
    }

    #[test]
    fn test_token_estimation() {
        let state = StreamingState::new();
        state.start("Claude");

        // Single word
        state.append("hello");
        let tokens1 = state.token_count.get();
        assert!(tokens1 >= 1);

        // Multiple words (should estimate more tokens)
        state.append(" world this is a test");
        let tokens2 = state.token_count.get();
        assert!(tokens2 > tokens1);
    }

    #[test]
    fn test_streaming_state_default() {
        let state = StreamingState::default();
        assert!(!state.is_streaming.get());
        assert_eq!(state.content.get(), "");
    }
}
