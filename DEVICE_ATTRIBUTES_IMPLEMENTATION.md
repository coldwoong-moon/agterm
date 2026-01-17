# Device Attributes Implementation

## Summary

Implemented comprehensive Device Attributes response functionality for AgTerm terminal emulator. Many terminal applications (vim, htop, tmux, etc.) query the terminal's capabilities and position using standard escape sequences. Without proper responses, these applications may not function correctly or may experience delays.

## Implementation Details

### 1. Data Structure Changes

**File: `src/terminal/screen.rs`**

Added a new field to `TerminalScreen` struct:
```rust
/// Pending responses to be sent to PTY (for DA, DSR, CPR, etc.)
pending_responses: Vec<String>,
```

This queue stores responses that need to be sent back to the PTY when the terminal receives query sequences.

### 2. Core Functionality

#### Added Method: `take_pending_responses()`
```rust
/// Take pending responses (for sending to PTY)
/// This drains the pending_responses vec and returns it
pub fn take_pending_responses(&mut self) -> Vec<String>
```

This method retrieves and clears all pending responses, allowing them to be sent to the PTY.

#### Enhanced CSI Dispatch Handler

Added support for the following terminal queries in `csi_dispatch()`:

1. **Primary DA (DA1) - CSI c**
   - Query: `\x1b[c`
   - Response: `\x1b[?1;2c` (VT100 with Advanced Video Option)
   - Purpose: Terminal identification

2. **Secondary DA (DA2) - CSI > c**
   - Query: `\x1b[>c`
   - Response: `\x1b[>0;0;0c` (VT100 compatible)
   - Purpose: Extended terminal identification

3. **Device Status Report (DSR) - CSI 5 n**
   - Query: `\x1b[5n`
   - Response: `\x1b[0n` (Terminal OK)
   - Purpose: Check terminal status

4. **Cursor Position Report (CPR) - CSI 6 n**
   - Query: `\x1b[6n`
   - Response: `\x1b[<row>;<col>R` (current cursor position with 1-based indexing)
   - Purpose: Report current cursor position to application

### 3. Integration with Main Application

**File: `src/main.rs`**

Modified the `Message::Tick` handler to send pending responses to PTY after processing incoming data:

```rust
// Process bytes through VTE parser
tab.screen.process(&data);

// Send pending responses (DA, DSR, CPR, etc.) to PTY
let pending_responses = tab.screen.take_pending_responses();
for response in pending_responses {
    let _ = self.pty_manager.write(session_id, response.as_bytes());
}
```

This ensures responses are sent immediately after the terminal processes query sequences.

## Testing

### Unit Tests Added

All tests are located in `src/terminal/screen.rs`:

1. **test_device_attributes_primary_da1**
   - Tests Primary DA (CSI c) response

2. **test_device_attributes_secondary_da2**
   - Tests Secondary DA (CSI > c) response

3. **test_device_status_report_dsr**
   - Tests Device Status Report (CSI 5 n)

4. **test_cursor_position_report_cpr**
   - Tests Cursor Position Report at a specific position

5. **test_cpr_at_origin**
   - Tests CPR at cursor position (0,0) → responds with (1,1)

6. **test_multiple_device_attribute_requests**
   - Tests handling multiple queries in sequence

7. **test_take_pending_responses_clears_queue**
   - Tests that pending responses are cleared after retrieval

8. **test_device_attributes_with_regular_content**
   - Tests that DA queries don't interfere with regular terminal content

### Test Results

All 8 new tests pass successfully:
```
test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured
```

### Manual Testing

A test script is provided: `test_device_attributes.sh`

Run it in AgTerm to verify DA functionality:
```bash
./test_device_attributes.sh
```

## Benefits

1. **Better Application Compatibility**: Applications like vim, htop, tmux now receive proper terminal responses
2. **Faster Startup**: Apps no longer need to timeout waiting for terminal responses
3. **Accurate Cursor Tracking**: Applications can query cursor position for UI alignment
4. **Standards Compliant**: Implements VT100-compatible responses

## Technical Notes

### 1-Based vs 0-Based Indexing

The terminal internally uses 0-based indexing for cursor positions, but CPR responses use VT100's 1-based indexing:
- Internal position (0, 0) → CPR response `\x1b[1;1R`
- Internal position (5, 10) → CPR response `\x1b[6;11R`

### Response Timing

Responses are queued immediately when processing escape sequences and sent on the next tick cycle when PTY data is processed. This ensures proper ordering and prevents race conditions.

### Thread Safety

The pending_responses Vec is owned by TerminalScreen and accessed only from the main thread, so no synchronization primitives are needed.

## Future Enhancements

Possible future improvements:

1. Add support for more DA variants (DA3, etc.)
2. Implement DECREQTPARM (Terminal Parameters)
3. Add support for DECRQSS (Request Status String)
4. Implement XTVERSION (XTerm version query)

## Files Modified

1. `src/terminal/screen.rs` - Core implementation and tests
2. `src/main.rs` - Integration with PTY manager
3. `test_device_attributes.sh` - Manual test script (new)

## Conclusion

This implementation provides essential terminal query response functionality that significantly improves compatibility with interactive terminal applications. All standard DA/DSR/CPR queries are now handled correctly.
