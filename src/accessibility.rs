//! Accessibility Module for AgTerm
//!
//! This module provides comprehensive accessibility features including:
//! - Screen reader support with announcement queue
//! - High contrast themes with WCAG compliance checking
//! - Keyboard-only navigation
//! - Reduced motion settings
//! - Focus indicators
//! - Visual bell alternative
//! - Font scaling for low vision users

use iced::keyboard::Key;
use iced::{Color, Rectangle as Rect};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::theme::Theme;

// ============================================================================
// Configuration
// ============================================================================

/// Accessibility configuration
#[derive(Debug, Clone)]
pub struct AccessibilityConfig {
    pub screen_reader_enabled: bool,
    pub high_contrast_mode: bool,
    pub reduced_motion: bool,
    pub large_cursor: bool,
    pub cursor_blink_disabled: bool,
    pub bell_visual: bool,
    pub announce_output: bool,
    pub font_scale: f32,
    pub minimum_contrast_ratio: f32,
    pub focus_highlight: bool,
    pub keyboard_only: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            screen_reader_enabled: false,
            high_contrast_mode: false,
            reduced_motion: false,
            large_cursor: false,
            cursor_blink_disabled: false,
            bell_visual: false,
            announce_output: false,
            font_scale: 1.0,
            minimum_contrast_ratio: 4.5, // WCAG AA standard
            focus_highlight: true,
            keyboard_only: false,
        }
    }
}

// ============================================================================
// Screen Reader Support
// ============================================================================

/// Priority level for screen reader announcements
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnnouncePriority {
    Background = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Screen reader announcement
#[derive(Debug, Clone)]
pub struct ScreenReaderAnnouncement {
    pub text: String,
    pub priority: AnnouncePriority,
    pub interrupt: bool,
    pub timestamp: Instant,
}

impl ScreenReaderAnnouncement {
    pub fn new(text: String, priority: AnnouncePriority) -> Self {
        Self {
            text,
            priority,
            interrupt: false,
            timestamp: Instant::now(),
        }
    }

    pub fn with_interrupt(mut self, interrupt: bool) -> Self {
        self.interrupt = interrupt;
        self
    }
}

/// Screen reader engine for text-to-speech announcements
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScreenReader {
    queue: VecDeque<ScreenReaderAnnouncement>,
    enabled: bool,
    rate: f32,
    pitch: f32,
    volume: f32,
}

impl ScreenReader {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            enabled: false,
            rate: 1.0,
            pitch: 1.0,
            volume: 1.0,
        }
    }

    /// Announce text with given priority
    pub fn announce(&mut self, text: &str, priority: AnnouncePriority) {
        if !self.enabled {
            return;
        }

        let announcement = ScreenReaderAnnouncement::new(
            Self::format_for_speech(text),
            priority,
        );

        // Insert based on priority
        match priority {
            AnnouncePriority::Critical => {
                self.queue.push_front(announcement.with_interrupt(true));
            }
            AnnouncePriority::High => {
                // Insert at position 1 if queue not empty, else front
                if self.queue.is_empty() {
                    self.queue.push_front(announcement);
                } else {
                    self.queue.insert(1, announcement);
                }
            }
            _ => {
                self.queue.push_back(announcement);
            }
        }

        // Limit queue size
        while self.queue.len() > 50 {
            self.queue.pop_back();
        }
    }

    /// Announce terminal output
    pub fn announce_output(&mut self, lines: &[String]) {
        if !self.enabled {
            return;
        }

        for line in lines {
            let cleaned = Self::format_for_speech(line);
            if !cleaned.is_empty() {
                self.announce(&cleaned, AnnouncePriority::Low);
            }
        }
    }

    /// Announce command prompt
    pub fn announce_prompt(&mut self, prompt: &str) {
        self.announce(
            &format!("Command prompt: {prompt}"),
            AnnouncePriority::Normal,
        );
    }

    /// Announce terminal bell
    pub fn announce_bell(&mut self) {
        self.announce("Bell", AnnouncePriority::High);
    }

    /// Announce cursor position
    pub fn announce_cursor_position(&mut self, row: usize, col: usize) {
        self.announce(
            &format!("Cursor at row {}, column {}", row + 1, col + 1),
            AnnouncePriority::Low,
        );
    }

    /// Clear announcement queue
    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    /// Get next announcement to speak
    pub fn next(&mut self) -> Option<ScreenReaderAnnouncement> {
        self.queue.pop_front()
    }

    /// Format text for speech (clean ANSI codes, special characters)
    pub fn format_for_speech(text: &str) -> String {
        let mut result = String::new();
        let mut in_escape = false;

        for ch in text.chars() {
            match ch {
                '\x1b' => in_escape = true,
                'm' if in_escape => in_escape = false,
                _ if in_escape => continue,
                '\t' => result.push_str("    "),
                '\r' => continue,
                '\n' => result.push(' '),
                ch if ch.is_control() => continue,
                ch => result.push(ch),
            }
        }

        result.trim().to_string()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for ScreenReader {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Contrast Checking (WCAG Compliance)
// ============================================================================

/// Contrast ratio checker for WCAG compliance
#[derive(Debug, Clone, Copy)]
pub struct ContrastChecker;

impl ContrastChecker {
    /// Calculate contrast ratio between two colors (1:1 to 21:1)
    pub fn check_ratio(fg: Color, bg: Color) -> f64 {
        let l1 = Self::luminance(fg);
        let l2 = Self::luminance(bg);

        let lighter = l1.max(l2);
        let darker = l1.min(l2);

        (lighter + 0.05) / (darker + 0.05)
    }

    /// Check if colors meet WCAG AA standard (4.5:1)
    pub fn meets_wcag_aa(fg: Color, bg: Color) -> bool {
        Self::check_ratio(fg, bg) >= 4.5
    }

    /// Check if colors meet WCAG AAA standard (7:1)
    pub fn meets_wcag_aaa(fg: Color, bg: Color) -> bool {
        Self::check_ratio(fg, bg) >= 7.0
    }

    /// Suggest color adjustments to meet target ratio
    pub fn suggest_adjustment(fg: Color, bg: Color, target_ratio: f64) -> (Color, Color) {
        let current_ratio = Self::check_ratio(fg, bg);

        if current_ratio >= target_ratio {
            return (fg, bg);
        }

        // Try darkening foreground
        for step in 1..=20 {
            let factor = 1.0 - (step as f32 * 0.05);
            let adjusted_fg = Color {
                r: fg.r * factor,
                g: fg.g * factor,
                b: fg.b * factor,
                a: fg.a,
            };

            if Self::check_ratio(adjusted_fg, bg) >= target_ratio {
                return (adjusted_fg, bg);
            }
        }

        // Try lightening background
        for step in 1..=20 {
            let factor = 1.0 + (step as f32 * 0.05);
            let adjusted_bg = Color {
                r: (bg.r * factor).min(1.0),
                g: (bg.g * factor).min(1.0),
                b: (bg.b * factor).min(1.0),
                a: bg.a,
            };

            if Self::check_ratio(fg, adjusted_bg) >= target_ratio {
                return (fg, adjusted_bg);
            }
        }

        // If still not meeting ratio, return high contrast black/white
        if Self::luminance(bg) > 0.5 {
            (Color::BLACK, bg)
        } else {
            (Color::WHITE, bg)
        }
    }

    /// Calculate relative luminance of a color (0.0 to 1.0)
    pub fn luminance(color: Color) -> f64 {
        fn linearize(component: f32) -> f64 {
            let c = component as f64;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }

        let r = linearize(color.r);
        let g = linearize(color.g);
        let b = linearize(color.b);

        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}

// ============================================================================
// High Contrast Themes
// ============================================================================

/// High contrast theme optimized for accessibility
#[derive(Debug, Clone)]
pub struct HighContrastTheme {
    pub foreground: Color,
    pub background: Color,
    pub cursor: Color,
    pub selection: Color,
    pub link: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
}

impl HighContrastTheme {
    /// Default high contrast theme (white on black)
    pub fn default_high_contrast() -> Self {
        Self {
            foreground: Color::WHITE,
            background: Color::BLACK,
            cursor: Color::from_rgb(1.0, 1.0, 0.0), // Yellow
            selection: Color::from_rgb(0.0, 0.5, 1.0), // Blue
            link: Color::from_rgb(0.0, 0.8, 1.0), // Cyan
            error: Color::from_rgb(1.0, 0.0, 0.0), // Red
            warning: Color::from_rgb(1.0, 0.8, 0.0), // Yellow-orange
            success: Color::from_rgb(0.0, 1.0, 0.0), // Green
        }
    }

    /// Inverted high contrast theme (black on white)
    pub fn inverted_high_contrast() -> Self {
        Self {
            foreground: Color::BLACK,
            background: Color::WHITE,
            cursor: Color::from_rgb(0.8, 0.3, 0.0), // Dark orange - better contrast
            selection: Color::from_rgb(0.0, 0.3, 0.6), // Darker blue
            link: Color::from_rgb(0.0, 0.0, 0.6), // Darker blue
            error: Color::from_rgb(0.6, 0.0, 0.0), // Darker red
            warning: Color::from_rgb(0.6, 0.3, 0.0), // Darker orange
            success: Color::from_rgb(0.0, 0.5, 0.0), // Darker green
        }
    }

    /// Verify all colors meet WCAG AA standard
    pub fn verify_wcag_aa(&self) -> bool {
        ContrastChecker::meets_wcag_aa(self.foreground, self.background)
            && ContrastChecker::meets_wcag_aa(self.cursor, self.background)
            && ContrastChecker::meets_wcag_aa(self.link, self.background)
            && ContrastChecker::meets_wcag_aa(self.error, self.background)
            && ContrastChecker::meets_wcag_aa(self.warning, self.background)
            && ContrastChecker::meets_wcag_aa(self.success, self.background)
    }
}

// ============================================================================
// Focus Management
// ============================================================================

/// Types of focusable UI elements
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusableElement {
    Terminal,
    Tab(usize),
    SearchBar,
    CommandPalette,
    SettingsPanel,
    Button(String),
    Input(String),
}

/// Focus animation style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusAnimation {
    Pulse,
    Glow,
    Solid,
    None,
}

/// Focus indicator style
#[derive(Debug, Clone)]
pub struct FocusStyle {
    pub border_color: Color,
    pub border_width: f32,
    pub animation: Option<FocusAnimation>,
}

impl Default for FocusStyle {
    fn default() -> Self {
        Self {
            border_color: Color::from_rgb(0.0, 0.5, 1.0),
            border_width: 2.0,
            animation: Some(FocusAnimation::Solid),
        }
    }
}

/// Focus indicator overlay
#[derive(Debug, Clone)]
pub struct FocusIndicator {
    pub visible: bool,
    pub element_type: FocusableElement,
    pub bounds: Rect,
    pub style: FocusStyle,
}

impl FocusIndicator {
    pub fn new(element_type: FocusableElement, bounds: Rect) -> Self {
        Self {
            visible: true,
            element_type,
            bounds,
            style: FocusStyle::default(),
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn show(&mut self) {
        self.visible = true;
    }
}

// ============================================================================
// Keyboard Navigation
// ============================================================================

/// Navigation action resulting from keyboard input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationAction {
    MoveFocusNext,
    MoveFocusPrev,
    Activate,
    Cancel,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

/// Keyboard-only navigation manager
#[derive(Debug, Clone)]
pub struct KeyboardNavigation {
    focusable_elements: Vec<FocusableElement>,
    current_focus: usize,
}

impl KeyboardNavigation {
    pub fn new() -> Self {
        Self {
            focusable_elements: vec![FocusableElement::Terminal],
            current_focus: 0,
        }
    }

    /// Add a focusable element
    pub fn add_element(&mut self, element: FocusableElement) {
        self.focusable_elements.push(element);
    }

    /// Remove a focusable element
    pub fn remove_element(&mut self, element: &FocusableElement) {
        self.focusable_elements.retain(|e| e != element);
        if self.current_focus >= self.focusable_elements.len() {
            self.current_focus = self.focusable_elements.len().saturating_sub(1);
        }
    }

    /// Move focus to next element
    pub fn next(&mut self) {
        if !self.focusable_elements.is_empty() {
            self.current_focus = (self.current_focus + 1) % self.focusable_elements.len();
        }
    }

    /// Move focus to previous element
    pub fn prev(&mut self) {
        if !self.focusable_elements.is_empty() {
            self.current_focus = if self.current_focus == 0 {
                self.focusable_elements.len() - 1
            } else {
                self.current_focus - 1
            };
        }
    }

    /// Move focus to first element
    pub fn focus_first(&mut self) {
        self.current_focus = 0;
    }

    /// Move focus to last element
    pub fn focus_last(&mut self) {
        self.current_focus = self.focusable_elements.len().saturating_sub(1);
    }

    /// Focus specific element
    pub fn focus_element(&mut self, element: FocusableElement) {
        if let Some(pos) = self.focusable_elements.iter().position(|e| e == &element) {
            self.current_focus = pos;
        }
    }

    /// Get currently focused element
    pub fn current(&self) -> Option<&FocusableElement> {
        self.focusable_elements.get(self.current_focus)
    }

    /// Handle keyboard input and return navigation action
    pub fn handle_key(key: Key) -> Option<NavigationAction> {
        match key {
            Key::Named(iced::keyboard::key::Named::Tab) => Some(NavigationAction::MoveFocusNext),
            Key::Named(iced::keyboard::key::Named::Enter) => Some(NavigationAction::Activate),
            Key::Named(iced::keyboard::key::Named::Escape) => Some(NavigationAction::Cancel),
            Key::Named(iced::keyboard::key::Named::ArrowUp) => Some(NavigationAction::MoveUp),
            Key::Named(iced::keyboard::key::Named::ArrowDown) => Some(NavigationAction::MoveDown),
            Key::Named(iced::keyboard::key::Named::ArrowLeft) => Some(NavigationAction::MoveLeft),
            Key::Named(iced::keyboard::key::Named::ArrowRight) => Some(NavigationAction::MoveRight),
            _ => None,
        }
    }
}

impl Default for KeyboardNavigation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Motion Settings
// ============================================================================

/// Scroll behavior preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollBehavior {
    Smooth,
    Instant,
    Auto,
}

/// Motion and animation settings
#[derive(Debug, Clone)]
pub struct MotionSettings {
    pub animation_duration: Duration,
    pub transition_enabled: bool,
    pub scroll_behavior: ScrollBehavior,
}

impl Default for MotionSettings {
    fn default() -> Self {
        Self {
            animation_duration: Duration::from_millis(200),
            transition_enabled: true,
            scroll_behavior: ScrollBehavior::Smooth,
        }
    }
}

impl MotionSettings {
    pub fn reduced_motion() -> Self {
        Self {
            animation_duration: Duration::from_millis(0),
            transition_enabled: false,
            scroll_behavior: ScrollBehavior::Instant,
        }
    }
}

// ============================================================================
// Accessibility Manager
// ============================================================================

/// Central accessibility manager
#[derive(Debug, Clone)]
pub struct AccessibilityManager {
    pub config: AccessibilityConfig,
    pub screen_reader: ScreenReader,
    pub keyboard_nav: KeyboardNavigation,
    pub contrast_checker: ContrastChecker,
}

impl AccessibilityManager {
    pub fn new(config: AccessibilityConfig) -> Self {
        let mut screen_reader = ScreenReader::new();
        screen_reader.set_enabled(config.screen_reader_enabled);

        Self {
            config,
            screen_reader,
            keyboard_nav: KeyboardNavigation::new(),
            contrast_checker: ContrastChecker,
        }
    }

    /// Apply high contrast adjustments to theme
    pub fn apply_high_contrast(&self, theme: &mut Theme) {
        if !self.config.high_contrast_mode {
            return;
        }

        let hc_theme = HighContrastTheme::default_high_contrast();

        // Update terminal colors
        theme.terminal.foreground = crate::theme::ColorDef::from_rgb_float(
            hc_theme.foreground.r,
            hc_theme.foreground.g,
            hc_theme.foreground.b,
        );
        theme.terminal.background = crate::theme::ColorDef::from_rgb_float(
            hc_theme.background.r,
            hc_theme.background.g,
            hc_theme.background.b,
        );
        theme.terminal.cursor = crate::theme::ColorDef::from_rgb_float(
            hc_theme.cursor.r,
            hc_theme.cursor.g,
            hc_theme.cursor.b,
        );

        // Update UI colors
        theme.ui.text_primary = crate::theme::ColorDef::from_rgb_float(
            hc_theme.foreground.r,
            hc_theme.foreground.g,
            hc_theme.foreground.b,
        );
        theme.ui.bg_primary = crate::theme::ColorDef::from_rgb_float(
            hc_theme.background.r,
            hc_theme.background.g,
            hc_theme.background.b,
        );
    }

    /// Adjust animation duration for reduced motion
    pub fn adjust_for_reduced_motion(&self, animation: &mut Duration) {
        if self.config.reduced_motion {
            *animation = Duration::from_millis(0);
        }
    }

    /// Get ARIA label for UI element
    pub fn get_aria_label(element: &FocusableElement) -> String {
        match element {
            FocusableElement::Terminal => "Terminal window".to_string(),
            FocusableElement::Tab(idx) => format!("Tab {}", idx + 1),
            FocusableElement::SearchBar => "Search bar".to_string(),
            FocusableElement::CommandPalette => "Command palette".to_string(),
            FocusableElement::SettingsPanel => "Settings panel".to_string(),
            FocusableElement::Button(name) => format!("{name} button"),
            FocusableElement::Input(name) => format!("{name} input"),
        }
    }

    /// Handle keyboard input for navigation
    pub fn handle_key(&mut self, key: Key) -> Option<NavigationAction> {
        if !self.config.keyboard_only && !self.config.focus_highlight {
            return None;
        }

        let action = KeyboardNavigation::handle_key(key)?;

        match action {
            NavigationAction::MoveFocusNext => {
                self.keyboard_nav.next();
                if let Some(element) = self.keyboard_nav.current() {
                    self.screen_reader.announce(
                        &Self::get_aria_label(element),
                        AnnouncePriority::Normal,
                    );
                }
            }
            NavigationAction::MoveFocusPrev => {
                self.keyboard_nav.prev();
                if let Some(element) = self.keyboard_nav.current() {
                    self.screen_reader.announce(
                        &Self::get_aria_label(element),
                        AnnouncePriority::Normal,
                    );
                }
            }
            _ => {}
        }

        Some(action)
    }

    /// Get motion settings based on config
    pub fn motion_settings(&self) -> MotionSettings {
        if self.config.reduced_motion {
            MotionSettings::reduced_motion()
        } else {
            MotionSettings::default()
        }
    }
}

impl Default for AccessibilityManager {
    fn default() -> Self {
        Self::new(AccessibilityConfig::default())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_reader_format_for_speech() {
        let input = "\x1b[31mError:\x1b[0m File not found\n";
        let output = ScreenReader::format_for_speech(input);
        assert_eq!(output, "Error: File not found");
    }

    #[test]
    fn test_screen_reader_announcement_priority() {
        let mut sr = ScreenReader::new();
        sr.set_enabled(true);

        sr.announce("low", AnnouncePriority::Low);
        sr.announce("critical", AnnouncePriority::Critical);
        sr.announce("normal", AnnouncePriority::Normal);

        // Critical should be first
        assert_eq!(sr.next().unwrap().text, "critical");
    }

    #[test]
    fn test_contrast_ratio_calculation() {
        let white = Color::WHITE;
        let black = Color::BLACK;

        let ratio = ContrastChecker::check_ratio(white, black);
        assert!((ratio - 21.0).abs() < 0.1); // Maximum contrast
    }

    #[test]
    fn test_wcag_aa_compliance() {
        let white = Color::WHITE;
        let black = Color::BLACK;

        assert!(ContrastChecker::meets_wcag_aa(white, black));
    }

    #[test]
    fn test_wcag_aaa_compliance() {
        let white = Color::WHITE;
        let black = Color::BLACK;

        assert!(ContrastChecker::meets_wcag_aaa(white, black));
    }

    #[test]
    fn test_insufficient_contrast() {
        let light_gray = Color::from_rgb(0.9, 0.9, 0.9);
        let white = Color::WHITE;

        assert!(!ContrastChecker::meets_wcag_aa(light_gray, white));
    }

    #[test]
    fn test_luminance_calculation() {
        let white = Color::WHITE;
        let black = Color::BLACK;

        let white_lum = ContrastChecker::luminance(white);
        let black_lum = ContrastChecker::luminance(black);

        assert!(white_lum > black_lum);
        assert!((white_lum - 1.0).abs() < 0.1);
        assert!(black_lum < 0.1);
    }

    #[test]
    fn test_high_contrast_theme_wcag_compliance() {
        let theme = HighContrastTheme::default_high_contrast();
        assert!(theme.verify_wcag_aa());
    }

    #[test]
    fn test_inverted_high_contrast_theme() {
        let theme = HighContrastTheme::inverted_high_contrast();
        assert_eq!(theme.foreground, Color::BLACK);
        assert_eq!(theme.background, Color::WHITE);
        assert!(theme.verify_wcag_aa());
    }

    #[test]
    fn test_keyboard_navigation_next() {
        let mut nav = KeyboardNavigation::new();
        nav.add_element(FocusableElement::Tab(0));
        nav.add_element(FocusableElement::SearchBar);

        assert_eq!(nav.current(), Some(&FocusableElement::Terminal));
        nav.next();
        assert_eq!(nav.current(), Some(&FocusableElement::Tab(0)));
        nav.next();
        assert_eq!(nav.current(), Some(&FocusableElement::SearchBar));
    }

    #[test]
    fn test_keyboard_navigation_prev() {
        let mut nav = KeyboardNavigation::new();
        nav.add_element(FocusableElement::Tab(0));

        nav.prev(); // Should wrap to last
        assert_eq!(nav.current(), Some(&FocusableElement::Tab(0)));
    }

    #[test]
    fn test_keyboard_navigation_focus_element() {
        let mut nav = KeyboardNavigation::new();
        let search = FocusableElement::SearchBar;
        nav.add_element(search.clone());

        nav.focus_element(search.clone());
        assert_eq!(nav.current(), Some(&search));
    }

    #[test]
    fn test_reduced_motion_settings() {
        let settings = MotionSettings::reduced_motion();
        assert_eq!(settings.animation_duration, Duration::from_millis(0));
        assert!(!settings.transition_enabled);
        assert_eq!(settings.scroll_behavior, ScrollBehavior::Instant);
    }

    #[test]
    fn test_accessibility_config_defaults() {
        let config = AccessibilityConfig::default();
        assert!(!config.screen_reader_enabled);
        assert!(!config.high_contrast_mode);
        assert_eq!(config.font_scale, 1.0);
        assert_eq!(config.minimum_contrast_ratio, 4.5);
    }

    #[test]
    fn test_accessibility_manager_creation() {
        let config = AccessibilityConfig::default();
        let manager = AccessibilityManager::new(config);

        assert!(!manager.screen_reader.is_enabled());
        assert!(manager.keyboard_nav.current().is_some());
    }

    #[test]
    fn test_aria_label_generation() {
        assert_eq!(
            AccessibilityManager::get_aria_label(&FocusableElement::Terminal),
            "Terminal window"
        );
        assert_eq!(
            AccessibilityManager::get_aria_label(&FocusableElement::Tab(2)),
            "Tab 3"
        );
        assert_eq!(
            AccessibilityManager::get_aria_label(&FocusableElement::Button("OK".to_string())),
            "OK button"
        );
    }

    #[test]
    fn test_suggest_adjustment_no_change_needed() {
        let white = Color::WHITE;
        let black = Color::BLACK;

        let (adjusted_fg, adjusted_bg) = ContrastChecker::suggest_adjustment(white, black, 4.5);

        // Should remain unchanged as already meets target
        assert_eq!(adjusted_fg, white);
        assert_eq!(adjusted_bg, black);
    }

    #[test]
    fn test_screen_reader_queue_limit() {
        let mut sr = ScreenReader::new();
        sr.set_enabled(true);

        // Add more than 50 announcements
        for i in 0..60 {
            sr.announce(&format!("Message {}", i), AnnouncePriority::Low);
        }

        // Queue should be limited to 50
        let mut count = 0;
        while sr.next().is_some() {
            count += 1;
        }
        assert_eq!(count, 50);
    }

    #[test]
    fn test_focus_indicator_visibility() {
        let mut indicator = FocusIndicator::new(
            FocusableElement::Terminal,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
        );

        assert!(indicator.visible);
        indicator.hide();
        assert!(!indicator.visible);
        indicator.show();
        assert!(indicator.visible);
    }
}
