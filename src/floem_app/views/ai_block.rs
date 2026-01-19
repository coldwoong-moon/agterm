//! AI Block Rendering View
//!
//! This module provides components for rendering AI responses as blocks within the terminal.
//! Supports different block types (Thinking, Response, Command, Error) with risk assessment
//! for executable commands.

use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use floem::views::{container, dyn_container, h_stack, label, v_stack, Decorators};
use floem::peniko::Color;
use floem::text::Weight;
use floem::style::CursorStyle;

use crate::floem_app::async_bridge::RiskLevel;
use crate::floem_app::theme;

/// Type of AI block being displayed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiBlockType {
    /// AI is thinking/processing
    Thinking,
    /// AI response text
    Response,
    /// Executable command suggestion
    Command,
    /// Error message
    Error,
}

/// Risk level for command execution (mirrors async_bridge::RiskLevel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandRiskLevel {
    /// Low risk - safe commands (ls, cat, etc.)
    Low,
    /// Medium risk - commands that modify files
    Medium,
    /// High risk - commands that affect system state
    High,
    /// Critical risk - dangerous commands (rm -rf, etc.)
    Critical,
}

impl CommandRiskLevel {
    /// Get the color associated with this risk level
    pub fn color(&self) -> Color {
        match self {
            Self::Low => theme::colors::ACCENT_GREEN,
            Self::Medium => theme::colors::ACCENT_BLUE,
            Self::High => Color::rgb8(242, 197, 92),  // Yellow
            Self::Critical => theme::colors::ACCENT_RED,
        }
    }

    /// Get the display name for this risk level
    pub fn name(&self) -> &'static str {
        match self {
            Self::Low => "Low Risk",
            Self::Medium => "Medium Risk",
            Self::High => "High Risk",
            Self::Critical => "Critical Risk",
        }
    }
}

/// Convert from async_bridge::RiskLevel to CommandRiskLevel
impl From<RiskLevel> for CommandRiskLevel {
    fn from(level: RiskLevel) -> Self {
        match level {
            RiskLevel::Low => CommandRiskLevel::Low,
            RiskLevel::Medium => CommandRiskLevel::Medium,
            RiskLevel::High => CommandRiskLevel::High,
            RiskLevel::Critical => CommandRiskLevel::Critical,
        }
    }
}

/// Individual AI block data structure
#[derive(Debug, Clone)]
pub struct AiBlock {
    /// Unique identifier for this block
    pub id: String,
    /// Type of block
    pub block_type: AiBlockType,
    /// Main content text
    pub content: String,
    /// Command string (for Command blocks)
    pub command: Option<String>,
    /// Risk level (for Command blocks)
    pub risk_level: Option<CommandRiskLevel>,
    /// Whether the command has been executed
    pub is_executed: bool,
    /// Timestamp when block was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AiBlock {
    /// Create a new Thinking block
    pub fn thinking(id: String, content: String) -> Self {
        Self {
            id,
            block_type: AiBlockType::Thinking,
            content,
            command: None,
            risk_level: None,
            is_executed: false,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new Response block
    pub fn response(id: String, content: String) -> Self {
        Self {
            id,
            block_type: AiBlockType::Response,
            content,
            command: None,
            risk_level: None,
            is_executed: false,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new Command block
    pub fn command(id: String, content: String, command: String, risk_level: CommandRiskLevel) -> Self {
        Self {
            id,
            block_type: AiBlockType::Command,
            content,
            command: Some(command),
            risk_level: Some(risk_level),
            is_executed: false,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new Error block
    pub fn error(id: String, content: String) -> Self {
        Self {
            id,
            block_type: AiBlockType::Error,
            content,
            command: None,
            risk_level: None,
            is_executed: false,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// State management for AI blocks
#[derive(Clone)]
pub struct AiBlockState {
    /// List of all blocks
    pub blocks: RwSignal<Vec<AiBlock>>,
}

impl AiBlockState {
    /// Create a new empty AI block state
    pub fn new() -> Self {
        Self {
            blocks: RwSignal::new(Vec::new()),
        }
    }

    /// Add a new block
    pub fn add_block(&self, block: AiBlock) {
        self.blocks.update(|blocks| blocks.push(block));
    }

    /// Remove a block by ID
    pub fn remove_block(&self, id: &str) {
        self.blocks.update(|blocks| blocks.retain(|b| b.id != id));
    }

    /// Mark a command as executed
    pub fn mark_executed(&self, id: &str) {
        self.blocks.update(|blocks| {
            if let Some(block) = blocks.iter_mut().find(|b| b.id == id) {
                block.is_executed = true;
            }
        });
    }

    /// Clear all blocks
    pub fn clear(&self) {
        self.blocks.update(|blocks| blocks.clear());
    }
}

impl Default for AiBlockState {
    fn default() -> Self {
        Self::new()
    }
}

/// Main view for displaying all AI blocks
pub fn ai_blocks_view<FExec, FEdit, FCancel, FCopy>(
    state: &AiBlockState,
    on_execute: FExec,
    on_edit: FEdit,
    on_cancel: FCancel,
    on_copy: FCopy,
) -> impl IntoView
where
    FExec: Fn(String) + Clone + 'static,
    FEdit: Fn(String) + Clone + 'static,
    FCancel: Fn(String) + Clone + 'static,
    FCopy: Fn(String) + Clone + 'static,
{
    let blocks_signal = state.blocks;

    container(
        v_stack((
            // Header
            label(|| "AI Blocks")
                .style(|s| {
                    s.font_size(14.0)
                        .font_weight(Weight::SEMIBOLD)
                        .color(theme::colors::TEXT_PRIMARY)
                        .padding(8.0)
                }),

            // List of blocks
            dyn_stack(
                move || blocks_signal.get(),
                |block| block.id.clone(),
                move |block| ai_block_item(
                    block,
                    on_execute.clone(),
                    on_edit.clone(),
                    on_cancel.clone(),
                    on_copy.clone(),
                ),
            )
            .style(|s| s.flex_col().gap(8.0)),
        ))
        .style(|s| s.flex_col().gap(4.0).width_full()),
    )
    .style(|s| {
        s.width_full()
            .padding(12.0)
            .background(theme::colors::BG_SECONDARY)
            .border_radius(8.0)
    })
}

/// Render a single AI block based on its type
fn ai_block_item<FExec, FEdit, FCancel, FCopy>(
    block: AiBlock,
    on_execute: FExec,
    on_edit: FEdit,
    on_cancel: FCancel,
    on_copy: FCopy,
) -> impl IntoView
where
    FExec: Fn(String) + Clone + 'static,
    FEdit: Fn(String) + Clone + 'static,
    FCancel: Fn(String) + Clone + 'static,
    FCopy: Fn(String) + Clone + 'static,
{
    let block_type = block.block_type;
    dyn_container(
        move || block_type,
        move |bt| match bt {
            AiBlockType::Thinking => container(thinking_block_view(block.clone())),
            AiBlockType::Response => container(response_block_view(block.clone(), on_copy.clone())),
            AiBlockType::Command => container(command_block_view(
                block.clone(),
                on_execute.clone(),
                on_edit.clone(),
                on_cancel.clone(),
            )),
            AiBlockType::Error => container(error_block_view(block.clone())),
        },
    )
}

/// Thinking block view with loading animation
fn thinking_block_view(block: AiBlock) -> impl IntoView {
    container(
        h_stack((
            // Loading indicator (animated dots could be added with a timer)
            label(|| "●●●")
                .style(|s| {
                    s.font_size(12.0)
                        .color(theme::colors::ACCENT_BLUE)
                        .margin_right(8.0)
                }),

            // Thinking text
            label(move || block.content.clone())
                .style(|s| {
                    s.font_size(13.0)
                        .color(theme::colors::TEXT_SECONDARY)
                        .font_style(floem::text::Style::Italic)
                }),
        ))
        .style(|s| s.items_center().gap(8.0)),
    )
    .style(|s| {
        s.width_full()
            .padding(12.0)
            .background(theme::colors::BG_PRIMARY)
            .border_radius(6.0)
            .border(1.0)
            .border_color(theme::colors::ACCENT_BLUE)
    })
}

/// Response block view with copy button
fn response_block_view<FCopy>(block: AiBlock, on_copy: FCopy) -> impl IntoView
where
    FCopy: Fn(String) + Clone + 'static,
{
    let content = block.content.clone();
    let content_for_copy = content.clone();

    container(
        v_stack((
            // Response content
            label(move || content.clone())
                .style(|s| {
                    s.font_size(13.0)
                        .color(theme::colors::TEXT_PRIMARY)
                        .line_height(1.5)
                }),

            // Action buttons
            h_stack((
                // Copy button
                container(
                    label(|| "Copy")
                        .style(|s| {
                            s.font_size(11.0)
                                .color(theme::colors::ACCENT_BLUE)
                        })
                )
                .on_click_stop(move |_| {
                    tracing::info!("Copy button clicked");
                    on_copy(content_for_copy.clone());
                })
                .style(|s| {
                    s.padding_horiz(8.0)
                        .padding_vert(4.0)
                        .border_radius(4.0)
                        .border(1.0)
                        .border_color(theme::colors::BORDER)
                        .cursor(CursorStyle::Pointer)
                        .hover(|s| {
                            s.background(theme::colors::BG_HOVER)
                                .border_color(theme::colors::ACCENT_BLUE)
                        })
                }),
            ))
            .style(|s| s.margin_top(8.0).gap(8.0)),
        ))
        .style(|s| s.flex_col().width_full()),
    )
    .style(|s| {
        s.width_full()
            .padding(12.0)
            .background(theme::colors::BG_PRIMARY)
            .border_radius(6.0)
            .border(1.0)
            .border_color(theme::colors::BORDER_SUBTLE)
    })
}

/// Command block view with risk level indicator and action buttons
fn command_block_view<FExec, FEdit, FCancel>(
    block: AiBlock,
    on_execute: FExec,
    on_edit: FEdit,
    on_cancel: FCancel,
) -> impl IntoView
where
    FExec: Fn(String) + Clone + 'static,
    FEdit: Fn(String) + Clone + 'static,
    FCancel: Fn(String) + Clone + 'static,
{
    let content = block.content.clone();
    let command = block.command.clone().unwrap_or_default();
    let command_for_exec = command.clone();
    let command_for_edit = command.clone();
    let block_id = block.id.clone();
    let risk_level = block.risk_level.unwrap_or(CommandRiskLevel::Low);
    let is_executed = block.is_executed;
    let risk_color = risk_level.color();
    let risk_name = risk_level.name();

    container(
        v_stack((
            // Command description
            label(move || content.clone())
                .style(|s| {
                    s.font_size(13.0)
                        .color(theme::colors::TEXT_PRIMARY)
                        .margin_bottom(8.0)
                }),

            // Command box with risk level
            container(
                v_stack((
                    // Command text
                    label(move || command.clone())
                        .style(|s| {
                            s.font_size(12.0)
                                .font_family("JetBrains Mono, Noto Sans Mono CJK KR, Menlo, monospace".to_string())
                                .color(theme::colors::TEXT_PRIMARY)
                        }),

                    // Risk level indicator
                    label(move || format!("Risk: {risk_name}"))
                        .style(move |s| {
                            s.font_size(10.0)
                                .color(risk_color)
                                .margin_top(4.0)
                        }),
                ))
                .style(|s| s.flex_col()),
            )
            .style(move |s| {
                s.width_full()
                    .padding(10.0)
                    .background(theme::colors::BG_SECONDARY)
                    .border_radius(4.0)
                    .border(2.0)
                    .border_color(risk_color)
            }),

            // Action buttons (only show if not executed)
            dyn_container(
                move || is_executed,
                move |executed| {
                    if !executed {
                        let on_execute = on_execute.clone();
                        let on_edit = on_edit.clone();
                        let on_cancel = on_cancel.clone();
                        let cmd_exec = command_for_exec.clone();
                        let cmd_edit = command_for_edit.clone();
                        let bid = block_id.clone();

                        container(
                            h_stack((
                                // Execute button
                                container(
                                    label(|| "Execute")
                                        .style(|s| {
                                            s.font_size(11.0)
                                                .color(Color::WHITE)
                                        })
                                )
                                .on_click_stop(move |_| {
                                    tracing::info!("Execute button clicked for command: {}", cmd_exec);
                                    on_execute(cmd_exec.clone());
                                })
                                .style(|s| {
                                    s.padding_horiz(12.0)
                                        .padding_vert(6.0)
                                        .border_radius(4.0)
                                        .background(theme::colors::ACCENT_GREEN)
                                        .cursor(CursorStyle::Pointer)
                                        .hover(|s| {
                                            s.background(Color::rgb8(70, 180, 120))
                                        })
                                }),

                                // Edit button
                                container(
                                    label(|| "Edit")
                                        .style(|s| {
                                            s.font_size(11.0)
                                                .color(theme::colors::ACCENT_BLUE)
                                        })
                                )
                                .on_click_stop(move |_| {
                                    tracing::info!("Edit button clicked for command: {}", cmd_edit);
                                    on_edit(cmd_edit.clone());
                                })
                                .style(|s| {
                                    s.padding_horiz(12.0)
                                        .padding_vert(6.0)
                                        .border_radius(4.0)
                                        .border(1.0)
                                        .border_color(theme::colors::BORDER)
                                        .cursor(CursorStyle::Pointer)
                                        .hover(|s| {
                                            s.background(theme::colors::BG_HOVER)
                                                .border_color(theme::colors::ACCENT_BLUE)
                                        })
                                }),

                                // Cancel button
                                container(
                                    label(|| "Cancel")
                                        .style(|s| {
                                            s.font_size(11.0)
                                                .color(theme::colors::ACCENT_RED)
                                        })
                                )
                                .on_click_stop(move |_| {
                                    tracing::info!("Cancel button clicked for block: {}", bid);
                                    on_cancel(bid.clone());
                                })
                                .style(|s| {
                                    s.padding_horiz(12.0)
                                        .padding_vert(6.0)
                                        .border_radius(4.0)
                                        .border(1.0)
                                        .border_color(theme::colors::BORDER)
                                        .cursor(CursorStyle::Pointer)
                                        .hover(|s| {
                                            s.background(theme::colors::BG_HOVER)
                                                .border_color(theme::colors::ACCENT_RED)
                                        })
                                }),
                            ))
                            .style(|s| s.gap(8.0))
                        ).style(|s| s.margin_top(8.0))
                    } else {
                        // Executed indicator
                        container(
                            label(|| "✓ Executed")
                                .style(|s| {
                                    s.font_size(11.0)
                                        .color(theme::colors::ACCENT_GREEN)
                                })
                        )
                        .style(|s| s.margin_top(8.0))
                    }
                },
            ),
        ))
        .style(|s| s.flex_col().width_full()),
    )
    .style(|s| {
        s.width_full()
            .padding(12.0)
            .background(theme::colors::BG_PRIMARY)
            .border_radius(6.0)
            .border(1.0)
            .border_color(theme::colors::BORDER)
    })
}

/// Error block view with error styling
fn error_block_view(block: AiBlock) -> impl IntoView {
    let content = block.content.clone();

    container(
        v_stack((
            // Error icon and title
            h_stack((
                label(|| "✗")
                    .style(|s| {
                        s.font_size(16.0)
                            .color(theme::colors::ACCENT_RED)
                            .margin_right(8.0)
                    }),

                label(|| "Error")
                    .style(|s| {
                        s.font_size(13.0)
                            .font_weight(Weight::SEMIBOLD)
                            .color(theme::colors::ACCENT_RED)
                    }),
            ))
            .style(|s| s.items_center()),

            // Error content
            label(move || content.clone())
                .style(|s| {
                    s.font_size(13.0)
                        .color(theme::colors::TEXT_PRIMARY)
                        .margin_top(8.0)
                        .line_height(1.5)
                }),
        ))
        .style(|s| s.flex_col().width_full()),
    )
    .style(|s| {
        s.width_full()
            .padding(12.0)
            .background(Color::rgba8(235, 100, 115, 25))  // Semi-transparent red
            .border_radius(6.0)
            .border(2.0)
            .border_color(theme::colors::ACCENT_RED)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_block_creation() {
        let thinking = AiBlock::thinking("1".to_string(), "Thinking...".to_string());
        assert_eq!(thinking.block_type, AiBlockType::Thinking);
        assert_eq!(thinking.content, "Thinking...");

        let response = AiBlock::response("2".to_string(), "Here is the answer".to_string());
        assert_eq!(response.block_type, AiBlockType::Response);

        let command = AiBlock::command(
            "3".to_string(),
            "Run this command".to_string(),
            "ls -la".to_string(),
            CommandRiskLevel::Low,
        );
        assert_eq!(command.block_type, AiBlockType::Command);
        assert_eq!(command.command, Some("ls -la".to_string()));

        let error = AiBlock::error("4".to_string(), "Something went wrong".to_string());
        assert_eq!(error.block_type, AiBlockType::Error);
    }

    #[test]
    fn test_command_risk_level() {
        assert_eq!(CommandRiskLevel::Low.name(), "Low Risk");
        assert_eq!(CommandRiskLevel::Medium.name(), "Medium Risk");
        assert_eq!(CommandRiskLevel::High.name(), "High Risk");
        assert_eq!(CommandRiskLevel::Critical.name(), "Critical Risk");

        // Test ordering
        assert!(CommandRiskLevel::Low < CommandRiskLevel::Medium);
        assert!(CommandRiskLevel::Medium < CommandRiskLevel::High);
        assert!(CommandRiskLevel::High < CommandRiskLevel::Critical);
    }

    #[test]
    fn test_ai_block_state() {
        let state = AiBlockState::new();
        assert_eq!(state.blocks.get().len(), 0);

        state.add_block(AiBlock::thinking("1".to_string(), "Test".to_string()));
        assert_eq!(state.blocks.get().len(), 1);

        state.add_block(AiBlock::response("2".to_string(), "Test 2".to_string()));
        assert_eq!(state.blocks.get().len(), 2);

        state.remove_block("1");
        assert_eq!(state.blocks.get().len(), 1);
        assert_eq!(state.blocks.get()[0].id, "2");

        state.clear();
        assert_eq!(state.blocks.get().len(), 0);
    }

    #[test]
    fn test_mark_executed() {
        let state = AiBlockState::new();
        state.add_block(AiBlock::command(
            "1".to_string(),
            "Test".to_string(),
            "ls".to_string(),
            CommandRiskLevel::Low,
        ));

        assert!(!state.blocks.get()[0].is_executed);

        state.mark_executed("1");
        assert!(state.blocks.get()[0].is_executed);
    }
}
