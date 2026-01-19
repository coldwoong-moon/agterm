//! Command Palette for AgTerm terminal emulator
//!
//! Provides a searchable command palette (Cmd+Shift+P) for quick access to terminal features.

use iced::widget::text_input::Id as TextInputId;
use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Border, Color, Element, Length};

/// Get the palette input ID
pub fn palette_input_id() -> TextInputId {
    TextInputId::new("palette_input")
}

/// A single command palette item
#[derive(Debug, Clone)]
pub struct PaletteItem {
    /// Unique command identifier (e.g., "new_tab", "close_tab")
    pub id: String,
    /// Display label (e.g., "New Tab")
    pub label: String,
    /// Optional keyboard shortcut (e.g., "Cmd+T")
    pub shortcut: Option<String>,
    /// Command category (e.g., "Tabs", "Panes", "View")
    pub category: String,
}

impl PaletteItem {
    /// Create a new palette item
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            category: category.into(),
        }
    }

    /// Set the keyboard shortcut
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Check if the item matches a search query
    fn matches(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        let query_lower = query.to_lowercase();
        let label_lower = self.label.to_lowercase();
        let category_lower = self.category.to_lowercase();
        let id_lower = self.id.to_lowercase();

        // Match against label, category, or ID
        label_lower.contains(&query_lower)
            || category_lower.contains(&query_lower)
            || id_lower.contains(&query_lower)
    }
}

/// Messages for the command palette
#[derive(Debug, Clone)]
pub enum PaletteMessage {
    /// Open the command palette
    Open,
    /// Close the command palette
    Close,
    /// Search input changed
    InputChanged(String),
    /// Select an item by filtered index
    SelectItem(usize),
    /// Execute the selected command
    Execute,
    /// Move selection up
    Up,
    /// Move selection down
    Down,
}

/// Command palette state
pub struct CommandPalette {
    /// Whether the palette is visible
    visible: bool,
    /// Current search input
    input: String,
    /// All available commands
    items: Vec<PaletteItem>,
    /// Indices of filtered items
    filtered_items: Vec<usize>,
    /// Index in filtered_items (not items!)
    selected_index: usize,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    /// Create a new empty command palette
    pub fn new() -> Self {
        Self {
            visible: false,
            input: String::new(),
            items: Vec::new(),
            filtered_items: Vec::new(),
            selected_index: 0,
        }
    }

    /// Create a command palette with default AgTerm commands
    pub fn with_default_commands() -> Self {
        let items = vec![
            // Tab management
            PaletteItem::new("new_tab", "New Tab", "Tabs").with_shortcut("Cmd+T"),
            PaletteItem::new("close_tab", "Close Tab", "Tabs").with_shortcut("Cmd+W"),
            PaletteItem::new("duplicate_tab", "Duplicate Tab", "Tabs").with_shortcut("Cmd+Shift+T"),
            PaletteItem::new("next_tab", "Next Tab", "Tabs").with_shortcut("Cmd+Shift+]"),
            PaletteItem::new("prev_tab", "Previous Tab", "Tabs").with_shortcut("Cmd+Shift+["),
            // Pane management
            PaletteItem::new("split_horizontal", "Split Horizontally", "Panes")
                .with_shortcut("Cmd+D"),
            PaletteItem::new("split_vertical", "Split Vertically", "Panes")
                .with_shortcut("Cmd+Shift+D"),
            PaletteItem::new("close_pane", "Close Pane", "Panes").with_shortcut("Cmd+Shift+W"),
            PaletteItem::new("next_pane", "Next Pane", "Panes").with_shortcut("Cmd+]"),
            PaletteItem::new("prev_pane", "Previous Pane", "Panes").with_shortcut("Cmd+["),
            PaletteItem::new("zoom_pane", "Zoom/Unzoom Pane", "Panes")
                .with_shortcut("Cmd+Shift+Enter"),
            // View
            PaletteItem::new("toggle_debug", "Toggle Debug Panel", "View")
                .with_shortcut("Cmd+Shift+I"),
            PaletteItem::new("clear_screen", "Clear Screen", "View").with_shortcut("Cmd+K"),
            PaletteItem::new("scroll_to_top", "Scroll to Top", "View"),
            PaletteItem::new("scroll_to_bottom", "Scroll to Bottom", "View"),
            // Font
            PaletteItem::new("increase_font", "Increase Font Size", "Font")
                .with_shortcut("Cmd++"),
            PaletteItem::new("decrease_font", "Decrease Font Size", "Font")
                .with_shortcut("Cmd+-"),
            PaletteItem::new("reset_font", "Reset Font Size", "Font").with_shortcut("Cmd+0"),
            // Theme
            PaletteItem::new("theme_warp", "Switch to Warp Theme", "Theme"),
            PaletteItem::new("theme_dracula", "Switch to Dracula Theme", "Theme"),
            PaletteItem::new("theme_nord", "Switch to Nord Theme", "Theme"),
            PaletteItem::new("theme_solarized", "Switch to Solarized Theme", "Theme"),
            // Clipboard
            PaletteItem::new("copy", "Copy Selection", "Clipboard").with_shortcut("Cmd+C"),
            PaletteItem::new("paste", "Paste", "Clipboard").with_shortcut("Cmd+V"),
        ];

        let mut palette = Self::new();
        palette.items = items;
        palette.filter_items();
        palette
    }

    /// Get the number of filtered items
    pub fn filtered_count(&self) -> usize {
        self.filtered_items.len()
    }

    /// Check if the palette is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the currently selected command ID (if any)
    pub fn selected_command_id(&self) -> Option<&str> {
        if self.filtered_items.is_empty() {
            return None;
        }
        self.filtered_items
            .get(self.selected_index)
            .and_then(|&idx| self.items.get(idx))
            .map(|item| item.id.as_str())
    }

    /// Update the command palette state
    ///
    /// Returns the command ID to execute (if any)
    pub fn update(&mut self, message: PaletteMessage) -> Option<String> {
        match message {
            PaletteMessage::Open => {
                self.visible = true;
                self.input.clear();
                self.filter_items();
                None
            }
            PaletteMessage::Close => {
                self.visible = false;
                self.input.clear();
                self.selected_index = 0;
                None
            }
            PaletteMessage::InputChanged(new_input) => {
                self.input = new_input;
                self.filter_items();
                self.selected_index = 0; // Reset selection on input change
                None
            }
            PaletteMessage::SelectItem(index) => {
                if index < self.filtered_items.len() {
                    self.selected_index = index;
                }
                None
            }
            PaletteMessage::Execute => {
                let command_id = self.selected_command_id().map(|s| s.to_string());
                if command_id.is_some() {
                    self.visible = false;
                    self.input.clear();
                    self.selected_index = 0;
                }
                command_id
            }
            PaletteMessage::Up => {
                if !self.filtered_items.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.filtered_items.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
                None
            }
            PaletteMessage::Down => {
                if !self.filtered_items.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.filtered_items.len();
                }
                None
            }
        }
    }

    /// Render the command palette
    pub fn view<'a>(&'a self) -> Element<'a, PaletteMessage> {
        if !self.visible {
            // Return minimal invisible element (1px to avoid zero-height panic)
            return Space::new(Length::Fill, Length::Fixed(1.0)).into();
        }

        // Background overlay (semi-transparent dark)
        let bg_overlay = Color::from_rgba(0.0, 0.0, 0.0, 0.5);
        let palette_bg = Color::from_rgb(0.12, 0.12, 0.15); // BG_SECONDARY
        let text_primary = Color::from_rgb(0.93, 0.93, 0.95);
        let text_secondary = Color::from_rgb(0.6, 0.62, 0.68);
        let border_color = Color::from_rgb(0.22, 0.22, 0.28);
        let selected_bg = Color::from_rgb(0.18, 0.18, 0.22); // BG_BLOCK_HOVER
        let accent_blue = Color::from_rgb(0.36, 0.54, 0.98);

        // Build the search input
        let search_input = text_input("Search commands...", &self.input)
            .id(palette_input_id())
            .on_input(PaletteMessage::InputChanged)
            .on_submit(PaletteMessage::Execute)
            .padding(12)
            .size(16)
            .style(move |_theme, status| text_input::Style {
                background: palette_bg.into(),
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

        // Build the command list
        let mut items_column = column![].spacing(2);

        if self.filtered_items.is_empty() {
            // No results
            items_column = items_column.push(
                container(text("No commands found").size(14).color(text_secondary))
                    .padding(20)
                    .width(Length::Fill)
                    .center_x(Length::Fill),
            );
        } else {
            for (filtered_idx, &item_idx) in self.filtered_items.iter().enumerate() {
                if let Some(item) = self.items.get(item_idx) {
                    let is_selected = filtered_idx == self.selected_index;

                    let label_text = text(&item.label).size(14).color(text_primary);

                    let mut row_content = row![
                        container(text(&item.category).size(12).color(text_secondary))
                            .padding([2, 8])
                            .width(Length::Fixed(80.0)),
                        label_text,
                        Space::with_width(Length::Fill),
                    ]
                    .align_y(iced::Alignment::Center)
                    .spacing(12);

                    if let Some(shortcut) = &item.shortcut {
                        row_content = row_content.push(
                            text(shortcut).size(12).color(text_secondary),
                        );
                    }

                    let item_container = container(row_content)
                        .padding([8, 12])
                        .width(Length::Fill)
                        .style(move |_theme| container::Style {
                            background: if is_selected {
                                Some(selected_bg.into())
                            } else {
                                Some(palette_bg.into())
                            },
                            border: Border {
                                color: if is_selected {
                                    accent_blue
                                } else {
                                    Color::TRANSPARENT
                                },
                                width: if is_selected { 1.0 } else { 0.0 },
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        });

                    items_column = items_column.push(item_container);
                }
            }
        }

        let scrollable_list = scrollable(items_column)
            .height(Length::Fixed(400.0))
            .style(move |_theme, _status| scrollable::Style {
                container: container::Style {
                    background: Some(palette_bg.into()),
                    ..Default::default()
                },
                vertical_rail: scrollable::Rail {
                    background: Some(palette_bg.into()),
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
                    background: Some(palette_bg.into()),
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

        let palette_content = column![search_input, scrollable_list]
            .spacing(8)
            .width(Length::Fixed(600.0));

        let palette_container = container(palette_content)
            .padding(16)
            .style(move |_theme| container::Style {
                background: Some(palette_bg.into()),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                },
                ..Default::default()
            });

        // Center the palette with overlay
        container(
            container(palette_container)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .padding([100.0, 0.0]),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(bg_overlay.into()),
            ..Default::default()
        })
        .into()
    }

    /// Filter items based on current input
    fn filter_items(&mut self) {
        self.filtered_items = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.matches(&self.input))
            .map(|(idx, _)| idx)
            .collect();

        // Ensure selected_index is valid
        if self.selected_index >= self.filtered_items.len() && !self.filtered_items.is_empty() {
            self.selected_index = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_item_matches() {
        let item = PaletteItem::new("new_tab", "New Tab", "Tabs").with_shortcut("Cmd+T");

        assert!(item.matches(""));
        assert!(item.matches("new"));
        assert!(item.matches("tab"));
        assert!(item.matches("New Tab"));
        assert!(item.matches("tabs"));
        assert!(item.matches("NEW"));
        assert!(!item.matches("close"));
        assert!(!item.matches("xyz"));
    }

    #[test]
    fn test_palette_filtering() {
        let mut palette = CommandPalette::with_default_commands();

        // Initially all items should be visible
        let initial_count = palette.items.len();
        assert_eq!(palette.filtered_count(), initial_count);

        // Filter by "tab"
        palette.update(PaletteMessage::InputChanged("tab".to_string()));
        assert!(palette.filtered_count() < initial_count);
        assert!(palette.filtered_count() > 0);

        // Filter by non-existent term
        palette.update(PaletteMessage::InputChanged("nonexistent".to_string()));
        assert_eq!(palette.filtered_count(), 0);

        // Clear filter
        palette.update(PaletteMessage::InputChanged("".to_string()));
        assert_eq!(palette.filtered_count(), initial_count);
    }

    #[test]
    fn test_palette_navigation() {
        let mut palette = CommandPalette::with_default_commands();
        palette.visible = true;

        assert_eq!(palette.selected_index, 0);

        // Move down
        palette.update(PaletteMessage::Down);
        assert_eq!(palette.selected_index, 1);

        // Move up
        palette.update(PaletteMessage::Up);
        assert_eq!(palette.selected_index, 0);

        // Move up from first (should wrap to last)
        palette.update(PaletteMessage::Up);
        assert_eq!(palette.selected_index, palette.filtered_count() - 1);

        // Move down from last (should wrap to first)
        palette.update(PaletteMessage::Down);
        assert_eq!(palette.selected_index, 0);
    }

    #[test]
    fn test_palette_execute() {
        let mut palette = CommandPalette::with_default_commands();
        palette.visible = true;

        // Should return the first command ID
        let command = palette.update(PaletteMessage::Execute);
        assert!(command.is_some());
        assert_eq!(command.unwrap(), "new_tab");
        assert!(!palette.visible); // Should close after execute
    }

    #[test]
    fn test_palette_open_close() {
        let mut palette = CommandPalette::new();

        assert!(!palette.visible);

        palette.update(PaletteMessage::Open);
        assert!(palette.visible);

        palette.update(PaletteMessage::Close);
        assert!(!palette.visible);
    }

    #[test]
    fn test_palette_selected_command_id() {
        let mut palette = CommandPalette::with_default_commands();

        assert_eq!(palette.selected_command_id(), Some("new_tab"));

        palette.update(PaletteMessage::Down);
        assert_eq!(palette.selected_command_id(), Some("close_tab"));

        palette.update(PaletteMessage::InputChanged("debug".to_string()));
        assert_eq!(palette.selected_command_id(), Some("toggle_debug"));
    }

    #[test]
    fn test_palette_empty_filtered_items() {
        let mut palette = CommandPalette::with_default_commands();
        palette.update(PaletteMessage::InputChanged("xyz123nonexistent".to_string()));

        assert_eq!(palette.filtered_count(), 0);
        assert_eq!(palette.selected_command_id(), None);
        assert_eq!(palette.update(PaletteMessage::Execute), None);
    }
}
