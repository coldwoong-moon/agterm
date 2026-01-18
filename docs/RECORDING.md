# Terminal Session Recording

AgTerm supports recording and playing back terminal sessions in the asciicast v2 format, compatible with tools like [asciinema](https://asciinema.org/).

## Features

- **Record Terminal Sessions**: Capture all input, output, and resize events
- **Playback Controls**: Play, pause, stop, seek, and speed control
- **asciicast v2 Compatible**: Save and load recordings in the standard format
- **Compression**: Remove redundant events to reduce file size
- **Statistics**: Analyze recording content and duration

## Recording API

### Creating a Recording

```rust
use agterm::recording::Recording;
use std::time::Duration;

// Create a new recording with terminal dimensions
let mut recording = Recording::new(80, 24);

// Start recording
recording.start();

// Add events
recording.add_output(Duration::from_secs(0), b"Hello, world!\n");
recording.add_input(Duration::from_millis(100), b"ls\n");
recording.add_resize(Duration::from_millis(200), 120, 40);

// Or use automatic timestamping
recording.record_output(b"Output data");
recording.record_input(b"Input data");
recording.record_resize(100, 30);

// Stop recording
recording.stop();
```

### Saving and Loading

```rust
// Save to asciicast v2 format
recording.save_to_file("session.cast")?;

// Load from file
let recording = Recording::load_from_file("session.cast")?;
```

### Recording Metadata

You can customize recording metadata:

```rust
recording.metadata.title = Some("My Session".to_string());
recording.metadata.command = Some("/bin/bash".to_string());
recording.metadata.idle_time_limit = Some(2.0);
```

### Recording Statistics

```rust
let stats = recording.stats();
println!("Duration: {:?}", stats.duration);
println!("Output events: {}", stats.output_events);
println!("Input events: {}", stats.input_events);
println!("Resize events: {}", stats.resize_events);
```

### Compression

Remove redundant resize events to reduce file size:

```rust
recording.compress();
```

## Playback API

### Creating a Player

```rust
use agterm::recording::{RecordingPlayer, PlayerState};

let player = RecordingPlayer::new(recording);
```

### Playback Controls

```rust
// Start playback
player.play();

// Pause playback
player.pause();

// Stop and reset
player.stop();

// Check state
if player.state() == PlayerState::Playing {
    println!("Playing...");
}
```

### Speed Control

Control playback speed from 0.5x to 4x:

```rust
player.set_speed(2.0);  // 2x speed
player.set_speed(0.5);  // Half speed
let speed = player.speed();
```

### Seeking

Jump to specific timestamps:

```rust
use std::time::Duration;

// Seek to specific time
player.seek(Duration::from_secs(5));

// Skip forward/backward
player.skip_forward(Duration::from_secs(1));
player.skip_backward(Duration::from_millis(500));

// Get current position
let current = player.current_time();
let progress = player.progress(); // 0.0 to 1.0
```

### Update Loop

In your application's update loop:

```rust
// Update player and get events
let events = player.update();

for event in events {
    match event {
        RecordingEvent::Output { timestamp, data } => {
            // Process output
            terminal.write(data.as_bytes());
        }
        RecordingEvent::Input { timestamp, data } => {
            // Process input (if needed)
        }
        RecordingEvent::Resize { timestamp, cols, rows } => {
            // Handle resize
            terminal.resize(cols, rows);
        }
    }
}

// Check if finished
if player.is_finished() {
    println!("Playback complete!");
}
```

## asciicast v2 Format

The asciicast v2 format consists of:

1. **Header line** (JSON): Recording metadata
2. **Event lines** (JSON arrays): `[timestamp, event_type, data]`

### Example File

```json
{"version":2,"width":80,"height":24,"timestamp":"2026-01-18T10:30:00Z"}
[0.0,"o","$ echo hello\r\n"]
[0.1,"o","hello\r\n"]
[0.5,"i","ls\n"]
[1.0,"o","file1.txt file2.txt\r\n"]
[2.0,"r","120x40"]
```

### Event Types

- `"o"` - Output from terminal
- `"i"` - Input to terminal
- `"r"` - Resize event (format: "COLSxROWS")

## Integration Example

### Recording Terminal Sessions

```rust
use agterm::recording::Recording;
use agterm::terminal::pty::PtyManager;

// Create PTY and recording
let pty_manager = PtyManager::new();
let pty_id = pty_manager.create_session(80, 24)?;
let mut recording = Recording::new(80, 24);

recording.start();

// In your event loop
loop {
    // Read from PTY
    if let Ok(output) = pty_manager.read(&pty_id) {
        // Process output in terminal
        terminal.process_data(&output);

        // Record output
        recording.record_output(&output);
    }

    // Handle user input
    if let Some(input) = get_user_input() {
        // Send to PTY
        pty_manager.write(&pty_id, &input)?;

        // Record input
        recording.record_input(&input);
    }

    // Handle resize
    if let Some((cols, rows)) = check_resize() {
        pty_manager.resize(&pty_id, rows, cols)?;
        recording.record_resize(cols, rows);
    }
}

// Save when done
recording.stop();
recording.save_to_file("session.cast")?;
```

### Playing Back Recordings

```rust
use agterm::recording::{Recording, RecordingPlayer};

// Load recording
let recording = Recording::load_from_file("session.cast")?;
let mut player = RecordingPlayer::new(recording);

// Initialize terminal with recording dimensions
let (cols, rows) = (
    player.recording().metadata.width,
    player.recording().metadata.height,
);
terminal.resize(cols, rows);

// Start playback
player.play();

// In your render loop (e.g., 60 FPS)
loop {
    // Update player
    let events = player.update();

    // Process events
    for event in events {
        match event {
            RecordingEvent::Output { data, .. } => {
                terminal.process_data(data.as_bytes());
            }
            RecordingEvent::Resize { cols, rows, .. } => {
                terminal.resize(cols, rows);
            }
            _ => {}
        }
    }

    // Render terminal
    terminal.render();

    // Check if finished
    if player.is_finished() {
        break;
    }

    // Sleep to maintain frame rate
    std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
}
```

## UI Controls

Common UI controls for playback:

```rust
// Play/Pause toggle
if play_pause_button.clicked() {
    if player.state() == PlayerState::Playing {
        player.pause();
    } else {
        player.play();
    }
}

// Progress bar seeking
if progress_bar.clicked() {
    let position = progress_bar.click_position(); // 0.0 to 1.0
    let seek_time = player.duration().mul_f64(position);
    player.seek(seek_time);
}

// Speed control
if speed_dropdown.changed() {
    let speed = speed_dropdown.value(); // 0.5, 1.0, 2.0, 4.0
    player.set_speed(speed);
}

// Skip buttons
if skip_back_button.clicked() {
    player.skip_backward(Duration::from_secs(5));
}
if skip_forward_button.clicked() {
    player.skip_forward(Duration::from_secs(5));
}
```

## Performance Considerations

1. **Event Frequency**: The player processes all events up to the current timestamp. High-frequency output can impact performance.

2. **Update Rate**: Call `player.update()` at your desired frame rate (e.g., 60 FPS). The player handles timing automatically.

3. **Compression**: Use `recording.compress()` before saving to reduce file size and improve playback performance.

4. **Memory**: Large recordings with many events may consume significant memory. Consider streaming for very large files.

## Error Handling

```rust
use agterm::recording::RecordingError;

match recording.save_to_file("session.cast") {
    Ok(_) => println!("Saved successfully"),
    Err(RecordingError::Io(e)) => eprintln!("IO error: {}", e),
    Err(RecordingError::Json(e)) => eprintln!("JSON error: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Compatibility

The recording format is compatible with:

- [asciinema](https://asciinema.org/) - Terminal session recorder and player
- [asciinema-player](https://github.com/asciinema/asciinema-player) - Web player
- [terminalizer](https://github.com/faressoft/terminalizer) - Terminal recorder

You can record with AgTerm and play back with asciinema or vice versa.

## Testing

The recording module includes comprehensive tests:

```bash
# Run recording tests
cargo test --lib recording

# Run example
cargo run --example recording_demo
```

## Future Enhancements

Potential improvements for future versions:

- [ ] Streaming large files without loading entire recording into memory
- [ ] Support for asciicast v1 format
- [ ] Export to GIF/MP4 video formats
- [ ] Live recording indicators in UI
- [ ] Multiple recording tracks (e.g., separate audio)
- [ ] Recording annotations and bookmarks
- [ ] Editing capabilities (trim, splice, etc.)
