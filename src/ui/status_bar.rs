//! Status bar component for AgTerm terminal emulator
//!
//! Displays terminal information like shell, working directory, dimensions, encoding, etc.

use iced::widget::{container, horizontal_space, row, text};
use iced::{Color, Element, Length};

/// Information to display in the status bar
#[derive(Debug, Clone)]
pub struct StatusBarInfo {
    /// Current shell name (e.g., "zsh", "bash")
    pub shell: String,
    /// Current working directory
    pub cwd: Option<String>,
    /// Terminal columns
    pub cols: u16,
    /// Terminal rows
    pub rows: u16,
    /// Character encoding (e.g., "UTF-8")
    pub encoding: String,
    /// Current mode (e.g., "raw", "normal", "streaming")
    pub mode: Option<String>,
    /// Scroll position (current line, total lines)
    pub scroll_position: Option<(usize, usize)>,
}

impl Default for StatusBarInfo {
    fn default() -> Self {
        Self {
            shell: String::from("shell"),
            cwd: None,
            cols: 80,
            rows: 24,
            encoding: String::from("UTF-8"),
            mode: None,
            scroll_position: None,
        }
    }
}

/// Status bar configuration
#[derive(Debug, Clone)]
pub struct StatusBarConfig {
    /// Whether to show the status bar
    pub visible: bool,
    /// Whether to show current working directory
    pub show_cwd: bool,
    /// Whether to show terminal size
    pub show_size: bool,
    /// Whether to show encoding
    pub show_encoding: bool,
    /// Whether to show scroll position
    pub show_scroll_position: bool,
    /// Whether to show mode indicator
    pub show_mode: bool,
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            visible: true,
            show_cwd: true,
            show_size: true,
            show_encoding: true,
            show_scroll_position: true,
            show_mode: true,
        }
    }
}

/// Render the status bar
///
/// # Arguments
/// * `info` - Status bar information
/// * `config` - Status bar configuration
/// * `text_color` - Text color for status bar content
/// * `bg_color` - Background color for status bar
///
/// # Type Parameters
/// * `Message` - The message type for the application
pub fn view<'a, Message: 'a>(
    info: StatusBarInfo,
    config: StatusBarConfig,
    text_color: Color,
    bg_color: Color,
) -> Element<'a, Message> {
    if !config.visible {
        return container(text("")).height(Length::Shrink).into();
    }

    // Left section: shell and cwd
    let mut left_parts = vec![];

    left_parts.push(
        text(info.shell.clone())
            .size(12)
            .color(text_color)
            .into(),
    );

    if config.show_cwd {
        if let Some(cwd) = info.cwd.clone() {
            left_parts.push(text(" | ").size(12).color(text_color).into());
            left_parts.push(
                text(cwd)
                    .size(12)
                    .color(text_color)
                    .into(),
            );
        }
    }

    if config.show_mode {
        if let Some(mode) = info.mode.clone() {
            left_parts.push(text(" | ").size(12).color(text_color).into());
            left_parts.push(
                text(mode.to_uppercase())
                    .size(12)
                    .color(text_color)
                    .into(),
            );
        }
    }

    let left = row(left_parts).spacing(0);

    // Right section: scroll position, size, and encoding
    let mut right_parts = vec![];

    if config.show_scroll_position {
        if let Some((current, total)) = info.scroll_position {
            right_parts.push(
                text(format!("{current}/{total}"))
                    .size(12)
                    .color(text_color)
                    .into(),
            );
            right_parts.push(text(" | ").size(12).color(text_color).into());
        }
    }

    if config.show_size {
        right_parts.push(
            text(format!("{}x{}", info.cols, info.rows))
                .size(12)
                .color(text_color)
                .into(),
        );
    }

    if config.show_encoding {
        if config.show_size {
            right_parts.push(text(" | ").size(12).color(text_color).into());
        }
        right_parts.push(
            text(info.encoding.clone())
                .size(12)
                .color(text_color)
                .into(),
        );
    }

    let right = row(right_parts).spacing(0);

    // Combine left and right with spacing
    container(
        row![left, horizontal_space(), right]
            .spacing(8)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([4, 12])
    .style(move |_theme| container::Style {
        background: Some(bg_color.into()),
        ..Default::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_info_default() {
        let info = StatusBarInfo::default();
        assert_eq!(info.shell, "shell");
        assert_eq!(info.cols, 80);
        assert_eq!(info.rows, 24);
        assert_eq!(info.encoding, "UTF-8");
        assert!(info.cwd.is_none());
        assert!(info.mode.is_none());
        assert!(info.scroll_position.is_none());
    }

    #[test]
    fn test_status_bar_config_default() {
        let config = StatusBarConfig::default();
        assert!(config.visible);
        assert!(config.show_cwd);
        assert!(config.show_size);
        assert!(config.show_encoding);
        assert!(config.show_scroll_position);
        assert!(config.show_mode);
    }

    #[test]
    fn test_status_bar_info_with_values() {
        let info = StatusBarInfo {
            shell: String::from("zsh"),
            cwd: Some(String::from("/home/user")),
            cols: 120,
            rows: 40,
            encoding: String::from("UTF-8"),
            mode: Some(String::from("streaming")),
            scroll_position: Some((100, 500)),
        };

        assert_eq!(info.shell, "zsh");
        assert_eq!(info.cwd, Some(String::from("/home/user")));
        assert_eq!(info.cols, 120);
        assert_eq!(info.rows, 40);
        assert_eq!(info.mode, Some(String::from("streaming")));
        assert_eq!(info.scroll_position, Some((100, 500)));
    }

    #[test]
    fn test_status_bar_config_custom() {
        let config = StatusBarConfig {
            visible: false,
            show_cwd: false,
            show_size: true,
            show_encoding: false,
            show_scroll_position: false,
            show_mode: true,
        };

        assert!(!config.visible);
        assert!(!config.show_cwd);
        assert!(config.show_size);
        assert!(!config.show_encoding);
    }
}
