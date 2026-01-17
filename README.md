# AgTerm

**AI Agent Terminal Orchestrator with MCP Support**

AgTerm is a next-generation terminal multiplexer designed for AI agent workflows. It provides tree-based task management, real-time visualization, intelligent archiving, and native Model Context Protocol (MCP) integration.

## Features

- **Tree-based Task Management**: Organize tasks hierarchically with dependency tracking
- **Real-time Visualization**: Monitor task progress with live updates and graph views
- **Intelligent Archiving**: AI-powered session summarization and full-text search
- **MCP Native**: First-class support for Model Context Protocol servers
- **Multi-Terminal**: Split and manage multiple terminal sessions
- **Customizable**: Themes, keybindings, and extensive configuration options

## Installation

### From Binary

Download the latest release for your platform:

```bash
# macOS (Apple Silicon)
curl -L -o agterm https://github.com/user/agterm/releases/latest/download/agterm-macos-arm64
chmod +x agterm
sudo mv agterm /usr/local/bin/

# macOS (Intel)
curl -L -o agterm https://github.com/user/agterm/releases/latest/download/agterm-macos-amd64
chmod +x agterm
sudo mv agterm /usr/local/bin/

# Linux (x86_64)
curl -L -o agterm https://github.com/user/agterm/releases/latest/download/agterm-linux-amd64
chmod +x agterm
sudo mv agterm /usr/local/bin/
```

### From Source

```bash
# Clone the repository
git clone https://github.com/user/agterm.git
cd agterm

# Build and install
cargo install --path .
```

### Requirements

- Rust 1.75 or later (for building from source)
- A terminal emulator with 256-color support
- SQLite 3.x (bundled)

## Quick Start

```bash
# Start AgTerm
agterm

# Start with a specific working directory
agterm --working-dir ~/projects/myproject

# Start with a custom config file
agterm --config ~/.config/agterm/custom.toml
```

## Keyboard Shortcuts

AgTerm supports multiple keybinding styles: `vim` (default), `emacs`, and `arrows`.

### Vim Style (Default)

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate down / up |
| `h` / `l` | Navigate left / right |
| `g` / `G` | Go to first / last |
| `Ctrl+u` / `Ctrl+d` | Page up / down |
| `Tab` | Focus next pane |
| `?` / `F1` | Help |
| `F3` | Split vertical |
| `F4` | Graph view |
| `F6` | Archive browser |
| `/` | Search |
| `a` | Add task |
| `d` | Delete task |
| `r` | Retry task |
| `Ctrl+c` | Cancel task |
| `Ctrl+q` | Quit |

### Views

- **Task Tree** (default): Hierarchical view of all tasks
- **Graph View** (`F4`): Visual dependency graph with progress
- **Archive Browser** (`F6`): Browse and search past sessions
- **MCP Panel** (`F5`): MCP server status and tools

## Configuration

AgTerm looks for configuration in these locations (in order):

1. `/etc/agterm/config.toml` (system-wide)
2. `~/.config/agterm/config.toml` (user)
3. `.agterm/config.toml` (project-local)
4. Environment variables (`AGTERM_*`)

### Example Configuration

```toml
[general]
default_shell = "/bin/zsh"

[pty]
max_sessions = 32
scrollback_lines = 10000

[tui]
theme = "dark"          # default, dark, light, monokai
keybindings = "vim"     # vim, emacs, arrows
mouse_support = true
target_fps = 60

[storage]
compression_level = "compacted"
ai_summarization = false

[[mcp.servers]]
name = "filesystem"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
auto_connect = true
```

### Environment Variables

All configuration options can be overridden via environment variables:

```bash
export AGTERM__TUI__THEME="dark"
export AGTERM__PTY__MAX_SESSIONS="16"
export AGTERM__LOGGING__LEVEL="debug"
```

## Themes

AgTerm includes several built-in themes:

- **default**: Terminal default colors
- **dark**: High-contrast dark theme (One Dark inspired)
- **light**: Light theme for bright environments
- **monokai**: Classic Monokai color scheme

Custom themes can be defined in your config file:

```toml
[tui.theme]
name = "custom"
[tui.theme.colors]
primary = "#61afef"
secondary = "#56b6c2"
success = "#98c379"
error = "#e06c75"
```

## MCP Integration

AgTerm can connect to MCP servers as a client, enabling AI agents to use external tools.

### Configuring MCP Servers

```toml
[[mcp.servers]]
name = "github"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
auto_connect = true
[mcp.servers.env]
GITHUB_TOKEN = "${GITHUB_TOKEN}"
```

### Available Transports

- **stdio**: Spawn a subprocess (recommended)
- **sse**: Connect via Server-Sent Events (HTTP)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Presentation Layer                    │
│  TUI (ratatui) │ Widgets │ Theme │ Keybindings          │
├─────────────────────────────────────────────────────────┤
│                    Application Layer                     │
│  Config │ State │ Event Loop                            │
├─────────────────────────────────────────────────────────┤
│                      Domain Layer                        │
│  Task Graph │ Session │ Memory │ Events                  │
├─────────────────────────────────────────────────────────┤
│                   Infrastructure Layer                   │
│  PTY Pool │ MCP Client │ SQLite Storage                  │
└─────────────────────────────────────────────────────────┘
```

## Context Engineering

AgTerm implements advanced context management strategies:

1. **WRITE**: Store outputs to filesystem and SQLite
2. **SELECT**: Dynamic retrieval of relevant past sessions
3. **COMPRESS**: Progressive summarization pipeline
4. **ISOLATE**: Separate contexts for parallel tasks

### Compression Pipeline

```
Raw Output → Compaction (reversible) → AI Summary (lossy) → Rolling Summary
```

## Development

```bash
# Run tests
cargo test

# Run with debug logging
AGTERM__LOGGING__LEVEL=debug cargo run

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-targets
```

## License

MIT License. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests.

## Credits

Built with:
- [ratatui](https://ratatui.rs/) - TUI framework
- [portable-pty](https://docs.rs/portable-pty) - Cross-platform PTY
- [rmcp](https://github.com/anthropics/rmcp) - MCP SDK
- [petgraph](https://github.com/petgraph/petgraph) - Graph data structure
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings

Inspired by [Zellij](https://zellij.dev/), [tmux](https://github.com/tmux/tmux), and [Claude Code](https://claude.ai/code).
