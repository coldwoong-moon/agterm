//! Demonstration of environment detection capabilities
//!
//! Run with: cargo run --example env_detection_demo

use agterm::terminal::env::EnvironmentInfo;

fn main() {
    println!("=== AgTerm Environment Detection Demo ===\n");

    // Detect current environment
    let env_info = EnvironmentInfo::detect();

    println!("Environment Summary:");
    println!("  {}\n", env_info.description());

    println!("Detailed Environment Information:");
    println!("  SSH Session:      {}", if env_info.is_ssh { "Yes" } else { "No" });
    println!("  Container:        {}", if env_info.is_container { "Yes" } else { "No" });
    println!("  tmux:             {}", if env_info.is_tmux { "Yes" } else { "No" });
    println!("  GNU screen:       {}", if env_info.is_screen { "Yes" } else { "No" });
    println!("  Terminal Type:    {}", env_info.term_type);
    println!("  Color Support:    {:?}", env_info.color_support);
    println!("  True Color:       {}", if env_info.has_truecolor { "Yes" } else { "No" });
    println!("  Mouse Support:    {}", if env_info.has_mouse_support { "Yes" } else { "No" });
    println!("  Unicode Support:  {}\n", if env_info.has_unicode { "Yes" } else { "No" });

    println!("Environment Classification:");
    println!("  Constrained:      {}", if env_info.is_constrained() { "Yes (SSH/Container)" } else { "No" });
    println!("  Multiplexed:      {}\n", if env_info.is_multiplexed() { "Yes (tmux/screen)" } else { "No" });

    // Get suggested settings
    let settings = env_info.suggested_settings();

    println!("Suggested Settings:");
    println!("  True Color:       {}", if settings.enable_truecolor { "Enabled" } else { "Disabled" });
    println!("  Mouse:            {}", if settings.enable_mouse { "Enabled" } else { "Disabled" });
    println!("  Unicode:          {}", if settings.enable_unicode { "Enabled" } else { "Disabled" });
    println!("  Animations:       {}", if settings.enable_animations { "Enabled" } else { "Disabled" });
    println!("  Font Ligatures:   {}", if settings.enable_font_ligatures { "Enabled" } else { "Disabled" });
    println!("  Scrollback Lines: {}", settings.scrollback_lines);
    println!("  Refresh Rate:     {}ms", settings.refresh_rate_ms);

    println!("\n=== Environment Variables ===");
    println!("TERM:             {}", std::env::var("TERM").unwrap_or_else(|_| "not set".to_string()));
    println!("COLORTERM:        {}", std::env::var("COLORTERM").unwrap_or_else(|_| "not set".to_string()));
    println!("TERM_PROGRAM:     {}", std::env::var("TERM_PROGRAM").unwrap_or_else(|_| "not set".to_string()));
    println!("SSH_CONNECTION:   {}", std::env::var("SSH_CONNECTION").unwrap_or_else(|_| "not set".to_string()));
    println!("TMUX:             {}", std::env::var("TMUX").unwrap_or_else(|_| "not set".to_string()));
    println!("STY:              {}", std::env::var("STY").unwrap_or_else(|_| "not set".to_string()));
    println!("LANG:             {}", std::env::var("LANG").unwrap_or_else(|_| "not set".to_string()));
}
