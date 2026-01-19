//! Command Queue View
//!
//! This module provides a UI component for displaying and managing the command execution queue.
//! It shows pending commands with their risk levels and provides approve/edit/reject actions.

use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use floem::views::{container, dyn_stack, h_stack, label, v_stack, Decorators, scroll};
use floem::peniko::Color;
use floem::text::Weight;
use floem::style::CursorStyle;

use crate::floem_app::async_bridge::RiskLevel;
use crate::floem_app::execution_pipeline::{ExecutionPipeline, PipelineItem, PipelineStage};
use crate::floem_app::theme;

/// Main command queue view
pub fn command_queue_view<FApprove, FEdit, FReject>(
    pipeline: ExecutionPipeline,
    theme: RwSignal<theme::Theme>,
    on_approve: FApprove,
    on_edit: FEdit,
    on_reject: FReject,
) -> impl IntoView
where
    FApprove: Fn(uuid::Uuid) + Clone + 'static,
    FEdit: Fn(uuid::Uuid, String) + Clone + 'static,
    FReject: Fn(uuid::Uuid) + Clone + 'static,
{
    let items = pipeline.items;
    let auto_level = pipeline.auto_approve_level;

    container(
        v_stack((
            // Header with auto-approve selector
            h_stack((
                label(|| "📋 Command Queue")
                    .style(move |s| {
                        let colors = theme.get().colors();
                        s.font_size(14.0)
                            .font_weight(Weight::SEMIBOLD)
                            .color(colors.text_primary)
                    }),
                // Auto-approve selector
                auto_approve_selector(auto_level, theme),
            ))
            .style(|s| {
                s.width_full()
                    .justify_between()
                    .items_center()
                    .padding_bottom(12.0)
            }),
            // Scrollable list of queue items
            scroll(
                dyn_stack(
                    move || items.get(),
                    |item| item.id,
                    move |item| {
                        queue_item_view(
                            item,
                            theme,
                            on_approve.clone(),
                            on_edit.clone(),
                            on_reject.clone(),
                        )
                    },
                )
                .style(|s| s.flex_col().gap(8.0).width_full()),
            )
            .style(|s| s.flex_grow(1.0).width_full()),
        ))
        .style(|s| s.flex_col().width_full().height_full()),
    )
    .style(move |s| {
        let colors = theme.get().colors();
        s.width_full()
            .height_full()
            .padding(12.0)
            .background(colors.bg_secondary)
            .border_radius(8.0)
    })
}

/// Auto-approve level selector dropdown
fn auto_approve_selector(
    level: RwSignal<RiskLevel>,
    theme: RwSignal<theme::Theme>,
) -> impl IntoView {
    let level_text = move || match level.get() {
        RiskLevel::Low => "Auto: Low",
        RiskLevel::Medium => "Auto: Medium",
        RiskLevel::High => "Auto: High",
        RiskLevel::Critical => "Auto: Critical",
    };

    container(
        h_stack((
            label(level_text).style(move |s| {
                let colors = theme.get().colors();
                s.font_size(12.0).color(colors.text_secondary)
            }),
            label(|| " ▼").style(move |s| {
                let colors = theme.get().colors();
                s.font_size(10.0).color(colors.text_muted)
            }),
        ))
        .style(|s| s.items_center().gap(4.0)),
    )
    .on_click_stop(move |_| {
        // Cycle through risk levels
        level.update(|l| {
            *l = match *l {
                RiskLevel::Low => RiskLevel::Medium,
                RiskLevel::Medium => RiskLevel::High,
                RiskLevel::High => RiskLevel::Critical,
                RiskLevel::Critical => RiskLevel::Low,
            };
        });
        tracing::info!("Auto-approve level changed to: {:?}", level.get());
    })
    .style(move |s| {
        let colors = theme.get().colors();
        s.padding_horiz(10.0)
            .padding_vert(5.0)
            .border_radius(4.0)
            .border(1.0)
            .border_color(colors.border)
            .cursor(CursorStyle::Pointer)
            .hover(move |s| s.background(colors.bg_tab_hover).border_color(colors.accent_blue))
    })
}

/// Individual queue item view
fn queue_item_view<FApprove, FEdit, FReject>(
    item: PipelineItem,
    theme: RwSignal<theme::Theme>,
    on_approve: FApprove,
    on_edit: FEdit,
    on_reject: FReject,
) -> impl IntoView
where
    FApprove: Fn(uuid::Uuid) + Clone + 'static,
    FEdit: Fn(uuid::Uuid, String) + Clone + 'static,
    FReject: Fn(uuid::Uuid) + Clone + 'static,
{
    let risk_level = item.risk_level;
    let risk_icon = match risk_level {
        RiskLevel::Low => "🟢",
        RiskLevel::Medium => "🟡",
        RiskLevel::High => "🔴",
        RiskLevel::Critical => "⚠️",
    };
    let risk_name = match risk_level {
        RiskLevel::Low => "LOW",
        RiskLevel::Medium => "MEDIUM",
        RiskLevel::High => "HIGH",
        RiskLevel::Critical => "CRITICAL",
    };
    let risk_color = match risk_level {
        RiskLevel::Low => Color::rgb8(16, 185, 129),    // #10B981
        RiskLevel::Medium => Color::rgb8(245, 158, 11),  // #F59E0B
        RiskLevel::High => Color::rgb8(249, 115, 22),    // #F97316
        RiskLevel::Critical => Color::rgb8(239, 68, 68), // #EF4444
    };

    let command = item.command.clone();
    let command_for_display = command.clone();
    let command_for_edit = command.clone();
    let agent_id = item.agent_id.clone();
    let item_id = item.id;
    let stage_signal = item.stage;
    let is_auto_approved = stage_signal.get() == PipelineStage::Approved;

    // Clone callbacks for use in the dynamic container
    let on_approve = on_approve.clone();
    let on_edit = on_edit.clone();
    let on_reject = on_reject.clone();

    container(
        v_stack((
            // Header row with risk indicator
            h_stack((
                label(move || format!("{} {}", risk_icon, risk_name)).style(move |s| {
                    s.font_size(11.0)
                        .font_weight(Weight::BOLD)
                        .color(risk_color)
                }),
                label(move || command_for_display.clone()).style(move |s| {
                    let colors = theme.get().colors();
                    s.font_size(13.0)
                        .font_family("JetBrains Mono, Noto Sans Mono CJK KR, Menlo, monospace".to_string())
                        .color(colors.text_primary)
                        .margin_left(12.0)
                        .flex_grow(1.0)
                }),
                // Auto-approved indicator
                if is_auto_approved {
                    label(|| "[Auto ✓]")
                        .style(move |s| {
                            let colors = theme.get().colors();
                            s.font_size(10.0)
                                .color(colors.accent_green)
                        })
                        .into_any()
                } else {
                    floem::views::empty().into_any()
                },
            ))
            .style(|s| s.width_full().items_center()),
            // Agent source
            label(move || format!("from: {}", agent_id.clone())).style(move |s| {
                let colors = theme.get().colors();
                s.font_size(11.0)
                    .color(colors.text_muted)
                    .margin_top(4.0)
            }),
            // Warning message for high/critical risk
            if matches!(risk_level, RiskLevel::High | RiskLevel::Critical) {
                label(|| "⚠️ This command requires manual review").style(move |s| {
                    s.font_size(11.0)
                        .color(Color::rgb8(245, 158, 11))
                        .margin_top(4.0)
                })
                .into_any()
            } else {
                floem::views::empty().into_any()
            },
            // Action buttons (dynamic based on stage)
            dyn_container(
                move || stage_signal.get(),
                move |stage| {
                    let on_approve_inner = on_approve.clone();
                    let on_edit_inner = on_edit.clone();
                    let on_reject_inner = on_reject.clone();
                    let cmd_for_edit = command_for_edit.clone();

                    if stage == PipelineStage::PendingApproval {
                        h_stack((
                            // Approve button
                            create_action_button(
                                "✓ Approve",
                                Color::WHITE,
                                Color::rgb8(16, 185, 129), // green
                                theme,
                                move || on_approve_inner(item_id),
                            ),
                            // Edit button
                            create_action_button(
                                "✎ Edit",
                                theme.get().colors().accent_blue,
                                Color::TRANSPARENT,
                                theme,
                                move || {
                                    // For now, just log - in real implementation would show a modal
                                    tracing::info!("Edit command: {}", cmd_for_edit);
                                    // Simulate editing by appending " --dry-run"
                                    let edited = format!("{} --dry-run", cmd_for_edit);
                                    on_edit_inner(item_id, edited);
                                },
                            ),
                            // Reject button
                            create_action_button(
                                "✗ Reject",
                                Color::rgb8(239, 68, 68), // red
                                Color::TRANSPARENT,
                                theme,
                                move || on_reject_inner(item_id),
                            ),
                        ))
                        .style(|s| s.gap(8.0).margin_top(8.0))
                        .into_any()
                    } else if stage == PipelineStage::Executing {
                        label(|| "Executing...")
                            .style(move |s| {
                                let colors = theme.get().colors();
                                s.font_size(11.0)
                                    .color(colors.accent_blue)
                                    .margin_top(8.0)
                            })
                            .into_any()
                    } else if stage == PipelineStage::Completed {
                        label(|| "✓ Completed")
                            .style(move |s| {
                                let colors = theme.get().colors();
                                s.font_size(11.0)
                                    .color(colors.accent_green)
                                    .margin_top(8.0)
                            })
                            .into_any()
                    } else if stage == PipelineStage::Failed {
                        label(|| "✗ Failed")
                            .style(move |s| {
                                s.font_size(11.0)
                                    .color(Color::rgb8(239, 68, 68))
                                    .margin_top(8.0)
                            })
                            .into_any()
                    } else if stage == PipelineStage::Cancelled {
                        label(|| "Cancelled")
                            .style(move |s| {
                                let colors = theme.get().colors();
                                s.font_size(11.0)
                                    .color(colors.text_muted)
                                    .margin_top(8.0)
                            })
                            .into_any()
                    } else {
                        floem::views::empty().into_any()
                    }
                },
            ),
        ))
        .style(|s| s.flex_col().width_full()),
    )
    .style(move |s| {
        let colors = theme.get().colors();
        s.width_full()
            .padding(12.0)
            .background(colors.bg_primary)
            .border_radius(6.0)
            .border(1.0)
            .border_color(colors.border)
            .box_shadow_blur(4.0)
            .hover(move |s| s.background(colors.bg_tab_hover))
    })
}

/// Create an action button with consistent styling
fn create_action_button<F>(
    text: &'static str,
    text_color: Color,
    bg_color: Color,
    theme: RwSignal<theme::Theme>,
    on_click: F,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    container(label(move || text).style(move |s| s.font_size(11.0).color(text_color)))
        .on_click_stop(move |_| {
            tracing::info!("Button clicked: {}", text);
            on_click();
        })
        .style(move |s| {
            let colors = theme.get().colors();
            let is_filled = bg_color.a > 0;

            let mut style = s
                .padding_horiz(12.0)
                .padding_vert(6.0)
                .border_radius(4.0)
                .cursor(CursorStyle::Pointer);

            if is_filled {
                style = style.background(bg_color).hover(move |s| {
                    // Slightly darken on hover
                    let hover_color = Color::rgba8(bg_color.r, bg_color.g, bg_color.b, 220);
                    s.background(hover_color)
                });
            } else {
                style = style
                    .border(1.0)
                    .border_color(colors.border)
                    .hover(move |s| s.background(colors.bg_tab_hover).border_color(text_color));
            }

            style
        })
}
