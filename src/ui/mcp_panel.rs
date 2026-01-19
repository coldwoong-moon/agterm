//! MCP AI Assistant Panel for AgTerm terminal emulator
//!
//! Provides a side panel for interacting with MCP (Model Context Protocol) servers.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Border, Color, Element, Length};

/// MCP Server ID type (placeholder for future MCP integration)
pub type McpServerId = String;

/// MCP Response structure
#[derive(Debug, Clone)]
pub struct McpResponse {
    /// Response text from the MCP server
    pub text: String,
    /// Optional command that can be executed
    pub command: Option<String>,
    /// Timestamp of the response
    pub timestamp: std::time::SystemTime,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server name
    pub name: String,
    /// Server endpoint URL
    pub endpoint: String,
}

/// Connection status of an MCP server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Successfully connected
    Connected,
    /// Disconnected
    Disconnected,
    /// Connection error
    Error,
}

impl ConnectionStatus {
    /// Get the status indicator symbol
    pub fn symbol(&self) -> &str {
        match self {
            ConnectionStatus::Connected => "●",
            ConnectionStatus::Disconnected => "○",
            ConnectionStatus::Error => "×",
        }
    }

    /// Get the status color
    pub fn color(&self) -> Color {
        match self {
            ConnectionStatus::Connected => Color::from_rgb(0.4, 0.8, 0.4), // Green
            ConnectionStatus::Disconnected => Color::from_rgb(0.5, 0.5, 0.5), // Gray
            ConnectionStatus::Error => Color::from_rgb(0.9, 0.3, 0.3), // Red
        }
    }
}

/// Messages for the MCP panel
#[derive(Debug, Clone)]
pub enum McpPanelMessage {
    /// Input text changed
    InputChanged(String),
    /// Submit the current input to the active server
    Submit,
    /// Select a specific MCP server
    SelectServer(McpServerId),
    /// Toggle panel visibility
    TogglePanel,
    /// Execute a command from a response
    ExecuteCommand(String),
    /// Copy a response to clipboard
    CopyResponse(usize),
    /// Clear response history
    ClearHistory,
}

/// MCP Panel state
pub struct McpPanel {
    /// Whether the panel is visible
    visible: bool,
    /// Current input text
    input: String,
    /// Response history (server_id, response)
    responses: Vec<(McpServerId, McpResponse)>,
    /// Currently active server
    active_server: Option<McpServerId>,
    /// Available MCP servers
    servers: Vec<(McpServerId, ServerConfig, ConnectionStatus)>,
    /// Whether a request is in progress
    loading: bool,
}

impl Default for McpPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl McpPanel {
    /// Create a new MCP panel
    pub fn new() -> Self {
        Self {
            visible: false,
            input: String::new(),
            responses: Vec::new(),
            active_server: None,
            servers: Vec::new(),
            loading: false,
        }
    }

    /// Create a panel with example servers (for testing)
    pub fn with_example_servers() -> Self {
        let servers = vec![
            (
                "gpt4".to_string(),
                ServerConfig {
                    name: "GPT-4".to_string(),
                    endpoint: "https://api.openai.com".to_string(),
                },
                ConnectionStatus::Connected,
            ),
            (
                "claude".to_string(),
                ServerConfig {
                    name: "Claude".to_string(),
                    endpoint: "https://api.anthropic.com".to_string(),
                },
                ConnectionStatus::Disconnected,
            ),
            (
                "local".to_string(),
                ServerConfig {
                    name: "Local Server".to_string(),
                    endpoint: "http://localhost:8080".to_string(),
                },
                ConnectionStatus::Error,
            ),
        ];

        let active_server = Some("gpt4".to_string());

        Self {
            visible: false,
            input: String::new(),
            responses: Vec::new(),
            active_server,
            servers,
            loading: false,
        }
    }

    /// Check if the panel is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Add a server to the panel
    pub fn add_server(
        &mut self,
        id: McpServerId,
        config: ServerConfig,
        status: ConnectionStatus,
    ) {
        self.servers.push((id.clone(), config, status));
        if self.active_server.is_none() {
            self.active_server = Some(id);
        }
    }

    /// Update server connection status
    pub fn update_server_status(&mut self, id: &McpServerId, status: ConnectionStatus) {
        if let Some((_, _, current_status)) = self.servers.iter_mut().find(|(sid, _, _)| sid == id)
        {
            *current_status = status;
        }
    }

    /// Add a response to the history
    pub fn add_response(&mut self, server_id: McpServerId, response: McpResponse) {
        self.responses.push((server_id, response));
        self.loading = false;
    }

    /// Update the MCP panel state
    ///
    /// Returns an optional command to execute (if any)
    pub fn update(&mut self, message: McpPanelMessage) -> Option<String> {
        match message {
            McpPanelMessage::InputChanged(new_input) => {
                self.input = new_input;
                None
            }
            McpPanelMessage::Submit => {
                if !self.input.trim().is_empty() && self.active_server.is_some() {
                    // TODO: Send request to MCP server
                    // For now, just clear input and set loading
                    self.loading = true;
                    self.input.clear();

                    // Simulate response (remove this when implementing actual MCP integration)
                    let response = McpResponse {
                        text: "This is a placeholder response. Implement MCP integration here."
                            .to_string(),
                        command: None,
                        timestamp: std::time::SystemTime::now(),
                    };
                    if let Some(server_id) = &self.active_server {
                        self.add_response(server_id.clone(), response);
                    }
                }
                None
            }
            McpPanelMessage::SelectServer(server_id) => {
                self.active_server = Some(server_id);
                None
            }
            McpPanelMessage::TogglePanel => {
                self.visible = !self.visible;
                None
            }
            McpPanelMessage::ExecuteCommand(command) => Some(command),
            McpPanelMessage::CopyResponse(index) => {
                // TODO: Implement clipboard copy
                if let Some((_, response)) = self.responses.get(index) {
                    // Return command to copy to clipboard
                    // This should be handled by the parent application
                    eprintln!("Copy response: {}", response.text);
                }
                None
            }
            McpPanelMessage::ClearHistory => {
                self.responses.clear();
                None
            }
        }
    }

    /// Render the MCP panel
    pub fn view(&self) -> Element<McpPanelMessage> {
        if !self.visible {
            return Space::new(Length::Fixed(1.0), Length::Fill).into();
        }

        // Color palette (matching AgTerm dark theme)
        let bg_primary = Color::from_rgb(0.12, 0.12, 0.15); // #1e1e26
        let bg_secondary = Color::from_rgb(0.15, 0.15, 0.18); // Slightly lighter
        let text_primary = Color::from_rgb(0.93, 0.93, 0.95); // #edeff2
        let text_secondary = Color::from_rgb(0.6, 0.62, 0.68);
        let border_color = Color::from_rgb(0.22, 0.22, 0.28);
        let hover_bg = Color::from_rgb(0.18, 0.18, 0.22);
        let accent_blue = Color::from_rgb(0.36, 0.54, 0.98);

        // Header with title and close button
        let header = container(
            row![
                text("MCP Assistant")
                    .size(16)
                    .color(text_primary)
                    .width(Length::Fill),
                button(text("×").size(20).color(text_primary))
                    .on_press(McpPanelMessage::TogglePanel)
                    .padding([2, 8])
                    .style(move |_theme, status| {
                        let background = if matches!(status, button::Status::Hovered) {
                            Some(hover_bg.into())
                        } else {
                            None
                        };
                        button::Style {
                            background,
                            text_color: text_primary,
                            border: Border {
                                color: Color::TRANSPARENT,
                                width: 0.0,
                                radius: 4.0.into(),
                            },
                            shadow: iced::Shadow::default(),
                        }
                    }),
            ]
            .align_y(iced::Alignment::Center)
            .spacing(8),
        )
        .padding([12, 16])
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(bg_secondary.into()),
            border: Border {
                color: border_color,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        // Server selector
        let mut server_buttons = row![].spacing(8);
        for (server_id, config, status) in &self.servers {
            let is_active = self
                .active_server
                .as_ref()
                .map(|id| id == server_id)
                .unwrap_or(false);

            let server_btn = button(
                row![
                    text(status.symbol()).size(12).color(status.color()),
                    text(&config.name).size(12).color(text_primary),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
            .on_press(McpPanelMessage::SelectServer(server_id.clone()))
            .padding([6, 12])
            .style(move |_theme, btn_status| {
                let background = if is_active {
                    Some(accent_blue.into())
                } else if matches!(btn_status, button::Status::Hovered) {
                    Some(hover_bg.into())
                } else {
                    Some(bg_secondary.into())
                };
                button::Style {
                    background,
                    text_color: text_primary,
                    border: Border {
                        color: if is_active {
                            accent_blue
                        } else {
                            border_color
                        },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    shadow: iced::Shadow::default(),
                }
            });

            server_buttons = server_buttons.push(server_btn);
        }

        let server_selector = container(server_buttons)
            .padding([8, 16])
            .width(Length::Fill);

        // Response history
        let mut history_column = column![].spacing(12);

        if self.responses.is_empty() && !self.loading {
            history_column = history_column.push(
                container(
                    text("No responses yet. Start a conversation!")
                        .size(14)
                        .color(text_secondary),
                )
                .padding(20)
                .width(Length::Fill)
                .center_x(Length::Fill),
            );
        } else {
            for (idx, (server_id, response)) in self.responses.iter().enumerate() {
                // Find server name
                let server_name = self
                    .servers
                    .iter()
                    .find(|(id, _, _)| id == server_id)
                    .map(|(_, config, _)| config.name.as_str())
                    .unwrap_or("Unknown");

                let mut response_content = column![
                    text(server_name).size(12).color(text_secondary),
                    text(&response.text).size(14).color(text_primary),
                ]
                .spacing(4);

                // Add execute button if command is present
                if let Some(command) = &response.command {
                    let cmd = command.clone();
                    let execute_btn = button(text("Execute").size(12).color(text_primary))
                        .on_press(McpPanelMessage::ExecuteCommand(cmd))
                        .padding([4, 8])
                        .style(move |_theme, status| {
                            let background = if matches!(status, button::Status::Hovered) {
                                Some(accent_blue.into())
                            } else {
                                Some(bg_secondary.into())
                            };
                            button::Style {
                                background,
                                text_color: text_primary,
                                border: Border {
                                    color: accent_blue,
                                    width: 1.0,
                                    radius: 4.0.into(),
                                },
                                shadow: iced::Shadow::default(),
                            }
                        });

                    response_content = response_content.push(
                        row![
                            execute_btn,
                            button(text("Copy").size(12).color(text_primary))
                                .on_press(McpPanelMessage::CopyResponse(idx))
                                .padding([4, 8])
                                .style(move |_theme, status| {
                                    let background = if matches!(status, button::Status::Hovered)
                                    {
                                        Some(hover_bg.into())
                                    } else {
                                        None
                                    };
                                    button::Style {
                                        background,
                                        text_color: text_primary,
                                        border: Border {
                                            color: border_color,
                                            width: 1.0,
                                            radius: 4.0.into(),
                                        },
                                        shadow: iced::Shadow::default(),
                                    }
                                }),
                        ]
                        .spacing(8),
                    );
                } else {
                    // Just copy button
                    response_content = response_content.push(
                        button(text("Copy").size(12).color(text_primary))
                            .on_press(McpPanelMessage::CopyResponse(idx))
                            .padding([4, 8])
                            .style(move |_theme, status| {
                                let background = if matches!(status, button::Status::Hovered) {
                                    Some(hover_bg.into())
                                } else {
                                    None
                                };
                                button::Style {
                                    background,
                                    text_color: text_primary,
                                    border: Border {
                                        color: border_color,
                                        width: 1.0,
                                        radius: 4.0.into(),
                                    },
                                    shadow: iced::Shadow::default(),
                                }
                            }),
                    );
                }

                let response_container = container(response_content)
                    .padding(12)
                    .width(Length::Fill)
                    .style(move |_theme| container::Style {
                        background: Some(bg_secondary.into()),
                        border: Border {
                            color: border_color,
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    });

                history_column = history_column.push(response_container);
            }
        }

        if self.loading {
            history_column = history_column.push(
                container(
                    text("Loading...")
                        .size(14)
                        .color(text_secondary),
                )
                .padding(12)
                .width(Length::Fill)
                .center_x(Length::Fill),
            );
        }

        let history_scrollable = scrollable(history_column.padding([16, 16]))
            .height(Length::Fill)
            .style(move |_theme, _status| scrollable::Style {
                container: container::Style {
                    background: Some(bg_primary.into()),
                    ..Default::default()
                },
                vertical_rail: scrollable::Rail {
                    background: Some(bg_primary.into()),
                    border: Border::default(),
                    scroller: scrollable::Scroller {
                        color: border_color,
                        border: Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                    },
                },
                horizontal_rail: scrollable::Rail {
                    background: Some(bg_primary.into()),
                    border: Border::default(),
                    scroller: scrollable::Scroller {
                        color: border_color,
                        border: Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                    },
                },
                gap: None,
            });

        // Input section
        let input_field = text_input("Ask the AI assistant...", &self.input)
            .on_input(McpPanelMessage::InputChanged)
            .on_submit(McpPanelMessage::Submit)
            .padding(12)
            .size(14)
            .style(move |_theme, status| text_input::Style {
                background: bg_secondary.into(),
                border: Border {
                    color: if matches!(status, text_input::Status::Focused) {
                        accent_blue
                    } else {
                        border_color
                    },
                    width: 1.0,
                    radius: 4.0.into(),
                },
                icon: text_primary,
                placeholder: text_secondary,
                value: text_primary,
                selection: accent_blue,
            });

        let send_btn = button(text("Send").size(14).color(text_primary))
            .on_press(McpPanelMessage::Submit)
            .padding([12, 20])
            .style(move |_theme, status| {
                let background = if matches!(status, button::Status::Hovered) {
                    Some(Color::from_rgb(0.4, 0.58, 1.0).into())
                } else {
                    Some(accent_blue.into())
                };
                button::Style {
                    background,
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                    shadow: iced::Shadow::default(),
                }
            });

        let input_row = row![input_field, send_btn]
            .spacing(8)
            .width(Length::Fill)
            .align_y(iced::Alignment::Center);

        let input_section = container(input_row)
            .padding([12, 16])
            .width(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(bg_secondary.into()),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            });

        // Combine all sections
        let panel_content = column![header, server_selector, history_scrollable, input_section]
            .width(Length::Fixed(400.0))
            .height(Length::Fill);

        container(panel_content)
            .width(Length::Fixed(400.0))
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(bg_primary.into()),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_panel_new() {
        let panel = McpPanel::new();
        assert!(!panel.visible);
        assert!(panel.input.is_empty());
        assert!(panel.responses.is_empty());
        assert!(panel.active_server.is_none());
        assert!(panel.servers.is_empty());
        assert!(!panel.loading);
    }

    #[test]
    fn test_mcp_panel_with_example_servers() {
        let panel = McpPanel::with_example_servers();
        assert!(!panel.visible);
        assert_eq!(panel.servers.len(), 3);
        assert_eq!(panel.active_server, Some("gpt4".to_string()));
    }

    #[test]
    fn test_mcp_panel_add_server() {
        let mut panel = McpPanel::new();
        panel.add_server(
            "test".to_string(),
            ServerConfig {
                name: "Test Server".to_string(),
                endpoint: "http://test.com".to_string(),
            },
            ConnectionStatus::Connected,
        );

        assert_eq!(panel.servers.len(), 1);
        assert_eq!(panel.active_server, Some("test".to_string()));
    }

    #[test]
    fn test_mcp_panel_update_server_status() {
        let mut panel = McpPanel::with_example_servers();
        panel.update_server_status(&"gpt4".to_string(), ConnectionStatus::Error);

        let status = panel
            .servers
            .iter()
            .find(|(id, _, _)| id == "gpt4")
            .map(|(_, _, status)| *status);
        assert_eq!(status, Some(ConnectionStatus::Error));
    }

    #[test]
    fn test_mcp_panel_toggle() {
        let mut panel = McpPanel::new();
        assert!(!panel.is_visible());

        panel.update(McpPanelMessage::TogglePanel);
        assert!(panel.is_visible());

        panel.update(McpPanelMessage::TogglePanel);
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_mcp_panel_input_changed() {
        let mut panel = McpPanel::new();
        panel.update(McpPanelMessage::InputChanged("test input".to_string()));
        assert_eq!(panel.input, "test input");
    }

    #[test]
    fn test_mcp_panel_select_server() {
        let mut panel = McpPanel::with_example_servers();
        panel.update(McpPanelMessage::SelectServer("claude".to_string()));
        assert_eq!(panel.active_server, Some("claude".to_string()));
    }

    #[test]
    fn test_connection_status_symbol() {
        assert_eq!(ConnectionStatus::Connected.symbol(), "●");
        assert_eq!(ConnectionStatus::Disconnected.symbol(), "○");
        assert_eq!(ConnectionStatus::Error.symbol(), "×");
    }

    #[test]
    fn test_mcp_panel_add_response() {
        let mut panel = McpPanel::with_example_servers();
        let response = McpResponse {
            text: "Test response".to_string(),
            command: None,
            timestamp: std::time::SystemTime::now(),
        };

        panel.add_response("gpt4".to_string(), response);
        assert_eq!(panel.responses.len(), 1);
        assert!(!panel.loading);
    }

    #[test]
    fn test_mcp_panel_clear_history() {
        let mut panel = McpPanel::with_example_servers();
        let response = McpResponse {
            text: "Test response".to_string(),
            command: None,
            timestamp: std::time::SystemTime::now(),
        };

        panel.add_response("gpt4".to_string(), response);
        assert_eq!(panel.responses.len(), 1);

        panel.update(McpPanelMessage::ClearHistory);
        assert_eq!(panel.responses.len(), 0);
    }
}
