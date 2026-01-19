# Terminal Title - Quick Reference

## Overview

AgTerm supports dynamic tab titles that automatically update based on OSC (Operating System Command) sequences sent by applications and the shell.

## OSC Sequences

### Format

OSC sequences follow this format:

```
ESC ] <command> ; <data> BEL
```

Where:
- `ESC` = `\033` or `\x1b`
- `]` = Literal right bracket
- `<command>` = Numeric command (0, 1, 2, 7)
- `;` = Separator
- `<data>` = Title text or path
- `BEL` = `\007` or `\x07` (Bell character)

### Supported Commands

| Command | Description | Example |
|---------|-------------|---------|
| OSC 0 | Set both icon and window title | `\033]0;My Title\007` |
| OSC 1 | Set icon title only | `\033]1;Icon\007` |
| OSC 2 | Set window title only | `\033]2;Window\007` |
| OSC 7 | Set working directory | `\033]7;file:///path\007` |

## Usage Examples

### Bash/Shell

```bash
# Set title
echo -ne "\033]0;My Title\007"

# Set title to current directory
echo -ne "\033]0;${PWD}\007"

# Set title with command name
echo -ne "\033]0;$(basename $(pwd))\007"
```

### Python

```python
import sys

# Set title
sys.stdout.write("\033]0;My Python Script\007")
sys.stdout.flush()
```

### Rust

```rust
use std::io::{self, Write};

fn set_title(title: &str) -> io::Result<()> {
    print!("\x1b]0;{}\x07", title);
    io::stdout().flush()
}

fn main() -> io::Result<()> {
    set_title("My Rust App")?;
    // ... your code ...
    Ok(())
}
```

### C

```c
#include <stdio.h>

void set_title(const char *title) {
    printf("\033]0;%s\007", title);
    fflush(stdout);
}

int main() {
    set_title("My C Program");
    // ... your code ...
    return 0;
}
```

## Shell Integration

### Bash

Add to `~/.bashrc`:

```bash
# Simple: Show current directory
PROMPT_COMMAND='echo -ne "\033]0;${PWD/#$HOME/~}\007"'

# Advanced: Show user@host:directory
PROMPT_COMMAND='echo -ne "\033]0;${USER}@${HOSTNAME%%.*}:${PWD/#$HOME/~}\007"'

# With git branch
function set_title() {
    local branch=$(git branch 2>/dev/null | grep '^*' | cut -d' ' -f2)
    if [ -n "$branch" ]; then
        echo -ne "\033]0;${PWD/#$HOME/~} (${branch})\007"
    else
        echo -ne "\033]0;${PWD/#$HOME/~}\007"
    fi
}
PROMPT_COMMAND='set_title'
```

### Zsh

Add to `~/.zshrc`:

```zsh
# Simple: Show current directory
precmd() {
    echo -ne "\033]0;${PWD/#$HOME/~}\007"
}

# Advanced: Show command before execution
preexec() {
    echo -ne "\033]0;$1\007"
}

precmd() {
    echo -ne "\033]0;${PWD/#$HOME/~}\007"
}

# With git branch
function set_title() {
    local branch=$(git branch 2>/dev/null | grep '^*' | cut -d' ' -f2)
    if [ -n "$branch" ]; then
        echo -ne "\033]0;${PWD/#$HOME/~} (${branch})\007"
    else
        echo -ne "\033]0;${PWD/#$HOME/~}\007"
    fi
}

precmd() { set_title }
```

### Fish

Add to `~/.config/fish/config.fish`:

```fish
# Simple: Show current directory
function fish_title
    echo $PWD | sed "s|^$HOME|~|"
end

# Advanced: Show command
function fish_title
    if test -n "$_"
        echo $_
    else
        prompt_pwd
    end
end
```

## Testing

### Quick Test

```bash
# Run this in AgTerm to test title updates
echo -ne "\033]0;Test 1\007"; sleep 2
echo -ne "\033]0;Test 2\007"; sleep 2
echo -ne "\033]0;Test 3\007"; sleep 2
```

### Test Script

Use the included test script:

```bash
./test_title.sh
```

Or run the example:

```bash
cargo run --example title_demo
```

## Troubleshooting

### Title Not Updating

1. **Check if OSC sequences are being sent:**
   ```bash
   echo -ne "\033]0;Test\007" | cat -v
   # Should show: ^[]0;Test^G
   ```

2. **Verify terminal type:**
   ```bash
   echo $TERM
   # Should be: xterm-256color or similar
   ```

3. **Enable debug logging:**
   ```bash
   RUST_LOG=agterm::floem_app=debug cargo run
   ```

### Title Updates Slowly

The title updates every 500ms. This is a balance between responsiveness and performance. If you need faster updates, modify `src/floem_app/mod.rs`:

```rust
std::thread::sleep(std::time::Duration::from_millis(250)); // 250ms = faster
```

### Title Shows Wrong Text

Make sure your shell is not overwriting the title. Check for `PROMPT_COMMAND` (bash) or `precmd/preexec` (zsh) that might be resetting the title.

## Advanced Usage

### Conditional Titles

Show different titles based on context:

```bash
# In ~/.bashrc
function set_title() {
    if [ -n "$SSH_CLIENT" ]; then
        # Remote session
        echo -ne "\033]0;[SSH] ${HOSTNAME}:${PWD/#$HOME/~}\007"
    elif git rev-parse --git-dir > /dev/null 2>&1; then
        # Git repository
        local branch=$(git branch --show-current)
        echo -ne "\033]0;[Git] ${PWD/#$HOME/~} ($branch)\007"
    else
        # Regular directory
        echo -ne "\033]0;${PWD/#$HOME/~}\007"
    fi
}
PROMPT_COMMAND='set_title'
```

### Dynamic Colors

Combine with terminal colors:

```bash
# Show error status in title
function set_title() {
    local status=$?
    if [ $status -eq 0 ]; then
        echo -ne "\033]0;✓ ${PWD/#$HOME/~}\007"
    else
        echo -ne "\033]0;✗ ${PWD/#$HOME/~} [exit: $status]\007"
    fi
}
PROMPT_COMMAND='set_title'
```

### Multi-Tab Workflow

Name your tabs based on purpose:

```bash
# In tab 1
echo -ne "\033]0;[DEV] Main Project\007"

# In tab 2
echo -ne "\033]0;[TEST] Running Tests\007"

# In tab 3
echo -ne "\033]0;[LOG] Server Logs\007"
```

## Best Practices

1. **Keep titles short**: Long titles may be truncated
2. **Use prefixes**: `[DEV]`, `[PROD]`, etc. for context
3. **Update on directory change**: Use `cd()` function
4. **Avoid special characters**: Some chars may not display correctly
5. **Reset on exit**: Clear custom titles when exiting scripts

## See Also

- [DYNAMIC_TITLE_IMPLEMENTATION.md](../DYNAMIC_TITLE_IMPLEMENTATION.md) - Implementation details
- [XTerm Control Sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) - Official OSC documentation
- [Terminal Escape Codes](https://en.wikipedia.org/wiki/ANSI_escape_code) - General escape sequence reference
