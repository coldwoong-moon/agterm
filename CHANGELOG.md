# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure and architecture
- Core domain models (Task, Session, Memory)
- PTY pool with multi-session support
- MCP client integration with rmcp 0.13
- SQLite storage with FTS5 full-text search
- TUI framework with ratatui
- Theme system with customizable colors
- Keybinding system (vim, emacs, arrows)
- Task graph visualization
- Archive browser with search
- Session archiving and compaction
- CI/CD pipeline with GitHub Actions

## [0.1.0-alpha.1] - 2026-01-17

### Added
- Project initialization
- Cargo.toml with all dependencies
- Basic directory structure
- Error handling types
- Configuration loading system
- Logging setup with tracing

### Phase 1: Foundation
- Single PTY session support
- ANSI escape sequence parsing with vte
- Basic terminal I/O
- PTY pool management

### Phase 2: Multi-Terminal
- Terminal splitting (horizontal/vertical)
- Focus navigation (vim-style hjkl)
- Layout management

### Phase 3: Task Graph
- TaskGraph with petgraph
- Dependency-aware scheduling
- Task tree widget
- Error propagation policies

### Phase 4: Visualization
- Graph view widget
- Real-time progress display
- Task detail popup
- Timer and ETA calculation

### Phase 5: MCP Integration
- MCP client wrapper (rmcp 0.13)
- Server connection management
- Tool call helpers
- MCP status panel

### Phase 6: Archiving
- SQLite storage backend
- Schema migrations
- Archive repository with FTS5
- Compaction pipeline
- AI summarization prompts
- Archive browser widget

### Phase 7: Stabilization
- Theme customization system
- Keybinding configuration
- Default config file
- CI/CD pipelines
- README documentation

### Technical Details
- 96+ unit tests passing
- Cross-platform support (macOS, Linux, Windows)
- Memory-safe Rust implementation
- WAL mode SQLite for performance
