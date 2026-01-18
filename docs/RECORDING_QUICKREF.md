# Recording Quick Reference

## Basic Recording

```rust
use agterm::recording::Recording;

// Create
let mut rec = Recording::new(80, 24);

// Record
rec.start();
rec.record_output(b"data");
rec.record_input(b"input");
rec.record_resize(120, 40);
rec.stop();

// Save/Load
rec.save_to_file("session.cast")?;
let rec = Recording::load_from_file("session.cast")?;
```

## Basic Playback

```rust
use agterm::recording::{RecordingPlayer, PlayerState};

// Create player
let mut player = RecordingPlayer::new(recording);

// Controls
player.play();
player.pause();
player.stop();

// Update loop
let events = player.update(); // Call at ~60 FPS
for event in events {
    // Process event
}
```

## Speed Control

```rust
player.set_speed(2.0);   // 2x speed
player.set_speed(0.5);   // Half speed
let speed = player.speed();
```

## Seeking

```rust
use std::time::Duration;

player.seek(Duration::from_secs(5));
player.skip_forward(Duration::from_secs(1));
player.skip_backward(Duration::from_millis(500));

let current = player.current_time();
let progress = player.progress(); // 0.0 to 1.0
```

## Statistics

```rust
let stats = recording.stats();
println!("Duration: {:?}", stats.duration);
println!("Output events: {}", stats.output_events);
println!("Input events: {}", stats.input_events);
```

## Compression

```rust
recording.compress(); // Remove duplicate resize events
```

## Event Types

```rust
match event {
    RecordingEvent::Output { timestamp, data } => {
        // Handle output
    }
    RecordingEvent::Input { timestamp, data } => {
        // Handle input
    }
    RecordingEvent::Resize { timestamp, cols, rows } => {
        // Handle resize
    }
}
```

## Metadata

```rust
rec.metadata.title = Some("My Session".to_string());
rec.metadata.command = Some("/bin/bash".to_string());
rec.metadata.idle_time_limit = Some(2.0);
```

## Error Handling

```rust
use agterm::recording::RecordingError;

match rec.save_to_file("file.cast") {
    Ok(_) => println!("Saved"),
    Err(RecordingError::Io(e)) => eprintln!("IO: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

## asciicast v2 Format

```
{"version":2,"width":80,"height":24}
[0.0,"o","output"]
[0.1,"i","input"]
[0.2,"r","120x40"]
```

## Running Tests & Examples

```bash
cargo test --lib recording
cargo run --example recording_demo
```

## Speed Limits

- Minimum: 0.5x (half speed)
- Maximum: 4.0x (quad speed)
- Values automatically clamped

## State Checks

```rust
if player.state() == PlayerState::Playing { }
if player.is_finished() { }
if recording.is_recording() { }
if recording.is_empty() { }
```
