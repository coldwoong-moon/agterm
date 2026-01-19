# Terminal Command History Navigation Test Plan

## Test Environment
- Terminal: AgTerm (Floem UI)
- Shell: bash/zsh/fish
- OS: macOS/Linux

## Manual Test Cases

### Test 1: Arrow Up/Down History Navigation

**Steps**:
1. Launch AgTerm
2. Type and execute the following commands:
   ```bash
   echo "command 1"
   echo "command 2"
   echo "command 3"
   ```
3. Press `↑` (Up Arrow) once
4. Expected: Shows "echo "command 3""
5. Press `↑` again
6. Expected: Shows "echo "command 2""
7. Press `↑` again
8. Expected: Shows "echo "command 1""
9. Press `↓` (Down Arrow) once
10. Expected: Shows "echo "command 2""
11. Press `↓` again
12. Expected: Shows "echo "command 3""
13. Press `↓` again
14. Expected: Empty command line (current input)

**Status**: ✅ EXPECTED TO WORK (Standard ANSI escape sequences)

---

### Test 2: Ctrl+R Reverse History Search

**Steps**:
1. Execute several commands:
   ```bash
   echo "hello world"
   ls -la
   git status
   cargo build
   echo "goodbye world"
   ```
2. Press `Ctrl+R`
3. Expected: Shell shows reverse search prompt (e.g., `(reverse-i-search):`)
4. Type "echo"
5. Expected: Shows most recent "echo" command ("echo "goodbye world"")
6. Press `Ctrl+R` again
7. Expected: Shows previous "echo" command ("echo "hello world"")
8. Press `Enter`
9. Expected: Executes the selected command

**Status**: ✅ EXPECTED TO WORK (Ctrl+R sends byte 0x12)

---

### Test 3: Ctrl+S Forward History Search

**Steps**:
1. After using Ctrl+R to search backwards
2. Press `Ctrl+S`
3. Expected: Search forward through history
4. Note: May require `stty -ixon` in shell config

**Status**: ✅ EXPECTED TO WORK (Ctrl+S sends byte 0x13)

---

### Test 4: Home/End Navigation

**Steps**:
1. Type a long command: `echo "this is a very long command line"`
2. Press `Home`
3. Expected: Cursor moves to beginning
4. Press `End`
5. Expected: Cursor moves to end
6. Press `Ctrl+A`
7. Expected: Cursor moves to beginning (same as Home)
8. Press `Ctrl+E`
9. Expected: Cursor moves to end (same as End)

**Status**: ✅ EXPECTED TO WORK

---

### Test 5: Left/Right Arrow Navigation

**Steps**:
1. Type: `echo hello`
2. Press `←` (Left Arrow) 5 times
3. Expected: Cursor moves left, now before "hello"
4. Type "world "
5. Expected: Command now reads "echo world hello"
6. Press `→` (Right Arrow) 5 times
7. Expected: Cursor moves right to end

**Status**: ✅ EXPECTED TO WORK

---

### Test 6: Delete/Backspace

**Steps**:
1. Type: `echo test`
2. Press `Backspace`
3. Expected: Deletes "t", shows "echo tes"
4. Press `←` twice
5. Press `Delete`
6. Expected: Deletes "e", shows "echo ts"

**Status**: ✅ EXPECTED TO WORK

---

### Test 7: Tab Completion

**Steps**:
1. Type: `ec`
2. Press `Tab`
3. Expected: Completes to "echo" (if available)
4. Type: `ca`
5. Press `Tab` twice
6. Expected: Shows all commands starting with "ca"

**Status**: ✅ EXPECTED TO WORK (Tab sends \t)

---

### Test 8: Interrupt (Ctrl+C)

**Steps**:
1. Type: `echo hello`
2. Press `Ctrl+C`
3. Expected: Command is cancelled, shows new prompt
4. Start a long-running process: `sleep 100`
5. Press `Ctrl+C`
6. Expected: Process is interrupted

**Status**: ✅ EXPECTED TO WORK (Ctrl+C sends byte 0x03)

---

### Test 9: EOF (Ctrl+D)

**Steps**:
1. Press `Ctrl+D` on empty line
2. Expected: Exits shell (or shows logout message)
3. Type: `cat`
4. Press `Enter`
5. Type some text
6. Press `Ctrl+D`
7. Expected: Sends EOF, cat prints the text

**Status**: ✅ EXPECTED TO WORK (Ctrl+D sends byte 0x04)

---

### Test 10: Escape Key

**Steps**:
1. Press `Ctrl+R` to start search
2. Press `Escape`
3. Expected: Exits search mode, returns to normal prompt

**Status**: ✅ EXPECTED TO WORK (Escape sends \x1b)

---

## Keyboard Mapping Verification

| Key Combination | Expected Byte Sequence | Shell Interpretation |
|----------------|------------------------|---------------------|
| `↑` | `\x1b[A` | Previous command |
| `↓` | `\x1b[B` | Next command |
| `→` | `\x1b[C` | Cursor right |
| `←` | `\x1b[D` | Cursor left |
| `Home` | `\x1b[H` | Cursor to start |
| `End` | `\x1b[F` | Cursor to end |
| `PageUp` | `\x1b[5~` | Scroll up |
| `PageDown` | `\x1b[6~` | Scroll down |
| `Delete` | `\x1b[3~` | Delete forward |
| `Backspace` | `\x7f` | Delete backward |
| `Enter` | `\r` | Execute command |
| `Tab` | `\t` | Completion |
| `Escape` | `\x1b` | Cancel/Escape |
| `Ctrl+A` | `0x01` | Cursor to start |
| `Ctrl+B` | `0x02` | Cursor left |
| `Ctrl+C` | `0x03` | Interrupt |
| `Ctrl+D` | `0x04` | EOF |
| `Ctrl+E` | `0x05` | Cursor to end |
| `Ctrl+F` | `0x06` | Cursor right |
| `Ctrl+K` | `0x0B` | Kill to end |
| `Ctrl+L` | `0x0C` | Clear screen |
| `Ctrl+R` | `0x12` | Reverse search |
| `Ctrl+S` | `0x13` | Forward search |
| `Ctrl+U` | `0x15` | Kill line |
| `Ctrl+W` | `0x17` | Kill word |

## Implementation Details

**File**: `src/floem_app/views/pane_view.rs`

**Key Handler Location**: Lines 154-216

**Key Mapping Logic**:
1. Named keys (arrows, function keys) → ANSI escape sequences
2. Control keys → Control bytes (char - 'a' + 1)
3. Regular characters → UTF-8 bytes

**PTY Communication**:
- Keyboard bytes → PTY Manager → Shell process
- Shell output → PTY Manager → Terminal screen
- Bidirectional, asynchronous

## Debugging Tips

If history navigation doesn't work:

1. **Check shell history is enabled**:
   ```bash
   # Bash
   echo $HISTSIZE  # Should be > 0

   # Zsh
   echo $SAVEHIST  # Should be > 0
   ```

2. **Enable terminal logging**:
   ```bash
   AGTERM_LOG=agterm::terminal::pty=trace cargo run
   ```
   Check logs for keyboard input bytes

3. **Test with a simple shell command**:
   ```bash
   # In AgTerm, type these raw bytes
   printf '\x1b[A'  # Should act like Up arrow
   printf '\x12'    # Should act like Ctrl+R
   ```

4. **Verify PTY is working**:
   ```bash
   # Check PTY sessions
   ps aux | grep -i pty
   ```

5. **Test in a standard terminal first**:
   - Verify the same commands work in Terminal.app or iTerm2
   - Ensures shell configuration is correct

## Success Criteria

✅ All 10 manual test cases pass
✅ Arrow keys navigate history correctly
✅ Ctrl+R opens reverse search
✅ Ctrl+C interrupts commands
✅ Tab completion works
✅ Line editing with cursor movement works
✅ No keyboard input is dropped or delayed

## Known Limitations

1. **No OSC 133 support**: Cannot jump between prompts
2. **No custom history UI**: Uses shell's built-in history
3. **No history sync**: Each tab has independent history
4. **No persistent history UI**: History stored by shell only

## Future Enhancements

1. **OSC 133 Prompt Marking**
   - Parse OSC 133 sequences
   - Store prompt positions
   - Add Alt+Up/Down to jump between prompts

2. **History Panel**
   - Side panel showing command history
   - Click to insert or execute
   - Search and filter

3. **Smart History**
   - Suggest commands based on context
   - Learn from frequently used commands
   - Cross-tab history suggestions

4. **History Export**
   - Export history to file
   - Import history from other terminals
   - Share history between sessions
