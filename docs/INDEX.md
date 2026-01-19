# AgTerm Documentation Index

Complete guide to AgTerm documentation structure.

## Quick Links

### For Users
- [README.md](../README.md) - Main project overview and features
- [FLOEM_GUI.md](FLOEM_GUI.md) - Floem GUI user guide
- [QUICK_START.md](QUICK_START.md) - Getting started quickly

### For Developers
- [API_DOCUMENTATION.md](API_DOCUMENTATION.md) - Core API reference
- [FLOEM_IMPLEMENTATION_NOTES.md](FLOEM_IMPLEMENTATION_NOTES.md) - Technical implementation details
- [FLOEM_DEVELOPMENT_PHASES.md](FLOEM_DEVELOPMENT_PHASES.md) - Development history

### Features and Guides
- [THEMES.md](THEMES.md) - Theme system and customization
- [SESSION_RESTORATION.md](SESSION_RESTORATION.md) - Session persistence
- [ENVIRONMENT_DETECTION.md](ENVIRONMENT_DETECTION.md) - Environment detection
- [SHELL_INTEGRATION.md](../../SHELL_INTEGRATION.md) - Shell integration setup

### Advanced Features
- [RECORDING.md](RECORDING.md) - Terminal recording and playback
- [AUTOMATION.md](AUTOMATION.md) - Automation and scripting
- [AUTOMATION_DSL_REFERENCE.md](AUTOMATION_DSL_REFERENCE.md) - DSL reference
- [DIFF_VIEWER.md](DIFF_VIEWER.md) - Diff viewer features
- [HOOKS.md](HOOKS.md) - Hook system
- [LINK_HANDLER.md](LINK_HANDLER.md) - Link handling

### Reference
- [QUICK_START.md](QUICK_START.md) - Quick start guide
- [MENU_QUICKSTART.md](MENU_QUICKSTART.md) - Menu system quickstart
- [RECORDING_QUICKREF.md](RECORDING_QUICKREF.md) - Recording quick reference
- [AUTOMATION_QUICKSTART.md](AUTOMATION_QUICKSTART.md) - Automation quickstart

### Benchmarks
- [BENCHMARKS.md](BENCHMARKS.md) - Performance benchmarks

## GUI Implementations

AgTerm offers two GUI implementations:

### Iced GUI (Default)
Build with: `cargo build`

- Traditional tab-based interface
- Comprehensive feature set
- Stable and well-tested

### Floem GUI (Alternative)
Build with: `cargo build --bin agterm-floem --features floem-gui --no-default-features`

- Reactive, modern interface
- Pane-based layout system
- GPU-accelerated rendering
- See [FLOEM_GUI.md](FLOEM_GUI.md) for complete guide

## Documentation Hierarchy

```
docs/
├── INDEX.md                              # This file
├── README.md                             # Documentation overview
├── QUICK_START.md                        # Getting started
├── API_DOCUMENTATION.md                  # Core API reference
│
├── FLOEM_GUI.md                          # Floem GUI user guide
├── FLOEM_IMPLEMENTATION_NOTES.md         # Technical implementation
├── FLOEM_DEVELOPMENT_PHASES.md           # Development history
│
├── THEMES.md                             # Theme system
├── SESSION_RESTORATION.md                # Session management
├── ENVIRONMENT_DETECTION.md              # Environment detection
│
├── RECORDING.md                          # Recording feature
├── RECORDING_ARCHITECTURE.md             # Recording internals
├── RECORDING_UI_INTEGRATION.md           # Recording UI
├── RECORDING_QUICKREF.md                 # Recording quick reference
│
├── AUTOMATION.md                         # Automation system
├── AUTOMATION_DSL_REFERENCE.md           # DSL syntax reference
├── AUTOMATION_QUICKSTART.md              # Automation quickstart
│
├── DIFF_VIEWER.md                        # Diff viewer
├── LINK_HANDLER.md                       # Link handling
├── HOOKS.md                              # Hook system
├── HOOKS_QUICKSTART.md                   # Hook quickstart
│
├── MENU_QUICKSTART.md                    # Menu system guide
├── BENCHMARKS.md                         # Performance data
└── (archived implementation docs)        # Old phase documentation
```

## Document Types

### User Guides
- **Purpose**: Help users accomplish tasks
- **Audience**: End users
- **Examples**: QUICK_START.md, FLOEM_GUI.md, THEMES.md

### API Documentation
- **Purpose**: Document code interfaces
- **Audience**: Developers integrating AgTerm
- **Examples**: API_DOCUMENTATION.md

### Implementation Notes
- **Purpose**: Explain technical details
- **Audience**: Developers working on codebase
- **Examples**: FLOEM_IMPLEMENTATION_NOTES.md, RECORDING_ARCHITECTURE.md

### Quick References
- **Purpose**: Provide quick lookup
- **Audience**: Users who know the basics
- **Examples**: RECORDING_QUICKREF.md, AUTOMATION_QUICKSTART.md

### Architecture Documents
- **Purpose**: Explain system design
- **Audience**: Developers maintaining subsystems
- **Examples**: FLOEM_DEVELOPMENT_PHASES.md

## Building and Running

### Default (Iced GUI)
```bash
cargo build
cargo run --release
```

### Floem GUI (Alternative)
```bash
cargo build --bin agterm-floem --features floem-gui --no-default-features
cargo run --bin agterm-floem --features floem-gui --no-default-features
```

See [QUICK_START.md](QUICK_START.md) for detailed instructions.

## Contributing

When adding new features or documentation:

1. **User-facing docs**: Update [FLOEM_GUI.md](FLOEM_GUI.md) or [README.md](../README.md)
2. **API changes**: Update [API_DOCUMENTATION.md](API_DOCUMENTATION.md)
3. **Implementation details**: Update relevant architecture document
4. **Quick reference**: Add entry to appropriate quickstart guide

## Documentation Conventions

### Headers
- H1 (`#`): Document title
- H2 (`##`): Major sections
- H3 (`###`): Subsections
- H4 (`####`): Details

### Code Examples
- Use triple backticks with language identifier
- Keep examples focused and runnable

### External Links
- Use relative paths for internal docs
- Use absolute URLs for external references

### Status Indicators
- ✅ Complete/Working
- 🚧 In Progress
- ❌ Known Issue

## Maintenance

### Regular Updates
- Update when features change
- Update when APIs change
- Update when fixing bugs that affect usage

### Deprecation
- Mark deprecated features with notice
- Provide migration path
- Keep old docs for reference

### Archiving
- Move completed development phases to archive
- Keep implementation notes for reference
- Maintain development history

## Related Files

### Source Code Documentation
- Module-level documentation in source files
- Function documentation with examples
- Architecture comments in key files

### Configuration Files
- `.claude/CLAUDE.md` - Project guidelines
- `Cargo.toml` - Dependency and feature documentation
- `examples/` - Working code examples

### External Resources
- GitHub issues and discussions
- Commit history for changes
- PR descriptions for features

---

**Last Updated**: 2026-01-19
**Status**: Complete
