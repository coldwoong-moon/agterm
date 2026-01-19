# Clipboard Usage Guide

## Paste Functionality (Cmd+V)

### How to Use

1. **Copy text to clipboard**
   - Use any application to copy text (e.g., a text editor, web browser)
   - Or use the terminal: `echo "hello world" | pbcopy` (macOS)

2. **Open AgTerm with Floem GUI**
   ```bash
   cargo run --features floem-gui --bin agterm-floem
   ```

3. **Paste into terminal**
   - Press `Cmd+V` (macOS) or `Super+V` (Linux/Windows)
   - The clipboard content will be sent to the active terminal

### Examples

#### Example 1: Paste a command
```bash
# Copy this command in your text editor:
echo "Hello from clipboard!"

# Press Cmd+V in AgTerm
# The command will appear and can be executed with Enter
```

#### Example 2: Paste multi-line content
```bash
# Copy this multi-line script:
for i in {1..5}; do
  echo "Line $i"
done

# Press Cmd+V in AgTerm
# All lines will be pasted
```

#### Example 3: Paste code with proper indentation
```python
# Copy this Python code:
def hello_world():
    print("Hello, World!")
    return True

# Press Cmd+V in AgTerm
# The code maintains its indentation
```

#### Example 4: Paste Korean/CJK text
```bash
# Copy this Korean text:
안녕하세요, 세계!

# Press Cmd+V in AgTerm
# UTF-8 characters are properly handled
```

### Features

- **Multi-byte character support**: Handles UTF-8 text including Korean, Chinese, Japanese
- **Multi-line paste**: Pastes entire clipboard content, preserving line breaks
- **Pane-aware**: Pastes to the currently focused pane in split-pane layouts
- **Error handling**: Gracefully handles empty clipboard or clipboard access errors

### Debugging

If paste doesn't work, check the logs:

```bash
# Enable debug logging
RUST_LOG=agterm=debug cargo run --features floem-gui --bin agterm-floem

# Look for these log messages:
# - "Paste from clipboard (Cmd+V)" - Keyboard shortcut detected
# - "Pasting N bytes from clipboard" - Clipboard read successfully
# - "Successfully pasted N bytes to PTY" - Data sent to terminal
```

### Troubleshooting

| Issue | Solution |
|-------|----------|
| Nothing happens when pressing Cmd+V | Check if clipboard has content |
| Paste doesn't appear in terminal | Check logs for PTY write errors |
| Special characters garbled | Ensure terminal locale is UTF-8 |
| Paste appears in wrong pane | Click on desired pane first to focus it |

### Keyboard Shortcuts Reference

| Shortcut | Action | Status |
|----------|--------|--------|
| `Cmd+V` | Paste from clipboard | ✅ Implemented |
| `Cmd+C` | Copy selected text | 🚧 Not yet implemented |
| `Cmd+Shift+C` | Force copy (override interrupt) | 🚧 Not yet implemented |

### Platform Notes

- **macOS**: Uses `Cmd` key (⌘)
- **Linux**: Uses `Super` key (Windows key)
- **Windows**: Uses `Super` key (Windows key)

The implementation uses the `meta` modifier in Floem, which maps to the platform's primary modifier key.

### Related Features

- **OSC 52 Clipboard**: Terminal applications can read/write clipboard using escape sequences
- **Paste Bracketing**: Future enhancement to wrap pasted text with special markers
- **Selection Copy**: Future enhancement to copy selected terminal text

### Implementation Details

For developers interested in the implementation:
- Located in: `src/floem_app/mod.rs`
- Function: `handle_paste()`
- Uses: `arboard` crate for clipboard access
- Sends data via: `PtyManager::write()`
