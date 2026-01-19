//! Agent Dashboard View for Floem UI
//!
//! Provides a dashboard for managing multiple AI agents and viewing their connection status.

use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate};
use floem::style::{AlignItems, CursorStyle, FlexDirection, JustifyContent};
use floem::views::{container, dyn_container, h_stack, label, scroll, v_stack, Decorators};

use crate::floem_app::theme::Theme;

/// Connection state for an agent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Agent is connected and ready
    Connected,
    /// Agent is disconnected
    Disconnected,
    /// Agent is in the process of connecting
    Connecting,
    /// Agent is not configured
    NotConfigured,
}

impl ConnectionState {
    /// Get status emoji for the connection state
    pub fn emoji(&self) -> &'static str {
        match self {
            ConnectionState::Connected => "🟢",
            ConnectionState::Disconnected => "🔴",
            ConnectionState::Connecting => "🟡",
            ConnectionState::NotConfigured => "⚪",
        }
    }

    /// Get status text for the connection state
    pub fn status_text(&self) -> &'static str {
        match self {
            ConnectionState::Connected => "Connected",
            ConnectionState::Disconnected => "Disconnected",
            ConnectionState::Connecting => "Connecting...",
            ConnectionState::NotConfigured => "Not Configured",
        }
    }

    /// Get action button text
    pub fn action_text(&self) -> &'static str {
        match self {
            ConnectionState::Connected => "Disconnect",
            ConnectionState::Disconnected => "Connect",
            ConnectionState::Connecting => "...",
            ConnectionState::NotConfigured => "Setup",
        }
    }
}

/// Configuration for a single agent
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Agent name (e.g., "Claude", "Gemini", "Custom")
    pub name: String,
    /// Agent description
    pub description: String,
    /// Number of available tools
    pub tools_count: usize,
    /// Connection state
    pub connection_state: ConnectionState,
}

impl AgentConfig {
    /// Create a new agent configuration
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            tools_count: 0,
            connection_state: ConnectionState::NotConfigured,
        }
    }

    /// Set the number of tools
    pub fn with_tools(mut self, count: usize) -> Self {
        self.tools_count = count;
        self
    }

    /// Set the connection state
    pub fn with_state(mut self, state: ConnectionState) -> Self {
        self.connection_state = state;
        self
    }
}

/// Session information for the active agent
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Active agent name
    pub agent_name: String,
    /// Number of available tools
    pub tools_count: usize,
    /// Number of commands pending approval
    pub pending_commands: usize,
    /// Number of commands executed today
    pub executed_today: usize,
}

impl Default for SessionInfo {
    fn default() -> Self {
        Self {
            agent_name: "None".to_string(),
            tools_count: 0,
            pending_commands: 0,
            executed_today: 0,
        }
    }
}

/// Registry of available agents
#[derive(Debug, Clone)]
pub struct AgentRegistry {
    /// List of configured agents
    pub agents: RwSignal<Vec<AgentConfig>>,
}

impl AgentRegistry {
    /// Create a new agent registry with default agents
    pub fn new() -> Self {
        let default_agents = vec![
            AgentConfig::new("Claude", "Claude Code by Anthropic")
                .with_tools(15)
                .with_state(ConnectionState::Connected),
            AgentConfig::new("Gemini", "Gemini CLI by Google")
                .with_state(ConnectionState::Disconnected),
            AgentConfig::new("Custom", "Custom MCP Server")
                .with_state(ConnectionState::NotConfigured),
        ];

        Self {
            agents: RwSignal::new(default_agents),
        }
    }

    /// Get an agent by name
    pub fn get_agent(&self, name: &str) -> Option<AgentConfig> {
        self.agents
            .get()
            .iter()
            .find(|a| a.name == name)
            .cloned()
    }

    /// Update an agent's configuration
    pub fn update_agent<F>(&self, name: &str, f: F)
    where
        F: FnOnce(&mut AgentConfig),
    {
        self.agents.update(|agents| {
            if let Some(agent) = agents.iter_mut().find(|a| a.name == name) {
                f(agent);
            }
        });
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent Dashboard state with reactive signals
#[derive(Clone)]
pub struct AgentDashboardState {
    /// Whether the dashboard is visible
    pub visible: RwSignal<bool>,
    /// Registry of available agents
    pub registry: AgentRegistry,
    /// Currently selected agent
    pub selected_agent: RwSignal<Option<String>>,
    /// Active session information
    pub session_info: RwSignal<SessionInfo>,
}

impl AgentDashboardState {
    /// Create a new agent dashboard state
    pub fn new() -> Self {
        Self {
            visible: RwSignal::new(false),
            registry: AgentRegistry::new(),
            selected_agent: RwSignal::new(Some("Claude".to_string())),
            session_info: RwSignal::new(SessionInfo {
                agent_name: "Claude Code".to_string(),
                tools_count: 15,
                pending_commands: 8,
                executed_today: 23,
            }),
        }
    }

    /// Toggle dashboard visibility
    pub fn toggle_visibility(&self) {
        self.visible.update(|v| *v = !*v);
    }

    /// Select an agent
    pub fn select_agent(&self, agent_name: String) {
        self.selected_agent.set(Some(agent_name));
    }

    /// Update session information
    pub fn update_session(&self, info: SessionInfo) {
        self.session_info.set(info);
    }
}

impl Default for AgentDashboardState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the agent dashboard view
pub fn agent_dashboard(state: AgentDashboardState, theme: RwSignal<Theme>) -> impl IntoView {
    let visible = state.visible;

    dyn_container(
        move || visible.get(),
        move |is_visible| {
            if !is_visible {
                return container(label(|| ""))
                    .style(|s| s.display(floem::style::Display::None));
            }

            // Dashboard content
            container(
                v_stack((
                    // Header section
                    dashboard_header(state.clone(), theme),
                    // Agent cards grid
                    agent_cards_grid(state.clone(), theme),
                    // Active session info
                    active_session_section(state.clone(), theme),
                ))
                .style(move |s| {
                    s.flex_direction(FlexDirection::Column)
                        .width_full()
                        .height_full()
                        .gap(0.0)
                }),
            )
            .style(move |s| {
                let colors = theme.get().colors();
                s.width(500.0)
                    .height(400.0)
                    .background(colors.bg_primary)
                    .border(1.0)
                    .border_color(colors.border)
                    .border_radius(8.0)
                    .position(floem::style::Position::Absolute)
                    .inset_top(50.0)
                    .inset_left(50.0)
            })
        },
    )
}

/// Dashboard header with title and close button
fn dashboard_header(state: AgentDashboardState, theme: RwSignal<Theme>) -> impl IntoView {
    let state_clone = state.clone();

    h_stack((
        label(|| "🤖 Agent Dashboard".to_string()).style(move |s| {
            let colors = theme.get().colors();
            s.font_size(16.0)
                .font_weight(floem::text::Weight::BOLD)
                .color(colors.text_primary)
                .flex_grow(1.0)
        }),
        // Close button
        container(label(|| "×".to_string()))
            .on_click_stop(move |_| {
                state_clone.toggle_visibility();
            })
            .style(move |s| {
                let colors = theme.get().colors();
                s.padding(4.0)
                    .font_size(20.0)
                    .color(colors.text_primary)
                    .cursor(CursorStyle::Pointer)
                    .hover(|s| s.background(colors.bg_tab_hover))
                    .border_radius(4.0)
            }),
    ))
    .style(move |s| {
        let colors = theme.get().colors();
        s.width_full()
            .padding(12.0)
            .background(colors.bg_secondary)
            .border_bottom(1.0)
            .border_color(colors.border)
            .justify_content(JustifyContent::SpaceBetween)
            .align_items(AlignItems::Center)
    })
}

/// Grid of agent cards
fn agent_cards_grid(state: AgentDashboardState, theme: RwSignal<Theme>) -> impl IntoView {
    let agents = state.registry.agents;

    scroll(
        dyn_container(
            move || agents.get(),
            move |agent_list| {
                container(
                    h_stack((
                        agent_list
                            .iter()
                            .map(|agent| {
                                let agent_clone = agent.clone();
                                let state_clone = state.clone();
                                agent_card(&agent_clone, theme, move |name| {
                                    state_clone.select_agent(name);
                                })
                            })
                            .collect::<Vec<_>>(),
                    ))
                    .style(|s| s.width_full().gap(12.0).flex_wrap(floem::style::FlexWrap::Wrap)),
                )
            },
        )
        .style(|s| s.width_full().padding(12.0)),
    )
    .style(move |s| {
        let colors = theme.get().colors();
        s.width_full()
            .max_height(200.0)
            .background(colors.bg_primary)
    })
}

/// Individual agent card
fn agent_card<F>(agent: &AgentConfig, theme: RwSignal<Theme>, on_click: F) -> impl IntoView
where
    F: Fn(String) + Clone + 'static,
{
    let agent_name = agent.name.clone();
    let state = agent.connection_state;
    let tools_count = agent.tools_count;
    let name_for_click = agent_name.clone();
    let on_click_rc = std::rc::Rc::new(on_click);

    v_stack((
        // Status emoji and name
        h_stack((
            label(move || state.emoji().to_string()).style(move |s| s.font_size(24.0)),
            label(move || agent_name.clone()).style(move |s| {
                let colors = theme.get().colors();
                s.font_size(14.0)
                    .font_weight(floem::text::Weight::SEMIBOLD)
                    .color(colors.text_primary)
                    .margin_left(8.0)
            }),
        ))
        .style(|s| s.align_items(AlignItems::Center)),
        // Status text
        label(move || state.status_text().to_string()).style(move |s| {
            let colors = theme.get().colors();
            let color = match state {
                ConnectionState::Connected => colors.accent_green,
                ConnectionState::Disconnected => colors.accent_red,
                ConnectionState::Connecting => colors.accent_yellow,
                ConnectionState::NotConfigured => colors.text_muted,
            };
            s.font_size(11.0).color(color).margin_top(4.0)
        }),
        // Tools count or action button
        dyn_container(
            move || state,
            move |current_state| {
                if current_state == ConnectionState::Connected {
                    container(label(move || format!("{} tools", tools_count)))
                        .style(move |s| {
                            let colors = theme.get().colors();
                            s.font_size(11.0)
                                .color(colors.text_secondary)
                                .margin_top(4.0)
                        })
                } else {
                    let name_for_button = name_for_click.clone();
                    let on_click_clone = on_click_rc.clone();
                    container(label(move || current_state.action_text().to_string()))
                        .on_click_stop(move |_| {
                            on_click_clone(name_for_button.clone());
                        })
                        .style(move |s| {
                            let colors = theme.get().colors();
                            s.padding_horiz(8.0)
                                .padding_vert(4.0)
                                .font_size(10.0)
                                .border(1.0)
                                .border_radius(4.0)
                                .margin_top(8.0)
                                .cursor(CursorStyle::Pointer)
                                .background(colors.accent_blue)
                                .border_color(colors.accent_blue)
                                .color(Color::WHITE)
                                .hover(|s| {
                                    s.background(Color::rgba8(92, 138, 250, 220))
                                })
                        })
                }
            },
        ),
    ))
    .style(move |s| {
        let colors = theme.get().colors();
        s.padding(12.0)
            .background(colors.bg_secondary)
            .border(1.0)
            .border_color(colors.border_subtle)
            .border_radius(6.0)
            .min_width(140.0)
            .flex_grow(1.0)
            .align_items(AlignItems::Start)
    })
}

/// Active session information section
fn active_session_section(state: AgentDashboardState, theme: RwSignal<Theme>) -> impl IntoView {
    let session_info = state.session_info;

    dyn_container(
        move || session_info.get(),
        move |info| {
            container(
                v_stack((
                    // Session title
                    label(move || format!("Active Session: {}", info.agent_name.clone()))
                        .style(move |s| {
                            let colors = theme.get().colors();
                            s.font_size(13.0)
                                .font_weight(floem::text::Weight::SEMIBOLD)
                                .color(colors.text_primary)
                        }),
                    // Session details
                    v_stack((
                        session_detail_row("├─ Tools:", &format!("{} available", info.tools_count), theme),
                        session_detail_row(
                            "├─ Commands:",
                            &format!("{} pending approval", info.pending_commands),
                            theme,
                        ),
                        session_detail_row(
                            "└─ History:",
                            &format!("{} executed today", info.executed_today),
                            theme,
                        ),
                    ))
                    .style(|s| s.margin_top(8.0).gap(4.0)),
                ))
                .style(|s| s.width_full()),
            )
            .style(move |s| {
                let colors = theme.get().colors();
                s.width_full()
                    .padding(12.0)
                    .background(colors.bg_secondary)
                    .border_top(1.0)
                    .border_color(colors.border)
            })
        },
    )
}

/// Session detail row helper
fn session_detail_row(label_text: &'static str, value_text: &str, theme: RwSignal<Theme>) -> impl IntoView {
    let value_owned = value_text.to_string();
    h_stack((
        label(move || label_text.to_string()).style(move |s| {
            let colors = theme.get().colors();
            s.font_size(11.0)
                .color(colors.text_muted)
                .font_family("SF Mono".to_string())
        }),
        label(move || value_owned.clone()).style(move |s| {
            let colors = theme.get().colors();
            s.font_size(11.0)
                .color(colors.text_secondary)
                .margin_left(6.0)
        }),
    ))
    .style(|s| s.align_items(AlignItems::Center))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_emoji() {
        assert_eq!(ConnectionState::Connected.emoji(), "🟢");
        assert_eq!(ConnectionState::Disconnected.emoji(), "🔴");
        assert_eq!(ConnectionState::Connecting.emoji(), "🟡");
        assert_eq!(ConnectionState::NotConfigured.emoji(), "⚪");
    }

    #[test]
    fn test_connection_state_status_text() {
        assert_eq!(ConnectionState::Connected.status_text(), "Connected");
        assert_eq!(ConnectionState::Disconnected.status_text(), "Disconnected");
        assert_eq!(ConnectionState::Connecting.status_text(), "Connecting...");
        assert_eq!(ConnectionState::NotConfigured.status_text(), "Not Configured");
    }

    #[test]
    fn test_connection_state_action_text() {
        assert_eq!(ConnectionState::Connected.action_text(), "Disconnect");
        assert_eq!(ConnectionState::Disconnected.action_text(), "Connect");
        assert_eq!(ConnectionState::Connecting.action_text(), "...");
        assert_eq!(ConnectionState::NotConfigured.action_text(), "Setup");
    }

    #[test]
    fn test_agent_config_builder() {
        let config = AgentConfig::new("TestAgent", "Test Description")
            .with_tools(10)
            .with_state(ConnectionState::Connected);

        assert_eq!(config.name, "TestAgent");
        assert_eq!(config.description, "Test Description");
        assert_eq!(config.tools_count, 10);
        assert_eq!(config.connection_state, ConnectionState::Connected);
    }

    #[test]
    fn test_agent_registry_default() {
        let registry = AgentRegistry::new();
        let agents = registry.agents.get();

        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0].name, "Claude");
        assert_eq!(agents[1].name, "Gemini");
        assert_eq!(agents[2].name, "Custom");
    }

    #[test]
    fn test_agent_registry_get_agent() {
        let registry = AgentRegistry::new();

        let claude = registry.get_agent("Claude");
        assert!(claude.is_some());
        assert_eq!(claude.unwrap().name, "Claude");

        let nonexistent = registry.get_agent("NonExistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_agent_registry_update_agent() {
        let registry = AgentRegistry::new();

        registry.update_agent("Claude", |agent| {
            agent.tools_count = 20;
            agent.connection_state = ConnectionState::Disconnected;
        });

        let claude = registry.get_agent("Claude").unwrap();
        assert_eq!(claude.tools_count, 20);
        assert_eq!(claude.connection_state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_agent_dashboard_state_new() {
        let state = AgentDashboardState::new();

        assert!(!state.visible.get());
        assert_eq!(state.selected_agent.get(), Some("Claude".to_string()));

        let session = state.session_info.get();
        assert_eq!(session.agent_name, "Claude Code");
        assert_eq!(session.tools_count, 15);
        assert_eq!(session.pending_commands, 8);
        assert_eq!(session.executed_today, 23);
    }

    #[test]
    fn test_agent_dashboard_toggle_visibility() {
        let state = AgentDashboardState::new();
        assert!(!state.visible.get());

        state.toggle_visibility();
        assert!(state.visible.get());

        state.toggle_visibility();
        assert!(!state.visible.get());
    }

    #[test]
    fn test_agent_dashboard_select_agent() {
        let state = AgentDashboardState::new();

        state.select_agent("Gemini".to_string());
        assert_eq!(state.selected_agent.get(), Some("Gemini".to_string()));

        state.select_agent("Custom".to_string());
        assert_eq!(state.selected_agent.get(), Some("Custom".to_string()));
    }

    #[test]
    fn test_session_info_default() {
        let info = SessionInfo::default();

        assert_eq!(info.agent_name, "None");
        assert_eq!(info.tools_count, 0);
        assert_eq!(info.pending_commands, 0);
        assert_eq!(info.executed_today, 0);
    }
}
