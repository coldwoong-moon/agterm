//! Integration Tests for Floem GUI Components
//!
//! This file tests the core Floem GUI functionality including:
//! - Settings persistence and validation
//! - PaneTree management (splitting, navigation, closing)
//! - Theme switching
//!
//! Run with: cargo test --features floem-gui

#[cfg(feature = "floem-gui")]
mod floem_integration {
    use agterm::floem_app::settings::{Settings, CursorStyle};
    use agterm::floem_app::pane::{PaneTree, SplitDirection, NavigationDirection};
    use agterm::floem_app::theme::{Theme, ColorPalette};
    use agterm::terminal::pty::PtyManager;
    use std::sync::Arc;

    // ========================================================================
    // Settings Tests
    // ========================================================================

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();

        assert_eq!(settings.font_size, 14.0);
        assert_eq!(settings.theme_name, "Ghostty Dark");
        assert_eq!(settings.cursor_style, CursorStyle::Block);
        assert_eq!(settings.cursor_blink, true);
        assert_eq!(settings.scroll_back_lines, 10000);
        assert_eq!(settings.copy_on_select, false);
        assert_eq!(settings.confirm_close_with_running_processes, true);
        assert_eq!(settings.default_cols, Some(80));
        assert_eq!(settings.default_rows, Some(24));
    }

    #[test]
    fn test_settings_load_creates_default() {
        // When load() is called, it should return valid settings
        // (either from config file or defaults)
        let settings = Settings::load();

        // Font size should be within valid range (8.0-24.0)
        assert!(settings.font_size >= 8.0 && settings.font_size <= 24.0);

        // Theme name should not be empty
        assert!(!settings.theme_name.is_empty());

        // Config path should be accessible
        let _config_path = Settings::config_path().expect("Failed to get config path");
    }

    #[test]
    fn test_settings_validate_font_size() {
        let mut settings = Settings::default();

        // Test upper bound
        settings.font_size = 100.0;
        settings.validate();
        assert_eq!(settings.font_size, 24.0);

        // Test lower bound
        settings.font_size = 1.0;
        settings.validate();
        assert_eq!(settings.font_size, 8.0);

        // Test valid range
        settings.font_size = 16.0;
        settings.validate();
        assert_eq!(settings.font_size, 16.0);
    }

    #[test]
    fn test_settings_validate_scrollback() {
        let mut settings = Settings::default();

        // Too small
        settings.scroll_back_lines = 50;
        settings.validate();
        assert_eq!(settings.scroll_back_lines, 100);

        // Too large
        settings.scroll_back_lines = 200_000;
        settings.validate();
        assert_eq!(settings.scroll_back_lines, 100_000);

        // Valid
        settings.scroll_back_lines = 5000;
        settings.validate();
        assert_eq!(settings.scroll_back_lines, 5000);
    }

    #[test]
    fn test_settings_validate_terminal_size() {
        let mut settings = Settings::default();

        // Test cols validation
        settings.default_cols = Some(5);
        settings.validate();
        assert_eq!(settings.default_cols, Some(80));

        settings.default_cols = Some(1000);
        settings.validate();
        assert_eq!(settings.default_cols, Some(80));

        // Test rows validation
        settings.default_rows = Some(5);
        settings.validate();
        assert_eq!(settings.default_rows, Some(24));

        settings.default_rows = Some(500);
        settings.validate();
        assert_eq!(settings.default_rows, Some(24));

        // Valid values
        settings.default_cols = Some(100);
        settings.default_rows = Some(30);
        settings.validate();
        assert_eq!(settings.default_cols, Some(100));
        assert_eq!(settings.default_rows, Some(30));
    }

    #[test]
    fn test_settings_toml_serialization() {
        let settings = Settings::default();
        let toml_str = toml::to_string_pretty(&settings).expect("Failed to serialize");

        // Should contain all expected fields
        assert!(toml_str.contains("font_size"));
        assert!(toml_str.contains("theme_name"));
        assert!(toml_str.contains("cursor_style"));
        assert!(toml_str.contains("cursor_blink"));
        assert!(toml_str.contains("scroll_back_lines"));

        // Should be parseable
        let parsed: Settings = toml::from_str(&toml_str).expect("Failed to deserialize");
        assert_eq!(parsed.font_size, settings.font_size);
        assert_eq!(parsed.theme_name, settings.theme_name);
    }

    #[test]
    fn test_settings_cursor_styles() {
        let mut settings = Settings::default();

        // Test all cursor styles
        for style in [CursorStyle::Block, CursorStyle::Underline, CursorStyle::Bar] {
            settings.cursor_style = style;
            let toml_str = toml::to_string(&settings).expect("Failed to serialize");
            let parsed: Settings = toml::from_str(&toml_str).expect("Failed to deserialize");
            assert_eq!(parsed.cursor_style, style);
        }
    }

    #[test]
    fn test_cursor_style_serialization() {
        // Test cursor styles within a Settings struct (TOML can't serialize bare enums)
        let mut settings = Settings::default();

        // Block
        settings.cursor_style = CursorStyle::Block;
        let toml_str = toml::to_string(&settings).unwrap();
        assert!(toml_str.contains("cursor_style = \"block\""));

        // Bar
        settings.cursor_style = CursorStyle::Bar;
        let toml_str = toml::to_string(&settings).unwrap();
        assert!(toml_str.contains("cursor_style = \"bar\""));

        // Underline
        settings.cursor_style = CursorStyle::Underline;
        let toml_str = toml::to_string(&settings).unwrap();
        assert!(toml_str.contains("cursor_style = \"underline\""));
    }

    #[test]
    fn test_settings_partial_config() {
        // Test loading minimal config with defaults for missing fields
        let minimal_toml = r#"
            font_size = 16.0
            theme_name = "Ghostty Light"
        "#;

        let settings: Settings = toml::from_str(minimal_toml).expect("Failed to parse");

        // Explicit fields
        assert_eq!(settings.font_size, 16.0);
        assert_eq!(settings.theme_name, "Ghostty Light");

        // Default fields
        assert_eq!(settings.cursor_style, CursorStyle::Block);
        assert_eq!(settings.cursor_blink, true);
        assert_eq!(settings.scroll_back_lines, 10000);
    }

    // ========================================================================
    // Theme Tests
    // ========================================================================

    #[test]
    fn test_theme_from_name() {
        assert_eq!(Theme::from_name("Ghostty Dark"), Theme::GhosttyDark);
        assert_eq!(Theme::from_name("Ghostty Light"), Theme::GhosttyLight);

        // Default to dark for unknown themes
        assert_eq!(Theme::from_name("Unknown Theme"), Theme::GhosttyDark);
        assert_eq!(Theme::from_name(""), Theme::GhosttyDark);
    }

    #[test]
    fn test_theme_from_name_opt() {
        assert_eq!(Theme::from_name_opt("Ghostty Dark"), Some(Theme::GhosttyDark));
        assert_eq!(Theme::from_name_opt("Ghostty Light"), Some(Theme::GhosttyLight));

        // Return None for unknown themes
        assert_eq!(Theme::from_name_opt("Unknown"), None);
        assert_eq!(Theme::from_name_opt(""), None);
    }

    #[test]
    fn test_theme_name() {
        assert_eq!(Theme::GhosttyDark.name(), "Ghostty Dark");
        assert_eq!(Theme::GhosttyLight.name(), "Ghostty Light");
    }

    #[test]
    fn test_theme_toggle() {
        let dark = Theme::GhosttyDark;
        let light = Theme::GhosttyLight;

        assert_eq!(dark.toggle(), light);
        assert_eq!(light.toggle(), dark);

        // Toggle twice returns to original
        assert_eq!(dark.toggle().toggle(), dark);
    }

    #[test]
    fn test_theme_colors() {
        let dark_colors = Theme::GhosttyDark.colors();
        let light_colors = Theme::GhosttyLight.colors();

        // Dark theme should have dark backgrounds
        // Note: We can't directly compare Color values, so we just ensure colors() works
        let _dark_bg = dark_colors.bg_primary;
        let _light_bg = light_colors.bg_primary;

        // Ensure color palettes are different
        assert_ne!(
            format!("{:?}", dark_colors.bg_primary),
            format!("{:?}", light_colors.bg_primary)
        );
    }

    #[test]
    fn test_color_palette_completeness() {
        let colors = Theme::GhosttyDark.colors();

        // Ensure all color fields are accessible
        let _ = colors.bg_primary;
        let _ = colors.bg_secondary;
        let _ = colors.bg_tab_bar;
        let _ = colors.bg_tab_active;
        let _ = colors.bg_tab_hover;
        let _ = colors.bg_status;
        let _ = colors.text_primary;
        let _ = colors.text_secondary;
        let _ = colors.text_muted;
        let _ = colors.accent_blue;
        let _ = colors.accent_green;
        let _ = colors.accent_red;
        let _ = colors.border;
        let _ = colors.border_subtle;
    }

    // ========================================================================
    // PaneTree Tests
    // ========================================================================

    #[test]
    fn test_pane_tree_new_leaf() {
        let pty_manager = Arc::new(PtyManager::new());
        let pane = PaneTree::new_leaf(&pty_manager);

        match pane {
            PaneTree::Leaf { id, terminal_state, is_focused } => {
                // UUID should be valid
                assert_ne!(id.to_string(), "");

                // Terminal state should exist
                let _state = terminal_state;

                // New panes are focused by default
                use floem::reactive::SignalGet;
                assert_eq!(is_focused.get(), true);
            }
            _ => panic!("Expected Leaf, got Split"),
        }
    }

    #[test]
    fn test_pane_tree_count_leaves() {
        let pty_manager = Arc::new(PtyManager::new());
        let pane = PaneTree::new_leaf(&pty_manager);

        // Single pane
        assert_eq!(pane.count_leaves(), 1);
    }

    #[test]
    fn test_pane_tree_split_horizontal() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);

        // Split horizontally
        pane.split_horizontal(&pty_manager);

        match &pane {
            PaneTree::Split { direction, first, second, ratio } => {
                assert_eq!(*direction, SplitDirection::Horizontal);

                use floem::reactive::SignalGet;
                assert_eq!(ratio.get(), 0.5);

                // Should have two leaf children
                use floem::reactive::SignalWith;
                assert!(matches!(first.with(|t| t.clone()), PaneTree::Leaf { .. }));
                assert!(matches!(second.with(|t| t.clone()), PaneTree::Leaf { .. }));
            }
            _ => panic!("Expected Split, got Leaf"),
        }

        // Should now have 2 leaves
        assert_eq!(pane.count_leaves(), 2);
    }

    #[test]
    fn test_pane_tree_split_vertical() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);

        // Split vertically
        pane.split_vertical(&pty_manager);

        match &pane {
            PaneTree::Split { direction, .. } => {
                assert_eq!(*direction, SplitDirection::Vertical);
            }
            _ => panic!("Expected Split, got Leaf"),
        }

        assert_eq!(pane.count_leaves(), 2);
    }

    #[test]
    fn test_pane_tree_multiple_splits() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);

        // First split
        pane.split_horizontal(&pty_manager);
        assert_eq!(pane.count_leaves(), 2);

        // Split again (splits the entire tree, creating 2 more leaves)
        pane.split_vertical(&pty_manager);
        assert_eq!(pane.count_leaves(), 3);
    }

    #[test]
    fn test_pane_tree_get_all_leaf_ids() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);

        // Single pane - should have 1 ID
        let ids = pane.get_all_leaf_ids();
        assert_eq!(ids.len(), 1);

        // After split - should have 2 IDs
        pane.split_horizontal(&pty_manager);
        let ids = pane.get_all_leaf_ids();
        assert_eq!(ids.len(), 2);

        // IDs should be unique
        assert_ne!(ids[0], ids[1]);
    }

    #[test]
    fn test_pane_tree_focus_management() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);
        pane.split_horizontal(&pty_manager);

        let ids = pane.get_all_leaf_ids();
        assert_eq!(ids.len(), 2);

        // Set focus on first pane
        let success = pane.set_focus(ids[0]);
        assert!(success);

        // Get focused leaf
        let focused = pane.get_focused_leaf();
        assert!(focused.is_some());
        let (focused_id, _) = focused.unwrap();
        assert_eq!(focused_id, ids[0]);

        // Clear focus
        pane.clear_focus();
        let focused = pane.get_focused_leaf();
        assert!(focused.is_none());
    }

    #[test]
    fn test_pane_tree_navigation_next() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);
        pane.split_horizontal(&pty_manager);

        let ids = pane.get_all_leaf_ids();

        // Focus first pane
        pane.set_focus(ids[0]);

        // Navigate to next
        let next_id = pane.navigate(NavigationDirection::Next);
        assert!(next_id.is_some());
        assert_eq!(next_id.unwrap(), ids[1]);
    }

    #[test]
    fn test_pane_tree_navigation_previous() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);
        pane.split_horizontal(&pty_manager);

        let ids = pane.get_all_leaf_ids();

        // Focus second pane
        pane.set_focus(ids[1]);

        // Navigate to previous
        let prev_id = pane.navigate(NavigationDirection::Previous);
        assert!(prev_id.is_some());
        assert_eq!(prev_id.unwrap(), ids[0]);
    }

    #[test]
    fn test_pane_tree_navigation_wraps() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);
        pane.split_horizontal(&pty_manager);

        let ids = pane.get_all_leaf_ids();

        // Focus last pane, navigate next (should wrap to first)
        pane.set_focus(ids[1]);
        let next_id = pane.navigate(NavigationDirection::Next);
        assert_eq!(next_id.unwrap(), ids[0]);

        // Focus first pane, navigate previous (should wrap to last)
        pane.set_focus(ids[0]);
        let prev_id = pane.navigate(NavigationDirection::Previous);
        assert_eq!(prev_id.unwrap(), ids[1]);
    }

    #[test]
    fn test_pane_tree_close_focused_pane() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);
        pane.split_horizontal(&pty_manager);

        assert_eq!(pane.count_leaves(), 2);
        let ids = pane.get_all_leaf_ids();

        // Focus and close first pane
        pane.set_focus(ids[0]);
        let closed = pane.close_focused_pane(&pty_manager);

        assert!(closed);
        assert_eq!(pane.count_leaves(), 1);

        // Remaining pane should be the second one
        let remaining_ids = pane.get_all_leaf_ids();
        assert_eq!(remaining_ids.len(), 1);
    }

    #[test]
    fn test_pane_tree_cannot_close_last_pane() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);

        let ids = pane.get_all_leaf_ids();
        pane.set_focus(ids[0]);

        // Try to close the only pane
        let _closed = pane.close_focused_pane(&pty_manager);

        // Should fail or handle gracefully (implementation specific)
        // If it returns true, the pane tree might be in an invalid state
        // This test documents the current behavior
        assert_eq!(pane.count_leaves(), 1);
    }

    #[test]
    fn test_pane_tree_get_focused_title() {
        let pty_manager = Arc::new(PtyManager::new());
        let pane = PaneTree::new_leaf(&pty_manager);

        let ids = pane.get_all_leaf_ids();
        pane.set_focus(ids[0]);

        // Get title (should return default or PTY title)
        let title = pane.get_focused_title("Default Title");

        // Should not be empty
        assert!(!title.is_empty());
    }

    #[test]
    fn test_pane_tree_complex_split_structure() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);

        // Create complex structure:
        // - Split horizontal (2 panes)
        // - Split the result vertical (3 panes)
        pane.split_horizontal(&pty_manager);
        assert_eq!(pane.count_leaves(), 2);

        pane.split_vertical(&pty_manager);
        assert_eq!(pane.count_leaves(), 3);

        // All leaves should be navigable
        let ids = pane.get_all_leaf_ids();
        assert_eq!(ids.len(), 3);

        // Should be able to focus each one
        for id in ids {
            let success = pane.set_focus(id);
            assert!(success);
        }
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[test]
    fn test_settings_with_theme() {
        let mut settings = Settings::default();
        settings.theme_name = "Ghostty Light".to_string();

        // Parse theme from settings
        let theme = Theme::from_name(&settings.theme_name);
        assert_eq!(theme, Theme::GhosttyLight);

        // Toggle theme
        let new_theme = theme.toggle();
        settings.theme_name = new_theme.name().to_string();
        assert_eq!(settings.theme_name, "Ghostty Dark");
    }

    #[test]
    fn test_pane_tree_with_settings() {
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);
        let settings = Settings::default();

        // Settings should define terminal size
        assert!(settings.default_cols.is_some());
        assert!(settings.default_rows.is_some());

        // Pane tree should be able to use these settings
        pane.split_horizontal(&pty_manager);
        assert_eq!(pane.count_leaves(), 2);
    }

    #[test]
    fn test_full_workflow_settings_theme_panes() {
        // Simulate a full workflow:
        // 1. Load settings
        let mut settings = Settings::default();
        settings.validate();

        // 2. Parse theme
        let theme = Theme::from_name(&settings.theme_name);
        let _colors = theme.colors();

        // 3. Create pane tree
        let pty_manager = Arc::new(PtyManager::new());
        let mut pane = PaneTree::new_leaf(&pty_manager);

        // 4. Split panes
        pane.split_horizontal(&pty_manager);
        pane.split_vertical(&pty_manager);

        // 5. Navigate panes
        let ids = pane.get_all_leaf_ids();
        pane.set_focus(ids[0]);

        // 6. Toggle theme
        settings.theme_name = theme.toggle().name().to_string();

        // 7. Update font size
        settings.font_size = 16.0;
        settings.validate();

        // Everything should work together
        assert_eq!(pane.count_leaves(), 3);
        assert_eq!(settings.font_size, 16.0);
        assert_eq!(settings.theme_name, "Ghostty Light");
    }
}
