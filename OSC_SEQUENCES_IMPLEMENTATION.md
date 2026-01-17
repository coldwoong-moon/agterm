# OSC Sequence Implementation

## Overview
Added comprehensive support for OSC (Operating System Command) sequences in the terminal screen buffer (`src/terminal/screen.rs`).

## Implemented OSC Sequences

### 1. OSC 0 - Set Icon Name and Window Title
```
ESC ] 0 ; title ST
```
Sets both the window title and icon name to the same value.

### 2. OSC 1 - Set Icon Name
```
ESC ] 1 ; name ST
```
Sets only the icon name.

### 3. OSC 2 - Set Window Title
```
ESC ] 2 ; title ST
```
Sets only the window title.

### 4. OSC 7 - Set Current Working Directory
```
ESC ] 7 ; file://hostname/path ST
```
Sets the current working directory from the shell. Supports various file URI formats:
- `file:///absolute/path` - Absolute path
- `file://hostname/path` - Path with hostname
- `file:/path` - Path with single slash

### 5. OSC 52 - Clipboard Operations
```
ESC ] 52 ; c ; base64-data ST
```
Handles clipboard operations. The base64-encoded data is stored for external handling.
- Selection parameter (typically 'c' for clipboard, 'p' for primary)
- Base64-encoded clipboard content

## Implementation Details

### Added Fields to `TerminalScreen`
```rust
/// Window title (OSC 0 or OSC 2)
window_title: Option<String>,

/// Icon name (OSC 1)
icon_name: Option<String>,

/// Current working directory from shell (OSC 7)
cwd_from_shell: Option<String>,

/// Clipboard request data (OSC 52)
clipboard_request: Option<String>,
```

### Public API Methods

#### Getters
```rust
pub fn window_title(&self) -> Option<&str>
pub fn icon_name(&self) -> Option<&str>
pub fn cwd_from_shell(&self) -> Option<&str>
pub fn clipboard_request(&self) -> Option<&str>
```

#### Utility Methods
```rust
pub fn clear_clipboard_request(&mut self)
fn parse_file_uri(&self, uri: &str) -> Option<String>
```

### OSC Dispatcher Implementation
The `osc_dispatch` function in the `Perform` trait implementation handles all OSC sequences:

```rust
fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
    // Parses command number and dispatches to appropriate handler
    // Supports both ST (String Terminator) and BEL (Bell) terminators
}
```

## Testing

Created comprehensive integration tests in `tests/osc_sequences_test.rs`:

### Test Coverage
1. `test_osc_window_title` - Tests OSC 2 sequence
2. `test_osc_icon_name` - Tests OSC 1 sequence
3. `test_osc_both_title_and_icon` - Tests OSC 0 sequence
4. `test_osc_cwd` - Tests OSC 7 with file URI
5. `test_osc_clipboard_request` - Tests OSC 52 clipboard operations
6. `test_osc_bell_terminated` - Tests BEL terminator
7. `test_osc_file_uri_parsing` - Tests various file URI formats

All tests pass successfully:
```
running 7 tests
test test_osc_bell_terminated ... ok
test test_osc_clipboard_request ... ok
test test_osc_both_title_and_icon ... ok
test test_osc_cwd ... ok
test test_osc_file_uri_parsing ... ok
test test_osc_icon_name ... ok
test test_osc_window_title ... ok

test result: ok. 7 passed
```

## File Changes

### Modified Files
1. `/Users/yunwoopc/SIDE-PROJECT/agterm/src/terminal/screen.rs`
   - Added OSC-related fields to `TerminalScreen` struct
   - Implemented `osc_dispatch` function
   - Added public getter methods
   - Added `parse_file_uri` helper function

2. `/Users/yunwoopc/SIDE-PROJECT/agterm/Cargo.toml`
   - Added library target for testing

### New Files
1. `/Users/yunwoopc/SIDE-PROJECT/agterm/src/lib.rs`
   - Library entry point exposing terminal module

2. `/Users/yunwoopc/SIDE-PROJECT/agterm/tests/osc_sequences_test.rs`
   - Comprehensive integration tests for OSC sequences

## Usage Example

```rust
use agterm::terminal::screen::TerminalScreen;

let mut screen = TerminalScreen::new(80, 24);

// Process OSC sequence to set window title
screen.process(b"\x1b]2;My Terminal Window\x1b\\");

// Read the window title
if let Some(title) = screen.window_title() {
    println!("Window title: {}", title);
}

// Process OSC sequence to set CWD
screen.process(b"\x1b]7;file:///home/user/project\x1b\\");

// Read the CWD
if let Some(cwd) = screen.cwd_from_shell() {
    println!("Current directory: {}", cwd);
}
```

## Notes

- Both ST (String Terminator: `ESC \`) and BEL (Bell: `\x07`) terminators are supported
- The clipboard data is stored as base64 and must be decoded by the caller
- File URI parsing handles multiple formats for maximum compatibility
- All OSC fields are optional and return `None` if not set
