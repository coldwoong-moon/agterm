# IME Input Testing Guide for AgTerm Floem

## Overview

AgTerm Floem now supports Input Method Editor (IME) for composing characters in Korean, Japanese, Chinese, and other languages that require character composition.

## Implementation Details

### IME Event Handlers

The implementation includes four IME event handlers in `src/floem_app/views/pane_view.rs`:

1. **ImeEnabled** - Triggered when IME is activated
   - Clears the composing text buffer
   - Logs activation for debugging

2. **ImePreedit** - Triggered during character composition
   - Receives the current composition string (e.g., "ㅎ" → "하" → "한")
   - Updates the IME overlay to show the composing text
   - Cursor position information is available but not currently displayed

3. **ImeCommit** - Triggered when composition is complete
   - Receives the final composed text (e.g., "한글")
   - Sends the text to the PTY session
   - Clears the IME overlay

4. **ImeDisabled** - Triggered when IME is deactivated
   - Clears the composing text buffer

### Visual Feedback

- **IME Overlay**: A small popup appears at the bottom-left of the terminal showing "IME: {composing_text}"
- **Styling**: Blue border with semi-transparent background
- **Auto-hide**: Disappears when no text is being composed

## Testing Korean Input

### Test 1: Basic Korean Composition

1. Launch agterm-floem:
   ```bash
   cargo run --bin agterm-floem --features floem-gui
   ```

2. Switch to Korean input method (macOS: Cmd+Space → Korean)

3. Type the following and verify each stage:
   - Type "ㅎ" (h key) → Should see "IME: ㅎ" in overlay
   - Type "ㅏ" (a key) → Should see "IME: 하" in overlay
   - Type "ㄴ" (s key) → Should see "IME: 한" in overlay
   - Type "ㄱ" (r key) → Should see "IME: 한ㄱ" in overlay
   - Type "ㅡ" (m key) → Should see "IME: 한그" in overlay
   - Type "ㄹ" (f key) → Should see "IME: 한글" in overlay
   - Press Space or Enter → "한글" is sent to terminal, overlay disappears

### Test 2: Multiple Words

1. Type "안녕하세요" (Hello in Korean)
   - Each character should compose properly
   - Space between words should commit the previous character

2. Verify the text appears in the terminal after composition

### Test 3: Mixed Input

1. Type some English text: "Hello"
2. Switch to Korean and type: "한글"
3. Switch back to English and type: "test"
4. Verify all text appears correctly in the terminal

## Testing Japanese Input

### Test 1: Hiragana Composition

1. Switch to Japanese input method
2. Type "konnichiha" (こんにちは - Hello)
3. Verify the IME overlay shows the composition
4. Press Enter to commit
5. Verify the text appears in terminal

### Test 2: Kanji Conversion

1. Type "nihon" (にほん)
2. Press Space to see kanji candidates
3. Select "日本" (Japan)
4. Verify the conversion works and text is sent to terminal

## Testing Chinese Input

### Test 1: Pinyin Input (Simplified Chinese)

1. Switch to Chinese (Simplified) input method
2. Type "nihao" (你好 - Hello)
3. Select the correct characters from candidates
4. Verify text appears in terminal

### Test 2: Multiple Characters

1. Type "zhongwen" (中文 - Chinese)
2. Verify composition and candidate selection
3. Confirm text in terminal

## Debugging

### Enable Debug Logging

Run with debug logging to see IME events:

```bash
RUST_LOG=agterm=debug,agterm_floem=debug cargo run --bin agterm-floem --features floem-gui
```

### Expected Log Output

```
[DEBUG] IME enabled
[DEBUG] IME preedit: text='ㅎ', cursor=Some((1, 1))
[DEBUG] IME preedit: text='하', cursor=Some((1, 1))
[DEBUG] IME preedit: text='한', cursor=Some((1, 1))
[INFO ] IME commit: '한'
[DEBUG] Sent 3 bytes (IME) to PTY: "한"
```

## Known Limitations

1. **Cursor Position**: The IME cursor position is logged but not visually displayed in the overlay
2. **Overlay Position**: Currently fixed at bottom-left; could be enhanced to follow cursor
3. **Candidate Window**: OS-level candidate selection window (not controlled by AgTerm)
4. **RTL Languages**: Right-to-left languages not specifically tested

## Architecture

### File Structure

- `src/floem_app/mod.rs` - Enables IME with `set_ime_allowed(true)`
- `src/floem_app/views/pane_view.rs` - IME event handlers and overlay rendering
- `src/floem_app/views/terminal.rs` - Terminal state with `ime_composing` signal

### State Management

- `TerminalState.ime_composing: RwSignal<String>` - Reactive signal for composing text
- Updates trigger automatic UI refresh via Floem's reactive system

## Troubleshooting

### IME Not Working

1. **Check if IME is enabled**: Look for "IME input enabled" in logs
2. **Verify input method**: Ensure OS input method is active
3. **Check focus**: Terminal pane must be focused (blue border)
4. **Check logs**: Look for IME event messages

### Text Not Appearing

1. **Check PTY session**: Verify PTY is active (shown in pane header)
2. **Check write errors**: Look for "Failed to write IME commit to PTY" in logs
3. **Test with ASCII**: Verify basic keyboard input works first

### Overlay Not Showing

1. **Check reactivity**: Verify `ime_composing` signal is updating
2. **Check styling**: Ensure overlay is not hidden by other elements
3. **Check theme**: Try different themes to rule out color issues

## Future Enhancements

1. **Dynamic overlay position** - Follow terminal cursor
2. **Show cursor position** - Display IME cursor within composed text
3. **Candidate window** - Custom candidate selection UI
4. **Composition preview** - Show composition in-place at cursor
5. **Per-language settings** - Different behaviors for different input methods

## Success Criteria

- [ ] Korean composition works (ㅎ → 하 → 한 → 한글)
- [ ] Japanese hiragana input works
- [ ] Japanese kanji conversion works
- [ ] Chinese pinyin input works
- [ ] IME overlay displays composing text
- [ ] Committed text appears in terminal
- [ ] Mixed language input works
- [ ] No crashes or hangs during composition
- [ ] PTY receives correct UTF-8 bytes

## References

- [Floem IME Documentation](https://github.com/lapce/floem)
- [winit IME Support](https://docs.rs/winit/latest/winit/event/enum.Ime.html)
- [Unicode Input Method Editor](https://en.wikipedia.org/wiki/Input_method)
