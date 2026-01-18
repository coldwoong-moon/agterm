# OSC 8 Hyperlink Protocol Implementation

## Summary

Successfully implemented OSC 8 hyperlink protocol support for AgTerm terminal emulator.

## Files Modified/Created

### 1. **src/terminal/hyperlink.rs** (NEW)
- Created new hyperlink module with OSC 8 parser
- `Hyperlink` struct with URL and optional ID
- `parse_osc8()` method to parse OSC 8 sequences
- Parse ID parameters for link grouping
- 7 comprehensive tests covering various scenarios

### 2. **src/terminal/mod.rs**
- Added `pub mod hyperlink;` to expose the new module

### 3. **src/terminal/screen.rs**
- Imported `Hyperlink` struct
- Added `current_hyperlink: Option<Arc<String>>` field to `TerminalScreen`
- Implemented OSC 8 handling in `osc_dispatch()` method (line 1643-1662)
  - Parses OSC 8 sequences with format: `\x1b]8;params;url\x07`
  - Sets active hyperlink state
  - Uses string interning for memory efficiency
  - Supports hyperlink termination with empty URL
- Updated `print_char` logic to apply `current_hyperlink` to cells (3 locations)
  - Regular characters
  - Wide characters (CJK, emoji)
  - Placeholder cells

### 4. **src/config/mod.rs**
- Added placeholder `ClipboardConfig` and `SearchConfig` structs
- Updated `AppConfig` struct to include new config fields
- Updated `Default` implementation and `merge()` method

### 5. **examples/test_hyperlink.sh** (NEW)
- Test script demonstrating OSC 8 functionality
- Examples of:
  - Simple hyperlinks
  - Hyperlinks with IDs
  - File URLs
  - Multiple links on one line
  - Links with query parameters

## OSC 8 Protocol Format

```
\x1b]8;[params];[url]\x07   or   \x1b]8;[params];[url]\x1b\\
```

- **params**: Optional parameters (e.g., `id=value` for grouping)
- **url**: Target URL (can be http://, https://, file://, etc.)
- Empty URL terminates active hyperlink

## Examples

### Simple Hyperlink
```bash
printf "\e]8;;https://github.com\e\\GitHub\e]8;;\e\\\n"
```

### Hyperlink with ID
```bash
printf "\e]8;id=link1;https://example.com\e\\Example Site\e]8;;\e\\\n"
```

### File URL
```bash
printf "\e]8;;file:///etc/hosts\e\\/etc/hosts\e]8;;\e\\\n"
```

## Test Results

All 7 hyperlink tests pass:
- ✓ test_parse_simple_url
- ✓ test_parse_url_with_id
- ✓ test_parse_url_with_multiple_params
- ✓ test_parse_empty_url_terminates
- ✓ test_parse_invalid_format
- ✓ test_parse_url_with_special_chars
- ✓ test_parse_file_url

## Technical Details

### Memory Optimization
- Uses `Arc<String>` for hyperlink URLs
- String interning via `StringInterner` for efficient memory usage
- Same URL shared across multiple cells without duplication

### Architecture
- OSC sequences parsed by VTE parser
- `osc_dispatch()` routes OSC 8 commands
- Current hyperlink state maintained in `TerminalScreen`
- Applied to cells during character rendering

### Rendering Support
- Cells with hyperlinks already support:
  - Cyan color (line 638-639 in terminal_canvas.rs)
  - Underline rendering (line 660 in terminal_canvas.rs)
  - Click detection (line 791-794 in terminal_canvas.rs)
  - URL opening with `open` crate

## Next Steps (for UI implementation)

1. **Mouse hover**: Show URL in status bar or tooltip
2. **Ctrl+Click**: Open URL in default browser (already implemented)
3. **Visual feedback**: Highlight on hover
4. **Keyboard navigation**: Tab through links

## Testing

Run the test script:
```bash
chmod +x examples/test_hyperlink.sh
./examples/test_hyperlink.sh
```

Run unit tests:
```bash
cargo test hyperlink
```

Build project:
```bash
cargo build
```

## References

- [OSC 8 Hyperlink Specification](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda)
- [VTE OSC Sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)
