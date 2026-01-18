# Environment Detection and Adaptation

AgTerm includes comprehensive environment detection to automatically optimize terminal behavior based on the runtime environment.

## Features

### Detected Environments

#### SSH Sessions
- Detects SSH connections via `SSH_CONNECTION`, `SSH_CLIENT`, or `SSH_TTY` environment variables
- Reduces refresh rate for better network performance
- Disables animations to reduce bandwidth usage

#### Containers
Detects when running inside containers:
- Docker (`/.dockerenv`)
- Podman (`/run/.containerenv`)
- Kubernetes and other OCI containers (via cgroup inspection)
- Adjusts settings for constrained resources

#### Terminal Multiplexers
- **tmux**: Detected via `TMUX` environment variable
- **GNU screen**: Detected via `STY` or `WINDOW` environment variables
- Shows multiplexer indicator in status bar

### Terminal Capabilities

#### Color Support
Detects and adapts to different color capabilities:
- **True Color (24-bit)**: Full RGB color support
- **256 Colors**: Extended color palette
- **Basic (16 colors)**: Standard ANSI colors
- **No Color**: Fallback for limited terminals

Detection methods:
- `COLORTERM` environment variable (truecolor, 24bit)
- `TERM` value analysis (256color, xterm, etc.)
- Known terminal programs (iTerm, WezTerm, Alacritty, kitty)

#### Input Capabilities
- **Mouse Support**: Detected based on TERM type
  - Enabled for modern terminals (xterm, screen, tmux variants)
  - Disabled for legacy terminals (vt100, vt220, dumb)

- **Unicode/UTF-8**: Detected via:
  - `LANG` environment variable
  - `LC_ALL` and `LC_CTYPE` locales
  - `TERM` UTF-8 indicators

## Adaptive Behavior

### Refresh Rate Adjustment

AgTerm dynamically adjusts its refresh rate based on:

1. **Environment Type**:
   - Local: 16ms (60 FPS)
   - SSH/Container: 50ms (20 FPS)

2. **PTY Activity**:
   - Recent activity (< 500ms): Full speed
   - Medium activity (< 2s): 3x slower
   - Idle: 12x slower

This reduces CPU and network usage while maintaining responsiveness.

### Resource Optimization

For constrained environments (SSH/Container):
- Reduced scrollback buffer (5000 vs 10000 lines)
- Disabled animations
- Disabled font ligatures
- Lower refresh rates

### Visual Indicators

The status bar displays environment indicators:
- **SSH**: Yellow badge when in SSH session
- **Container**: Blue badge when in container
- **tmux/screen**: Gray badge for terminal multiplexers

## API Usage

### Detecting Environment

```rust
use agterm::terminal::env::EnvironmentInfo;

// Detect current environment
let env_info = EnvironmentInfo::detect();

// Check specific conditions
if env_info.is_ssh {
    println!("Running over SSH");
}

if env_info.is_container {
    println!("Running in container");
}

// Get human-readable description
println!("Environment: {}", env_info.description());
```

### Getting Suggested Settings

```rust
let suggested = env_info.suggested_settings();

if suggested.enable_truecolor {
    // Use 24-bit colors
}

if suggested.enable_mouse {
    // Enable mouse support
}

println!("Recommended refresh rate: {}ms", suggested.refresh_rate_ms);
println!("Recommended scrollback: {} lines", suggested.scrollback_lines);
```

### Classification Methods

```rust
// Check if environment is constrained (SSH/container)
if env_info.is_constrained() {
    // Apply performance optimizations
}

// Check if running in terminal multiplexer
if env_info.is_multiplexed() {
    // Adjust for tmux/screen quirks
}
```

## Environment Variables Reference

### Terminal Type
- `TERM`: Terminal type (e.g., xterm-256color, screen-256color)
- `COLORTERM`: Color capabilities (truecolor, 24bit)
- `TERM_PROGRAM`: Terminal emulator name

### Session Detection
- `SSH_CONNECTION`: SSH client/server connection details
- `SSH_CLIENT`: SSH client information
- `SSH_TTY`: SSH TTY device
- `TMUX`: tmux session information
- `STY`: GNU screen session name
- `WINDOW`: GNU screen window number
- `container`: Container environment indicator

### Localization
- `LANG`: System language and encoding
- `LC_ALL`: Override for all locale categories
- `LC_CTYPE`: Character classification and case conversion

## Testing

Run the environment detection demo:

```bash
cargo run --example env_detection_demo
```

This shows:
- Detected environment properties
- Terminal capabilities
- Suggested settings
- Current environment variables

### Simulating Environments

Test SSH environment:
```bash
SSH_CONNECTION="1.2.3.4 1234 5.6.7.8 22" cargo run --example env_detection_demo
```

Test container environment:
```bash
# Create temporary .dockerenv
touch /.dockerenv
cargo run --example env_detection_demo
sudo rm /.dockerenv
```

Test tmux environment:
```bash
TMUX="/tmp/tmux-1000/default,1234,0" cargo run --example env_detection_demo
```

## Implementation Details

### Detection Methods

1. **File System Checks**:
   - `/.dockerenv`: Docker container marker
   - `/run/.containerenv`: Podman container marker
   - `/proc/self/cgroup`: Container runtime detection

2. **Environment Variables**:
   - Direct detection via well-known variables
   - Pattern matching in TERM values
   - Locale string parsing

3. **Heuristics**:
   - Terminal program database (iTerm, Alacritty, etc.)
   - TERM prefix matching (xterm-, screen-, tmux-)
   - Default assumptions for common scenarios

### Performance Considerations

Environment detection runs once at startup:
- No runtime overhead
- Results cached in `AgTerm::env_info`
- Settings recomputed on demand

File system checks are minimal:
- Only checks 2-3 well-known paths
- Falls back to environment variables if unavailable
- No recursive directory scanning

## Future Enhancements

Potential improvements:
- Terminal capability queries (XTVERSION, DA3)
- Runtime environment re-detection
- User-configurable detection overrides
- Platform-specific optimizations (Windows, WSL)
- Network latency measurement for adaptive refresh
