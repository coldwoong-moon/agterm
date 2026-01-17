# /agterm-debug - AgTerm Debug Mode

Launch AgTerm in debug mode with enhanced logging and debug panel.

## Activation

This skill is invoked when user types `/agterm-debug` or requests to debug the AgTerm application.

## Instructions

When this skill is activated:

1. **Set up debug environment:**
   ```bash
   export AGTERM_DEBUG=1
   export AGTERM_LOG="agterm=debug,agterm::terminal::pty=trace"
   ```

2. **Build with debug profile:**
   ```bash
   cargo build 2>&1
   ```

3. **Run the application:**
   ```bash
   AGTERM_DEBUG=1 AGTERM_LOG="agterm=debug" cargo run 2>&1 &
   ```

4. **Monitor logs:**
   - Log files are stored in `~/.local/share/agterm/logs/`
   - Use `tail -f` to follow logs in real-time

## Debug Panel Controls

Inform the user about debug panel controls:
- `Cmd+D` or `F12`: Toggle debug panel
- Debug panel shows: FPS, PTY stats, input state, recent logs

## Troubleshooting Commands

If user reports issues, use these diagnostic commands:

```bash
# Check if app is running
pgrep -f agterm

# View recent logs
tail -100 ~/.local/share/agterm/logs/agterm.log.*

# Check for crashes
dmesg | grep -i agterm

# Memory usage
ps aux | grep agterm
```

## Output Format

After launching, report:
1. Build status (success/failure)
2. Process ID if running
3. Log file location
4. Debug panel activation status
