# AgTerm Keyboard Shortcuts Reference

## Command History Navigation

### Basic History

| Shortcut | Action | Description |
|----------|--------|-------------|
| `↑` Up Arrow | Previous command | Navigate backwards through command history |
| `↓` Down Arrow | Next command | Navigate forwards through command history |
| `Ctrl+P` | Previous command | Same as Up Arrow (Emacs binding) |
| `Ctrl+N` | Next command | Same as Down Arrow (Emacs binding) |

### History Search

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Ctrl+R` | Reverse search | Search backwards through history (type to filter) |
| `Ctrl+S` | Forward search | Search forwards through history (requires `stty -ixon`) |

**Reverse Search Usage**:
1. Press `Ctrl+R`
2. Type to search (e.g., "git")
3. Press `Ctrl+R` again to cycle through matches
4. Press `Enter` to execute, `Esc` to cancel

## Line Editing

### Cursor Movement

| Shortcut | Action | Description |
|----------|--------|-------------|
| `←` Left Arrow | Move left | Move cursor one character left |
| `→` Right Arrow | Move right | Move cursor one character right |
| `Ctrl+B` | Move backward | Same as Left Arrow |
| `Ctrl+F` | Move forward | Same as Right Arrow |
| `Home` / `Ctrl+A` | Beginning of line | Move cursor to start of line |
| `End` / `Ctrl+E` | End of line | Move cursor to end of line |
| `Alt+B` | Word backward | Move cursor one word left |
| `Alt+F` | Word forward | Move cursor one word right |

### Text Manipulation

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Backspace` | Delete backward | Delete character before cursor |
| `Delete` / `Ctrl+D` | Delete forward | Delete character at cursor |
| `Ctrl+K` | Kill to end | Delete from cursor to end of line |
| `Ctrl+U` | Kill line | Delete entire line before cursor |
| `Ctrl+W` | Kill word | Delete word before cursor |
| `Alt+D` | Kill word forward | Delete word after cursor |
| `Ctrl+Y` | Yank | Paste last killed text |

## Process Control

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Ctrl+C` | Interrupt | Send SIGINT (interrupt process) |
| `Ctrl+D` | EOF | Send EOF (exit shell if line empty) |
| `Ctrl+Z` | Suspend | Send SIGTSTP (suspend process to background) |
| `Ctrl+L` | Clear screen | Clear terminal screen |

## Tab Completion

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Tab` | Complete | Auto-complete command/file/path |
| `Tab Tab` | List completions | Show all possible completions |

## AgTerm-Specific Shortcuts

### Tab Management

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Cmd+T` (macOS) / `Ctrl+Shift+T` | New tab | Open new terminal tab |
| `Cmd+W` (macOS) / `Ctrl+Shift+W` | Close tab | Close current terminal tab |
| `Cmd+Tab` / `Ctrl+Tab` | Next tab | Switch to next tab |
| `Cmd+Shift+Tab` / `Ctrl+Shift+Tab` | Previous tab | Switch to previous tab |

### Pane Management

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Cmd+D` / `Ctrl+Shift+D` | Split vertical | Split current pane vertically |
| `Cmd+Shift+D` | Split horizontal | Split current pane horizontally |
| `Cmd+Shift+W` | Close pane | Close focused pane (if not last) |
| `Cmd+Tab` | Next pane | Focus next pane |
| `Cmd+Shift+Tab` | Previous pane | Focus previous pane |

### View Controls

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Cmd+Plus` / `Ctrl+Plus` | Increase font | Make text larger |
| `Cmd+Minus` / `Ctrl+Minus` | Decrease font | Make text smaller |
| `Cmd+0` / `Ctrl+0` | Reset font | Reset font size to default |
| `Cmd+Shift+T` / `Ctrl+Shift+T` | Toggle theme | Switch between Dark/Light theme |
| `PageUp` | Scroll up | Scroll terminal up |
| `PageDown` | Scroll down | Scroll terminal down |

### Clipboard

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Cmd+V` / `Ctrl+Shift+V` | Paste | Paste from clipboard |
| `Cmd+C` / `Ctrl+Shift+C` | Copy | Copy selected text (when text selected) |

## Shell-Specific Features

### Bash

```bash
# Enable in ~/.bashrc
export HISTSIZE=10000
export HISTFILESIZE=20000

# Enable Ctrl+S forward search
stty -ixon

# Enable incremental history search
bind '"\e[A": history-search-backward'
bind '"\e[B": history-search-forward'
```

### Zsh

```bash
# Enable in ~/.zshrc
HISTSIZE=10000
SAVEHIST=10000
HISTFILE=~/.zsh_history

# Enable better history search
bindkey '^R' history-incremental-search-backward
bindkey '^S' history-incremental-search-forward
```

### Fish

```fish
# Fish has excellent history search by default
# Start typing, then press Up arrow to search
# Ctrl+R opens a visual history picker
```

## Tips & Tricks

### 1. Fuzzy History Search (Fish)
In Fish shell, start typing part of a command and press Up arrow - it will search history for commands starting with what you typed.

### 2. Better Reverse Search
Instead of cycling with Ctrl+R, type more characters to narrow down the search.

### 3. Command Substitution
Use `!!` to repeat last command:
```bash
sudo !!  # Run last command with sudo
```

### 4. History Expansion
```bash
!git      # Run most recent command starting with "git"
!?status  # Run most recent command containing "status"
!$        # Use last argument of previous command
```

### 5. Quick Command Fix
```bash
^old^new  # Replace "old" with "new" in previous command
```

## Troubleshooting

### History Not Working?

1. **Check history size**:
   ```bash
   echo $HISTSIZE  # Should be > 0
   ```

2. **Check history file**:
   ```bash
   ls -la ~/.bash_history  # Bash
   ls -la ~/.zsh_history   # Zsh
   ls -la ~/.local/share/fish/fish_history  # Fish
   ```

3. **Enable history** (Bash):
   ```bash
   # Add to ~/.bashrc
   export HISTSIZE=10000
   export HISTFILESIZE=20000
   shopt -s histappend
   ```

### Ctrl+S Not Working?

Enable XON/XOFF flow control:
```bash
# Add to ~/.bashrc or ~/.zshrc
stty -ixon
```

### Arrow Keys Showing `^[[A`?

This means the terminal is not in application mode. AgTerm sends correct escape sequences, but your shell may not be configured correctly. Try:
```bash
# Reset terminal
reset
# Or
tput reset
```

## Advanced Features (Coming Soon)

### OSC 133 Prompt Marking
- Jump between command prompts with Alt+Up/Down
- Select command output
- Rerun previous commands

### History Panel
- Visual history browser
- Search and filter
- Click to insert or execute

### Smart Suggestions
- Context-aware completions
- Frequently used commands
- Command history analysis

## Quick Reference Card

```
HISTORY:        ↑↓ navigate    Ctrl+R search    Ctrl+S forward
CURSOR:         ←→ move        Ctrl+A start     Ctrl+E end
DELETE:         Bksp backward  Del forward      Ctrl+K kill
PROCESS:        Ctrl+C kill    Ctrl+D exit      Ctrl+Z suspend
COMPLETE:       Tab complete   Tab Tab list
SCREEN:         Ctrl+L clear   PgUp/PgDn scroll
TABS:           Cmd+T new      Cmd+W close      Cmd+Tab switch
PANES:          Cmd+D split    Cmd+Shift+W close
VIEW:           Cmd++ bigger   Cmd+- smaller    Cmd+0 reset
```

## See Also

- [Terminal History Navigation Analysis](../TERMINAL_HISTORY_ANALYSIS.md) - Technical details
- [Implementation Report](../HISTORY_NAVIGATION_REPORT.md) - Test results
- [Readline Documentation](https://tiswww.case.edu/php/chet/readline/readline.html)
- [Bash Reference Manual](https://www.gnu.org/software/bash/manual/bash.html)
