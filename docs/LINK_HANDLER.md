# Link Handler Module

The `link_handler` module provides comprehensive link detection and handling capabilities for terminal output. It supports multiple link types including URLs, file paths, email addresses, IP addresses, and custom regex patterns.

## Features

- **Multiple Link Types**: Detect URLs, file paths, emails, IP addresses, and custom patterns
- **Flexible Detection**: Configurable link patterns with regex support
- **Action Handlers**: Define custom actions for different link types
- **Position-based Lookup**: Find links at specific text positions
- **Non-overlapping Results**: Automatically resolves overlapping matches
- **Extensible**: Add custom link patterns and handlers

## Link Types

### Supported Link Types

1. **URL** - Web URLs (http, https, ftp, file)
   - Examples: `https://example.com`, `http://localhost:8080`, `file:///path/to/file`

2. **FilePath** - File system paths
   - Absolute: `/usr/local/bin/app.sh`
   - Home-relative: `~/config.toml`
   - Relative: `./src/main.rs`, `../README.md`

3. **Email** - Email addresses
   - Examples: `user@example.com`, `support+tag@example.com`

4. **IpAddress** - IP addresses with optional ports
   - IPv4: `192.168.1.1`, `192.168.1.1:8080`
   - IPv6: `[::1]:8080`, `[2001:db8::1]:443`

5. **Custom** - User-defined regex patterns
   - Examples: GitHub issues (`#123`), Jira tickets (`PROJ-456`)

## Basic Usage

### Simple Link Detection

```rust
use agterm::link_handler::{LinkDetector, LinkType};

let detector = LinkDetector::new();
let text = "Visit https://example.com or email support@example.com";
let links = detector.detect_links(text);

for link in links {
    println!("Found {} at position {}: {}",
             link.link_type.name(), link.start, link.text);
}
```

### Find Link at Position

```rust
let text = "Check /var/log/app.log for errors";
if let Some(link) = detector.find_link_at(text, 10) {
    println!("Link: {} ({})", link.text, link.link_type.name());
}
```

## Advanced Usage

### Custom Link Patterns

```rust
use agterm::link_handler::LinkDetector;

let mut detector = LinkDetector::new();

// Add pattern for GitHub issue references
detector.add_custom_pattern(
    "GitHub Issue".to_string(),
    r"#\d+"
).unwrap();

// Add pattern for Jira tickets
detector.add_custom_pattern(
    "Jira Ticket".to_string(),
    r"[A-Z]+-\d+"
).unwrap();

let text = "Fixed in #123 and PROJ-456";
let links = detector.detect_links(text);
```

### Selective Link Detection

```rust
use agterm::link_handler::{LinkDetector, LinkType};

// Only detect URLs and emails
let detector = LinkDetector::with_types(&[
    LinkType::Url,
    LinkType::Email
]);

let text = "Email info@example.com or visit https://example.com";
let links = detector.detect_links(text);
```

### Link Handler with Actions

```rust
use agterm::link_handler::{LinkHandler, LinkAction, LinkType};

let mut handler = LinkHandler::new();

// Set action for file paths to open in VS Code
handler.set_default_action(
    LinkType::FilePath,
    LinkAction::Command("code {}".to_string())
);

// Set email links to copy to clipboard
handler.set_default_action(
    LinkType::Email,
    LinkAction::CopyToClipboard
);

// Handle a link
let text = "Error in /src/main.rs";
if let Some(link) = handler.detector().find_link_at(text, 12) {
    match handler.handle_link(&link) {
        Ok(()) => println!("Link handled successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Custom Action Callbacks

```rust
use std::sync::Arc;
use agterm::link_handler::{LinkHandler, LinkAction, LinkType};

let mut handler = LinkHandler::new();

// Define custom action
let custom_action = LinkAction::Custom(Arc::new(|link| {
    println!("Custom action for: {}", link.text);
    // Perform custom logic here
    Ok(())
}));

handler.set_default_action(LinkType::Custom("Issue".to_string()), custom_action);
```

## Link Actions

### Built-in Actions

1. **OpenDefault** - Opens link in system's default application
   - URLs: Opens in default browser
   - File paths: Opens in default file manager/editor
   - Emails: Opens in default email client (with `mailto:` prefix)
   - IP addresses: Opens as HTTP URL if port is specified

2. **CopyToClipboard** - Copies link text to clipboard
   - Uses the `arboard` crate for cross-platform clipboard access

3. **Command** - Executes a shell command with the link
   - Use `{}` as placeholder for the link text
   - Example: `"code {}"` opens file in VS Code

4. **Custom** - User-defined callback function
   - Receives the `Link` object
   - Returns `Result<(), String>` for error handling

## API Reference

### Link

```rust
pub struct Link {
    pub link_type: LinkType,
    pub text: String,
    pub start: usize,
    pub end: usize,
}
```

Methods:
- `new(link_type, text, start, end)` - Create a new link
- `len()` - Get the length of the link text
- `is_empty()` - Check if the link is empty
- `contains_position(pos)` - Check if position is within link range

### LinkType

```rust
pub enum LinkType {
    Url,
    FilePath,
    Email,
    IpAddress,
    Custom(String),
}
```

Methods:
- `name()` - Get human-readable name for the link type

### LinkDetector

```rust
pub struct LinkDetector { ... }
```

Methods:
- `new()` - Create detector with all default patterns
- `with_types(&[LinkType])` - Create detector with specific types only
- `add_custom_pattern(name, pattern)` - Add a custom regex pattern
- `detect_links(text)` - Find all links in text
- `find_link_at(text, pos)` - Find link at specific position

### LinkHandler

```rust
pub struct LinkHandler { ... }
```

Methods:
- `new()` - Create handler with default actions
- `with_detector(detector)` - Create handler with custom detector
- `set_default_action(link_type, action)` - Set action for link type
- `detector()` - Get reference to detector
- `detector_mut()` - Get mutable reference to detector
- `handle_link(link)` - Handle link with default action
- `handle_link_with_action(link, action)` - Handle link with specific action

## Regular Expression Patterns

### URL Pattern
```regex
(?i)\b(?:https?|ftp|file)://[^\s<>"'\]\)]+
```
Matches: `http://`, `https://`, `ftp://`, `file://` URLs

### Email Pattern
```regex
\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b
```
Matches: Standard email addresses with local and domain parts

### IP Address Pattern
```regex
(?:\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(?::\d{1,5})?\b|\[(?:[0-9a-fA-F]{0,4}:){2,7}[0-9a-fA-F]{0,4}\](?::\d{1,5})?)
```
Matches: IPv4 and IPv6 addresses with optional ports

### File Path Pattern
```regex
(?:^|[\s:])(/[^\s:;,\]\)]+|~[^\s:;,\]\)]+|\.{1,2}/[^\s:;,\]\)]+)
```
Matches: Absolute, home-relative, and relative paths

## Examples

### Terminal Output Parsing

```rust
// Parse terminal output for clickable links
let output = r#"
Error occurred in /usr/local/bin/app.sh at line 42
Visit https://docs.example.com for help
Contact support@example.com for assistance
Server running at 192.168.1.100:8080
"#;

let detector = LinkDetector::new();
for line in output.lines() {
    let links = detector.detect_links(line);
    for link in links {
        println!("Found {}: {}", link.link_type.name(), link.text);
    }
}
```

### Interactive Terminal

```rust
// Handle click at specific position
fn handle_mouse_click(line: &str, column: usize, handler: &LinkHandler) {
    if let Some(link) = handler.detector().find_link_at(line, column) {
        match handler.handle_link(&link) {
            Ok(()) => println!("Opened: {}", link.text),
            Err(e) => eprintln!("Failed to open link: {}", e),
        }
    }
}
```

### Custom Link Highlighting

```rust
// Highlight all links in terminal output
let text = "Visit https://example.com or check /var/log/app.log";
let links = detector.detect_links(text);

for link in links {
    println!("Highlight {} from {} to {}",
             link.link_type.name(), link.start, link.end);
}
```

## Integration with Terminal

The link handler integrates with the existing terminal URL handling:

1. **OSC 8 Hyperlinks** (`terminal::hyperlink`) - Explicit hyperlinks in terminal output
2. **Automatic Detection** (`terminal::url`) - Detect URLs and file paths in plain text
3. **Link Handler** (`link_handler`) - Extended detection with actions and custom patterns

### Recommended Usage

Use all three systems together:
- OSC 8 for application-defined hyperlinks
- URL detector for basic URL/path detection
- Link handler for advanced detection and custom actions

## Testing

The module includes comprehensive tests covering:
- All link types (URL, email, IP, file path)
- Custom patterns
- Position-based lookups
- Overlapping link resolution
- Edge cases (line boundaries, special characters, etc.)

Run tests:
```bash
cargo test --lib link_handler
```

## Performance Considerations

- Regex compilation is done once using `Lazy` static initialization
- Links are detected on-demand, not cached
- Overlapping links are resolved efficiently with O(n log n) complexity
- Custom patterns add minimal overhead

## Future Enhancements

Potential improvements:
- Link caching for frequently accessed lines
- Async link validation
- Link preview/tooltip support
- Configurable link styling/colors
- Integration with terminal theme system
