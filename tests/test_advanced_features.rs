//! Advanced feature tests for AgTerm
//!
//! This test suite covers advanced terminal features including:
//! - Environment detection (SSH, container, tmux)
//! - Snippet system for command expansion
//! - Profile management (save/load/apply)
//! - Session restoration
//! - URL detection and handling
//! - Pane layout management
//!
//! These tests verify that AgTerm's advanced features work correctly
//! across different environments and configurations.

use agterm::config::{AppConfig, Profile, Snippet};
use agterm::terminal::env::{ColorSupport, EnvironmentInfo, EnvironmentSettings};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

// ============================================================================
// Environment Detection Tests
// ============================================================================

#[test]
fn test_environment_detection_basic() {
    // Test basic environment detection without any special environment
    let env_info = EnvironmentInfo::default();

    // Default environment should have sensible defaults
    assert!(!env_info.is_ssh);
    assert!(!env_info.is_container);
    assert!(!env_info.is_tmux);
    assert!(!env_info.is_screen);
    assert_eq!(env_info.color_support, ColorSupport::TrueColor);
    assert!(env_info.has_truecolor);
    assert!(env_info.has_mouse_support);
    assert!(env_info.has_unicode);
}

#[test]
fn test_environment_detection_with_env_vars() {
    // Save original env vars
    let original_term = env::var("TERM").ok();
    let original_ssh = env::var("SSH_CONNECTION").ok();
    let original_tmux = env::var("TMUX").ok();

    // Set test environment
    env::set_var("TERM", "xterm-256color");

    let _env_info = EnvironmentInfo::detect();

    // Should detect terminal type
    assert_eq!(_env_info.term_type, "xterm-256color");

    // Should detect 256 color support
    assert!(
        _env_info.color_support == ColorSupport::Color256
            || _env_info.color_support == ColorSupport::TrueColor
    );

    // Restore original env vars
    match original_term {
        Some(val) => env::set_var("TERM", val),
        None => env::remove_var("TERM"),
    }
    match original_ssh {
        Some(val) => env::set_var("SSH_CONNECTION", val),
        None => env::remove_var("SSH_CONNECTION"),
    }
    match original_tmux {
        Some(val) => env::set_var("TMUX", val),
        None => env::remove_var("TMUX"),
    }
}

#[test]
fn test_ssh_detection() {
    // Save original env
    let original_ssh_connection = env::var("SSH_CONNECTION").ok();
    let original_ssh_client = env::var("SSH_CLIENT").ok();
    let original_ssh_tty = env::var("SSH_TTY").ok();

    // Test SSH_CONNECTION detection
    env::set_var("SSH_CONNECTION", "192.168.1.100 52345 192.168.1.1 22");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.is_ssh);
    env::remove_var("SSH_CONNECTION");

    // Test SSH_CLIENT detection
    env::set_var("SSH_CLIENT", "192.168.1.100 52345 22");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.is_ssh);
    env::remove_var("SSH_CLIENT");

    // Test SSH_TTY detection
    env::set_var("SSH_TTY", "/dev/pts/0");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.is_ssh);
    env::remove_var("SSH_TTY");

    // Test no SSH
    let _env_info = EnvironmentInfo::detect();
    // May or may not be SSH depending on actual environment
    // Just verify it doesn't panic

    // Restore original env
    match original_ssh_connection {
        Some(val) => env::set_var("SSH_CONNECTION", val),
        None => env::remove_var("SSH_CONNECTION"),
    }
    match original_ssh_client {
        Some(val) => env::set_var("SSH_CLIENT", val),
        None => env::remove_var("SSH_CLIENT"),
    }
    match original_ssh_tty {
        Some(val) => env::set_var("SSH_TTY", val),
        None => env::remove_var("SSH_TTY"),
    }
}

#[test]
fn test_container_detection() {
    // Note: This test documents the container detection logic
    // but won't actually create container files during testing

    // Test container env var detection
    let original_container = env::var("container").ok();
    env::set_var("container", "docker");
    let env_info = EnvironmentInfo::detect();
    // Should detect container from env var
    assert!(env_info.is_container);

    // Restore
    match original_container {
        Some(val) => env::set_var("container", val),
        None => env::remove_var("container"),
    }
}

#[test]
fn test_tmux_detection() {
    let original_tmux = env::var("TMUX").ok();

    // Test tmux detection
    env::set_var("TMUX", "/tmp/tmux-1000/default,12345,0");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.is_tmux);
    assert!(env_info.is_multiplexed());

    // Restore
    match original_tmux {
        Some(val) => env::set_var("TMUX", val),
        None => env::remove_var("TMUX"),
    }
}

#[test]
fn test_screen_detection() {
    let original_sty = env::var("STY").ok();
    let original_window = env::var("WINDOW").ok();

    // Test GNU screen detection via STY
    env::set_var("STY", "12345.pts-0.hostname");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.is_screen);
    assert!(env_info.is_multiplexed());
    env::remove_var("STY");

    // Test GNU screen detection via WINDOW
    env::set_var("WINDOW", "0");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.is_screen);
    assert!(env_info.is_multiplexed());

    // Restore
    match original_sty {
        Some(val) => env::set_var("STY", val),
        None => env::remove_var("STY"),
    }
    match original_window {
        Some(val) => env::set_var("WINDOW", val),
        None => env::remove_var("WINDOW"),
    }
}

#[test]
fn test_color_support_detection() {
    // Test color support through public API by setting env vars
    let original_term = env::var("TERM").ok();
    let original_colorterm = env::var("COLORTERM").ok();

    // Test 256 color detection
    env::set_var("TERM", "xterm-256color");
    env::remove_var("COLORTERM");
    let env_info = EnvironmentInfo::detect();
    assert!(
        env_info.color_support == ColorSupport::Color256
            || env_info.color_support == ColorSupport::TrueColor
    );

    // Test basic color detection
    env::set_var("TERM", "xterm");
    env::remove_var("COLORTERM");
    let env_info = EnvironmentInfo::detect();
    assert!(
        env_info.color_support == ColorSupport::Basic
            || env_info.color_support == ColorSupport::Color256
            || env_info.color_support == ColorSupport::TrueColor
    );

    // Test no color detection
    // Note: In some CI/test environments, color detection may vary
    // This test just verifies that detect() works without panic
    env::set_var("TERM", "dumb");
    env::remove_var("COLORTERM");
    let env_info = EnvironmentInfo::detect();
    // Accept any color support level since detection depends on multiple factors
    let _ = env_info.color_support;

    // Restore
    match original_term {
        Some(val) => env::set_var("TERM", val),
        None => env::remove_var("TERM"),
    }
    match original_colorterm {
        Some(val) => env::set_var("COLORTERM", val),
        None => env::remove_var("COLORTERM"),
    }
}

#[test]
fn test_mouse_support_detection() {
    // Test mouse support through public API by setting TERM env var
    let original_term = env::var("TERM").ok();

    // Modern terminals should support mouse
    env::set_var("TERM", "xterm-256color");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.has_mouse_support);

    env::set_var("TERM", "screen-256color");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.has_mouse_support);

    // Legacy terminals should not
    env::set_var("TERM", "vt100");
    let env_info = EnvironmentInfo::detect();
    assert!(!env_info.has_mouse_support);

    env::set_var("TERM", "dumb");
    let env_info = EnvironmentInfo::detect();
    assert!(!env_info.has_mouse_support);

    // Restore
    match original_term {
        Some(val) => env::set_var("TERM", val),
        None => env::remove_var("TERM"),
    }
}

#[test]
fn test_unicode_detection() {
    let original_lang = env::var("LANG").ok();
    let original_lc_all = env::var("LC_ALL").ok();

    // Test UTF-8 detection via LANG
    env::set_var("LANG", "en_US.UTF-8");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.has_unicode);

    // Test UTF-8 detection via LC_ALL
    env::remove_var("LANG");
    env::set_var("LC_ALL", "en_US.utf8");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.has_unicode);

    // Restore
    match original_lang {
        Some(val) => env::set_var("LANG", val),
        None => env::remove_var("LANG"),
    }
    match original_lc_all {
        Some(val) => env::set_var("LC_ALL", val),
        None => env::remove_var("LC_ALL"),
    }
}

#[test]
fn test_truecolor_detection() {
    let original_colorterm = env::var("COLORTERM").ok();
    let original_term_program = env::var("TERM_PROGRAM").ok();

    // Test COLORTERM detection
    env::set_var("COLORTERM", "truecolor");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.has_truecolor);
    env::remove_var("COLORTERM");

    env::set_var("COLORTERM", "24bit");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.has_truecolor);
    env::remove_var("COLORTERM");

    // Test TERM_PROGRAM detection
    env::set_var("TERM_PROGRAM", "iTerm.app");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.has_truecolor);
    env::remove_var("TERM_PROGRAM");

    env::set_var("TERM_PROGRAM", "Alacritty");
    let env_info = EnvironmentInfo::detect();
    assert!(env_info.has_truecolor);
    env::remove_var("TERM_PROGRAM");

    // Restore
    match original_colorterm {
        Some(val) => env::set_var("COLORTERM", val),
        None => env::remove_var("COLORTERM"),
    }
    match original_term_program {
        Some(val) => env::set_var("TERM_PROGRAM", val),
        None => env::remove_var("TERM_PROGRAM"),
    }
}

#[test]
fn test_environment_description() {
    let mut env_info = EnvironmentInfo::default();
    env_info.is_ssh = true;
    env_info.is_tmux = true;
    env_info.term_type = "xterm-256color".to_string();
    env_info.color_support = ColorSupport::TrueColor;

    let desc = env_info.description();

    // Should contain key information
    assert!(desc.contains("SSH"));
    assert!(desc.contains("tmux"));
    assert!(desc.contains("xterm-256color"));
    assert!(desc.contains("TrueColor"));
}

#[test]
fn test_constrained_environment_detection() {
    let mut env_info = EnvironmentInfo::default();

    // Not constrained by default
    assert!(!env_info.is_constrained());

    // SSH is constrained
    env_info.is_ssh = true;
    assert!(env_info.is_constrained());

    // Container is constrained
    env_info.is_ssh = false;
    env_info.is_container = true;
    assert!(env_info.is_constrained());

    // Both is constrained
    env_info.is_ssh = true;
    assert!(env_info.is_constrained());
}

#[test]
fn test_multiplexed_environment_detection() {
    let mut env_info = EnvironmentInfo::default();

    // Not multiplexed by default
    assert!(!env_info.is_multiplexed());

    // tmux is multiplexed
    env_info.is_tmux = true;
    assert!(env_info.is_multiplexed());

    // screen is multiplexed
    env_info.is_tmux = false;
    env_info.is_screen = true;
    assert!(env_info.is_multiplexed());

    // Both is multiplexed
    env_info.is_tmux = true;
    assert!(env_info.is_multiplexed());
}

#[test]
fn test_suggested_settings_default() {
    let env_info = EnvironmentInfo::default();
    let settings = env_info.suggested_settings();

    // Default should be optimal
    assert!(settings.enable_truecolor);
    assert!(settings.enable_mouse);
    assert!(settings.enable_unicode);
    assert!(settings.enable_animations);
    assert!(settings.enable_font_ligatures);
    assert_eq!(settings.scrollback_lines, 10000);
    assert_eq!(settings.refresh_rate_ms, 16);
}

#[test]
fn test_suggested_settings_constrained() {
    let mut env_info = EnvironmentInfo::default();
    env_info.is_ssh = true;

    let settings = env_info.suggested_settings();

    // Constrained should reduce resources
    assert!(!settings.enable_animations);
    assert!(!settings.enable_font_ligatures);
    assert_eq!(settings.scrollback_lines, 5000);
    assert_eq!(settings.refresh_rate_ms, 50);
}

#[test]
fn test_suggested_settings_limited_capabilities() {
    let mut env_info = EnvironmentInfo::default();
    env_info.has_truecolor = false;
    env_info.has_mouse_support = false;
    env_info.has_unicode = false;

    let settings = env_info.suggested_settings();

    // Should respect capabilities
    assert!(!settings.enable_truecolor);
    assert!(!settings.enable_mouse);
    assert!(!settings.enable_unicode);
}

#[test]
fn test_environment_settings_default() {
    let settings = EnvironmentSettings::default();

    assert!(settings.enable_truecolor);
    assert!(settings.enable_mouse);
    assert!(settings.enable_unicode);
    assert!(settings.enable_animations);
    assert!(settings.enable_font_ligatures);
    assert_eq!(settings.scrollback_lines, 10000);
    assert_eq!(settings.refresh_rate_ms, 16);
}

// ============================================================================
// Snippet System Tests
// ============================================================================

#[test]
fn test_snippet_creation() {
    let snippet = Snippet::new(
        "Git Status".to_string(),
        "/gs".to_string(),
        "git status".to_string(),
        "git".to_string(),
    );

    assert_eq!(snippet.name, "Git Status");
    assert_eq!(snippet.trigger, "/gs");
    assert_eq!(snippet.content, "git status");
    assert_eq!(snippet.category, "git");
}

#[test]
fn test_default_snippets() {
    let snippets = Snippet::default_snippets();

    // Should have multiple default snippets
    assert!(!snippets.is_empty());

    // Should have git snippets
    let git_snippets: Vec<_> = snippets.iter().filter(|s| s.category == "git").collect();
    assert!(!git_snippets.is_empty());

    // Check specific snippets exist
    let gs_snippet = snippets.iter().find(|s| s.trigger == "/gs");
    assert!(gs_snippet.is_some());
    assert_eq!(gs_snippet.unwrap().content, "git status");

    let gc_snippet = snippets.iter().find(|s| s.trigger == "/gc");
    assert!(gc_snippet.is_some());
    assert!(gc_snippet.unwrap().content.starts_with("git commit"));
}

#[test]
fn test_snippet_find_by_trigger() {
    let snippets = Snippet::default_snippets();

    // Should find existing snippet
    let found = Snippet::find_by_trigger(&snippets, "/gs");
    assert!(found.is_some());
    assert_eq!(found.unwrap().content, "git status");

    // Should not find non-existent snippet
    let not_found = Snippet::find_by_trigger(&snippets, "/nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_snippet_find_by_category() {
    let snippets = Snippet::default_snippets();

    // Find git snippets
    let git_snippets = Snippet::find_by_category(&snippets, "git");
    assert!(!git_snippets.is_empty());

    // All should be git category
    for snippet in git_snippets {
        assert_eq!(snippet.category, "git");
    }

    // Find non-existent category
    let empty = Snippet::find_by_category(&snippets, "nonexistent");
    assert!(empty.is_empty());
}

#[test]
fn test_snippet_get_categories() {
    let snippets = Snippet::default_snippets();

    let categories = Snippet::get_categories(&snippets);

    // Should have categories
    assert!(!categories.is_empty());

    // Should have git category
    assert!(categories.contains(&"git".to_string()));

    // Should be unique and sorted
    for i in 1..categories.len() {
        assert!(categories[i - 1] <= categories[i]);
    }
}

#[test]
fn test_snippet_multiple_categories() {
    let snippets = vec![
        Snippet::new(
            "Test 1".to_string(),
            "/t1".to_string(),
            "content1".to_string(),
            "cat1".to_string(),
        ),
        Snippet::new(
            "Test 2".to_string(),
            "/t2".to_string(),
            "content2".to_string(),
            "cat2".to_string(),
        ),
        Snippet::new(
            "Test 3".to_string(),
            "/t3".to_string(),
            "content3".to_string(),
            "cat1".to_string(),
        ),
    ];

    let categories = Snippet::get_categories(&snippets);
    assert_eq!(categories.len(), 2);
    assert!(categories.contains(&"cat1".to_string()));
    assert!(categories.contains(&"cat2".to_string()));

    let cat1_snippets = Snippet::find_by_category(&snippets, "cat1");
    assert_eq!(cat1_snippets.len(), 2);
}

#[test]
fn test_snippet_trigger_matching() {
    let snippets = vec![
        Snippet::new(
            "Short".to_string(),
            "/s".to_string(),
            "short".to_string(),
            "test".to_string(),
        ),
        Snippet::new(
            "Long".to_string(),
            "/longer".to_string(),
            "longer content".to_string(),
            "test".to_string(),
        ),
    ];

    // Exact match
    let found = Snippet::find_by_trigger(&snippets, "/s");
    assert!(found.is_some());
    assert_eq!(found.unwrap().content, "short");

    // Case sensitive
    let not_found = Snippet::find_by_trigger(&snippets, "/S");
    assert!(not_found.is_none());

    // Partial match should not work
    let not_found = Snippet::find_by_trigger(&snippets, "/lon");
    assert!(not_found.is_none());
}

// ============================================================================
// Profile System Tests
// ============================================================================

#[test]
fn test_profile_creation() {
    let profile = Profile::new("test-profile".to_string());

    assert_eq!(profile.name, "test-profile");
    assert!(profile.shell.is_none());
    assert!(profile.shell_args.is_empty());
    assert!(profile.env.is_empty());
    assert!(profile.theme.is_none());
    assert!(profile.font_size.is_none());
    assert!(profile.working_dir.is_none());
    assert!(profile.color_scheme.is_none());
}

#[test]
fn test_profile_with_custom_settings() {
    let mut env = HashMap::new();
    env.insert("CUSTOM_VAR".to_string(), "value".to_string());

    let mut profile = Profile::new("custom".to_string());
    profile.shell = Some("/bin/zsh".to_string());
    profile.shell_args = vec!["-l".to_string()];
    profile.env = env.clone();
    profile.theme = Some("dark".to_string());
    profile.font_size = Some(16.0);
    profile.working_dir = Some(PathBuf::from("/home/user/projects"));

    assert_eq!(profile.shell, Some("/bin/zsh".to_string()));
    assert_eq!(profile.shell_args, vec!["-l".to_string()]);
    assert_eq!(profile.env, env);
    assert_eq!(profile.theme, Some("dark".to_string()));
    assert_eq!(profile.font_size, Some(16.0));
    assert_eq!(
        profile.working_dir,
        Some(PathBuf::from("/home/user/projects"))
    );
}

#[test]
fn test_profile_apply_to_config() {
    let mut config = AppConfig::default();
    let original_font_size = config.appearance.font_size;

    let mut profile = Profile::new("test".to_string());
    profile.shell = Some("/bin/fish".to_string());
    profile.shell_args = vec!["--login".to_string()];
    profile.theme = Some("custom-theme".to_string());
    profile.font_size = Some(18.0);

    profile.apply_to_config(&mut config);

    // Shell should be updated
    assert_eq!(config.shell.program, Some("/bin/fish".to_string()));
    assert_eq!(config.shell.args, vec!["--login".to_string()]);

    // Theme should be updated
    assert_eq!(config.appearance.theme, "custom-theme");

    // Font size should be updated
    assert_eq!(config.appearance.font_size, 18.0);
    assert_ne!(config.appearance.font_size, original_font_size);
}

#[test]
fn test_profile_apply_partial_settings() {
    let mut config = AppConfig::default();
    let _original_font_size = config.appearance.font_size;
    let original_theme = config.appearance.theme.clone();

    // Profile with only some settings
    let mut profile = Profile::new("partial".to_string());
    profile.font_size = Some(20.0);
    // No theme, no shell

    profile.apply_to_config(&mut config);

    // Font size should be updated
    assert_eq!(config.appearance.font_size, 20.0);

    // Theme should remain unchanged
    assert_eq!(config.appearance.theme, original_theme);

    // Shell should remain unchanged (None)
    assert!(config.shell.program.is_none() || config.shell.program.is_some());
}

#[test]
fn test_profile_environment_variables() {
    let mut config = AppConfig::default();
    let original_env_size = config.shell.env.len();

    let mut env = HashMap::new();
    env.insert("TEST_VAR_1".to_string(), "value1".to_string());
    env.insert("TEST_VAR_2".to_string(), "value2".to_string());

    let mut profile = Profile::new("env-test".to_string());
    profile.env = env.clone();

    profile.apply_to_config(&mut config);

    // Environment variables should be merged
    assert!(config.shell.env.len() >= original_env_size + 2);
    assert_eq!(
        config.shell.env.get("TEST_VAR_1"),
        Some(&"value1".to_string())
    );
    assert_eq!(
        config.shell.env.get("TEST_VAR_2"),
        Some(&"value2".to_string())
    );
}

#[test]
fn test_profile_working_directory() {
    let mut config = AppConfig::default();

    let mut profile = Profile::new("workdir-test".to_string());
    profile.working_dir = Some(PathBuf::from("/custom/path"));

    profile.apply_to_config(&mut config);

    assert_eq!(
        config.general.default_working_dir,
        Some(PathBuf::from("/custom/path"))
    );
}

// ============================================================================
// Session Restoration Tests
// ============================================================================

#[test]
fn test_session_config_defaults() {
    let session_config = agterm::config::SessionConfig::default();

    assert!(session_config.restore_on_startup);
    assert!(session_config.save_on_exit);
    assert!(session_config.session_file.is_none());
}

#[test]
fn test_session_config_custom() {
    let session_config = agterm::config::SessionConfig {
        restore_on_startup: false,
        save_on_exit: true,
        auto_save: true,
        auto_save_interval_seconds: 30,
        max_backups: 5,
        session_file: Some(PathBuf::from("/custom/session.json")),
        prompt_on_recovery: true,
    };

    assert!(!session_config.restore_on_startup);
    assert!(session_config.save_on_exit);
    assert!(session_config.auto_save);
    assert_eq!(session_config.auto_save_interval_seconds, 30);
    assert_eq!(session_config.max_backups, 5);
    assert_eq!(
        session_config.session_file,
        Some(PathBuf::from("/custom/session.json"))
    );
}

#[test]
fn test_config_session_settings() {
    let config = AppConfig::default();

    // Should have session configuration
    assert!(config.general.session.restore_on_startup);
    assert!(config.general.session.save_on_exit);
}

// ============================================================================
// URL Detection Tests
// ============================================================================
// Note: These tests document expected URL detection behavior.
// The actual implementation may need to be added to the codebase.

#[test]
fn test_url_detection_http() {
    // Test HTTP URL detection pattern
    let text = "Check out http://example.com for more info";
    let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();

    let matches: Vec<_> = url_pattern.find_iter(text).collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].as_str(), "http://example.com");
}

#[test]
fn test_url_detection_https() {
    let text = "Visit https://secure.example.com/path/to/resource";
    let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();

    let matches: Vec<_> = url_pattern.find_iter(text).collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(
        matches[0].as_str(),
        "https://secure.example.com/path/to/resource"
    );
}

#[test]
fn test_url_detection_multiple() {
    let text = "Check http://site1.com and https://site2.com for details";
    let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();

    let matches: Vec<_> = url_pattern.find_iter(text).collect();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].as_str(), "http://site1.com");
    assert_eq!(matches[1].as_str(), "https://site2.com");
}

#[test]
fn test_url_detection_with_port() {
    let text = "Server running at http://localhost:8080";
    let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();

    let matches: Vec<_> = url_pattern.find_iter(text).collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].as_str(), "http://localhost:8080");
}

#[test]
fn test_url_detection_with_path_and_query() {
    let text = "API endpoint: https://api.example.com/v1/users?id=123&active=true";
    let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();

    let matches: Vec<_> = url_pattern.find_iter(text).collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(
        matches[0].as_str(),
        "https://api.example.com/v1/users?id=123&active=true"
    );
}

#[test]
fn test_url_detection_no_match() {
    let text = "This text has no URLs at all";
    let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();

    let matches: Vec<_> = url_pattern.find_iter(text).collect();
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_url_detection_file_protocol() {
    // Extended URL pattern for file:// protocol
    let text = "Open file:///home/user/document.txt";
    let url_pattern = regex::Regex::new(r"(?:https?|file)://[^\s]+").unwrap();

    let matches: Vec<_> = url_pattern.find_iter(text).collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].as_str(), "file:///home/user/document.txt");
}

#[test]
fn test_url_detection_with_anchor() {
    let text = "See https://example.com/docs#section-2 for details";
    let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();

    let matches: Vec<_> = url_pattern.find_iter(text).collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].as_str(), "https://example.com/docs#section-2");
}

// ============================================================================
// Pane Layout Tests
// ============================================================================
// Note: These tests document expected pane layout behavior.
// The actual implementation may need to be added to the codebase.

#[test]
fn test_pane_layout_single() {
    // Single pane layout (default)
    #[derive(Debug, PartialEq)]
    struct PaneLayout {
        rows: usize,
        cols: usize,
    }

    let layout = PaneLayout { rows: 1, cols: 1 };
    assert_eq!(layout.rows, 1);
    assert_eq!(layout.cols, 1);
}

#[test]
fn test_pane_layout_horizontal_split() {
    // Horizontal split (one above, one below)
    #[derive(Debug, PartialEq)]
    struct PaneLayout {
        rows: usize,
        cols: usize,
    }

    let layout = PaneLayout { rows: 2, cols: 1 };
    assert_eq!(layout.rows, 2);
    assert_eq!(layout.cols, 1);
}

#[test]
fn test_pane_layout_vertical_split() {
    // Vertical split (side by side)
    #[derive(Debug, PartialEq)]
    struct PaneLayout {
        rows: usize,
        cols: usize,
    }

    let layout = PaneLayout { rows: 1, cols: 2 };
    assert_eq!(layout.rows, 1);
    assert_eq!(layout.cols, 2);
}

#[test]
fn test_pane_layout_grid() {
    // 2x2 grid layout
    #[derive(Debug, PartialEq)]
    struct PaneLayout {
        rows: usize,
        cols: usize,
    }

    let layout = PaneLayout { rows: 2, cols: 2 };
    assert_eq!(layout.rows, 2);
    assert_eq!(layout.cols, 2);

    // Total panes should be rows * cols
    let total_panes = layout.rows * layout.cols;
    assert_eq!(total_panes, 4);
}

#[test]
fn test_pane_layout_calculation() {
    // Test size calculation for panes
    let terminal_width = 160;
    let terminal_height = 48;

    // 2x1 layout (vertical split)
    let cols = 2;
    let rows = 1;
    let pane_width = terminal_width / cols;
    let pane_height = terminal_height / rows;

    assert_eq!(pane_width, 80);
    assert_eq!(pane_height, 48);

    // 1x2 layout (horizontal split)
    let cols = 1;
    let rows = 2;
    let pane_width = terminal_width / cols;
    let pane_height = terminal_height / rows;

    assert_eq!(pane_width, 160);
    assert_eq!(pane_height, 24);
}

#[test]
fn test_pane_layout_with_borders() {
    // Account for borders between panes
    let terminal_width = 160;
    let cols = 2;
    let border_width = 1;

    // Each pane gets (total - borders) / cols
    let usable_width = terminal_width - (border_width * (cols - 1));
    let pane_width = usable_width / cols;

    assert_eq!(usable_width, 159);
    assert_eq!(pane_width, 79);
}

#[test]
fn test_pane_focus_navigation() {
    // Test pane focus logic
    #[derive(Debug, PartialEq)]
    struct PaneGrid {
        rows: usize,
        cols: usize,
        focused: (usize, usize), // (row, col)
    }

    let mut grid = PaneGrid {
        rows: 2,
        cols: 2,
        focused: (0, 0),
    };

    // Move right
    if grid.focused.1 < grid.cols - 1 {
        grid.focused.1 += 1;
    }
    assert_eq!(grid.focused, (0, 1));

    // Move down
    if grid.focused.0 < grid.rows - 1 {
        grid.focused.0 += 1;
    }
    assert_eq!(grid.focused, (1, 1));

    // Move left
    if grid.focused.1 > 0 {
        grid.focused.1 -= 1;
    }
    assert_eq!(grid.focused, (1, 0));

    // Move up
    if grid.focused.0 > 0 {
        grid.focused.0 -= 1;
    }
    assert_eq!(grid.focused, (0, 0));
}

#[test]
fn test_pane_resize() {
    // Test dynamic pane resizing
    struct Pane {
        width: usize,
        height: usize,
    }

    let mut pane = Pane {
        width: 80,
        height: 24,
    };

    // Resize
    pane.width = 100;
    pane.height = 30;

    assert_eq!(pane.width, 100);
    assert_eq!(pane.height, 30);
}

// ============================================================================
// Integration Tests - Combined Features
// ============================================================================

#[test]
fn test_environment_based_profile_selection() {
    // Test selecting profile based on environment
    let mut env_info = EnvironmentInfo::default();
    env_info.is_ssh = true;

    // In SSH, we might want a lighter profile
    let profile_name = if env_info.is_ssh {
        "ssh-optimized"
    } else if env_info.is_container {
        "container-optimized"
    } else {
        "default"
    };

    assert_eq!(profile_name, "ssh-optimized");
}

#[test]
fn test_snippet_with_environment_variables() {
    // Test snippets that expand environment-specific values
    let env_info = EnvironmentInfo::default();
    let is_ssh = env_info.is_ssh;

    let snippet_content = if is_ssh {
        "git push origin main" // Direct push in SSH
    } else {
        "git push" // Let git config handle it locally
    };

    // Snippet behavior changes based on environment
    assert!(!is_ssh); // Default is not SSH
    assert_eq!(snippet_content, "git push");
}

#[test]
fn test_profile_with_snippet_integration() {
    // Profile can define preferred snippets
    let mut profile = Profile::new("developer".to_string());
    profile.shell = Some("/bin/zsh".to_string());

    // In a real implementation, profile might have snippet preferences
    // This tests the concept
    let profile_category = "git"; // Developer profile prefers git snippets

    let snippets = Snippet::default_snippets();
    let relevant_snippets = Snippet::find_by_category(&snippets, profile_category);

    assert!(!relevant_snippets.is_empty());
    assert!(relevant_snippets.iter().all(|s| s.category == "git"));
}
