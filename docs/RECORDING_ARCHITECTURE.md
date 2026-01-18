# Recording Module Architecture

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     AgTerm Application                      │
└─────────────────────────────────────────────────────────────┘
                           │
        ┌──────────────────┴──────────────────┐
        │                                     │
        ▼                                     ▼
┌───────────────┐                    ┌───────────────┐
│   Recording   │                    │    Player     │
│               │                    │               │
│  - Events     │◄───save/load──────►│  - Position   │
│  - Metadata   │   (asciicast v2)   │  - Speed      │
│  - State      │                    │  - State      │
└───────────────┘                    └───────────────┘
        │                                     │
        └──────────────────┬──────────────────┘
                           ▼
                  ┌─────────────────┐
                  │ RecordingEvent  │
                  │  - Output       │
                  │  - Input        │
                  │  - Resize       │
                  └─────────────────┘
```

## Data Flow

### Recording Flow

```
PTY Output ──┐
             │
User Input ──┼──► Recording.record_*() ──► Events[] ──► save_to_file()
             │                                                │
Resize ──────┘                                                ▼
                                                      session.cast
                                                      (asciicast v2)
```

### Playback Flow

```
session.cast ──► load_from_file() ──► Recording ──► RecordingPlayer
                                                           │
                                                           ▼
                                      Player.update() (called at 60 FPS)
                                                           │
                                                           ▼
                                      Check current_time vs event timestamps
                                                           │
                                                           ▼
                                      Return events to emit this frame
                                                           │
                    ┌──────────────────┬─────────────────┴─────────┐
                    ▼                  ▼                           ▼
              Output events      Input events                Resize events
                    │                  │                           │
                    ▼                  ▼                           ▼
            Terminal.write()   (optional logging)        Terminal.resize()
```

## Class Diagram

```
┌────────────────────────┐
│   RecordingEvent       │
├────────────────────────┤
│ + Output { time, data }│
│ + Input { time, data } │
│ + Resize { time, cols, │
│            rows }      │
├────────────────────────┤
│ + timestamp() Duration │
│ + is_output() bool     │
│ + is_input() bool      │
│ + is_resize() bool     │
│ + data() Option<&str>  │
└────────────────────────┘
           △
           │
           │ contains
           │
┌──────────┴─────────────┐          ┌────────────────────────┐
│   Recording            │          │  RecordingMetadata     │
├────────────────────────┤          ├────────────────────────┤
│ - events: Vec<Event>   │◄─────────│ + version: u32         │
│ - metadata: Metadata   │          │ + width: u16           │
│ - start_time: Instant  │          │ + height: u16          │
│ - is_recording: bool   │          │ + timestamp: DateTime  │
├────────────────────────┤          │ + duration: f64        │
│ + new(cols, rows)      │          │ + title: String        │
│ + start()              │          │ + command: String      │
│ + stop()               │          │ + theme: Theme         │
│ + add_output()         │          └────────────────────────┘
│ + add_input()          │
│ + add_resize()         │
│ + record_output()      │          ┌────────────────────────┐
│ + save_to_file()       │          │  RecordingStats        │
│ + load_from_file()     │          ├────────────────────────┤
│ + compress()           │          │ + duration: Duration   │
│ + stats()              │──────────►│ + output_events: usize │
│ + duration()           │          │ + output_bytes: usize  │
└────────────────────────┘          │ + input_events: usize  │
           △                        │ + input_bytes: usize   │
           │ used by                │ + resize_events: usize │
           │                        └────────────────────────┘
┌──────────┴─────────────┐
│  RecordingPlayer       │          ┌────────────────────────┐
├────────────────────────┤          │   PlayerState (enum)   │
│ - recording: Recording │          ├────────────────────────┤
│ - position: usize      │          │ + Stopped              │
│ - current_time: Dur.   │◄─────────│ + Playing              │
│ - speed: f64           │          │ + Paused               │
│ - state: PlayerState   │          └────────────────────────┘
│ - last_update: Instant │
├────────────────────────┤
│ + new(recording)       │
│ + play()               │
│ + pause()              │
│ + stop()               │
│ + set_speed(f64)       │
│ + seek(Duration)       │
│ + skip_forward()       │
│ + skip_backward()      │
│ + update()             │──────► Vec<RecordingEvent>
│ + current_time()       │
│ + progress()           │
│ + is_finished()        │
└────────────────────────┘
```

## State Machine

### Recording State

```
     ┌─────────┐
     │ Created │
     └────┬────┘
          │ start()
          ▼
     ┌─────────┐
     │Recording│◄───┐
     └────┬────┘    │ Events can be added
          │         │ only in this state
          │ stop()  │
          ▼         │
     ┌─────────┐    │
     │ Stopped │────┘
     └─────────┘
```

### Player State

```
     ┌─────────┐
     │ Stopped │◄────────┐
     └────┬────┘         │
          │ play()       │
          ▼              │
     ┌─────────┐         │
     │ Playing │         │ stop()
     └────┬────┘         │
          │ pause()      │
          ▼              │
     ┌─────────┐         │
     │ Paused  │─────────┘
     └─────────┘
          │ play()
          └──────► (back to Playing)
```

## File Format (asciicast v2)

```
┌────────────────────────────────────────────┐
│ Line 1: Header (JSON Object)               │
│ {                                          │
│   "version": 2,                            │
│   "width": 80,                             │
│   "height": 24,                            │
│   "timestamp": "2026-01-18T10:30:00Z",     │
│   "duration": 10.5,                        │
│   "title": "Session Title",                │
│   ...                                      │
│ }                                          │
├────────────────────────────────────────────┤
│ Line 2-N: Events (JSON Arrays)             │
│ [0.0, "o", "$ ls\r\n"]                     │
│ [0.1, "o", "file1.txt file2.txt\r\n"]      │
│ [0.5, "i", "ls\n"]                         │
│ [1.0, "r", "120x40"]                       │
│ ...                                        │
└────────────────────────────────────────────┘

Event Format:
┌──────────┬──────────┬─────────────────┐
│ Time (f64)│ Type(str)│  Data (string)  │
├──────────┼──────────┼─────────────────┤
│  0.0     │   "o"    │ "output data"   │
│  0.1     │   "i"    │ "input data"    │
│  0.2     │   "r"    │ "COLSxROWS"     │
└──────────┴──────────┴─────────────────┘
```

## Time Management

```
Recording:
    start_time (Instant) ────► elapsed() ────► Duration
                                                   │
                                                   ▼
                                         Event timestamp (f64)

Playback:
    last_update (Instant) ────► delta ────► scaled by speed
                                                   │
                                                   ▼
                                            current_time
                                                   │
                                                   ▼
                                    Compare with event timestamps
                                                   │
                                                   ▼
                                        Emit events <= current_time
```

## Integration Points

### With Terminal

```
Terminal
   │
   ├──► process_data(bytes) ◄───┐
   │                            │
   └──► resize(cols, rows) ◄────┤
                                │
                         RecordingEvent
                         (from Player.update())
```

### With PTY

```
PTY
   │
   ├──► read() ──────► Recording.record_output()
   │
   ├──► write() ◄───── Recording.record_input()
   │
   └──► resize() ────► Recording.record_resize()
```

## Memory Layout

### Recording

```
Recording (heap)
├── metadata: RecordingMetadata (stack)
│   └── ~100 bytes
├── events: Vec<RecordingEvent>
│   ├── capacity * size_of::<RecordingEvent>()
│   └── each event: ~50 bytes + data.len()
├── start_time: Option<Instant> (16 bytes)
└── is_recording: bool (1 byte)

Total: ~120 bytes + (events.len() * (50 + avg_data_size))
```

### Player

```
RecordingPlayer (stack/heap)
├── recording: Recording (heap)
├── position: usize (8 bytes)
├── current_time: Duration (16 bytes)
├── speed: f64 (8 bytes)
├── state: PlayerState (1 byte)
├── event_queue: VecDeque<Event> (heap)
└── last_update: Option<Instant> (16 bytes)

Total: ~50 bytes + recording size + queue size
```

## Performance Characteristics

| Operation           | Time Complexity | Space Complexity |
|---------------------|----------------|------------------|
| add_event()         | O(1)           | O(1)             |
| save_to_file()      | O(n)           | O(1)             |
| load_from_file()    | O(n)           | O(n)             |
| compress()          | O(n)           | O(n)             |
| player.update()     | O(k)*          | O(k)             |
| player.seek()       | O(n)           | O(1)             |
| player.set_speed()  | O(1)           | O(1)             |

*k = number of events in current frame (typically small)

## Thread Safety

All types are `!Send` and `!Sync` by default (contains `Instant`).

For multi-threaded usage:
- Wrap in `Arc<Mutex<>>` for shared mutable access
- Use message passing for cross-thread communication
- Consider separate recording threads for high-frequency events

## Error Flow

```
Operation
    │
    ▼
Result<T, RecordingError>
    │
    ├──► Ok(value) ──────► Success
    │
    └──► Err(error)
           │
           ├──► RecordingError::Io(io::Error)
           ├──► RecordingError::Json(serde_json::Error)
           ├──► RecordingError::InvalidFormat(String)
           └──► RecordingError::Compression(String)
```

## Extension Points

### Custom Event Types

```rust
// Future: Add custom event types
enum RecordingEvent {
    Output { ... },
    Input { ... },
    Resize { ... },
    // Custom events
    Annotation { timestamp: f64, text: String },
    Bookmark { timestamp: f64, label: String },
}
```

### Streaming Support

```rust
// Future: Streaming for large files
struct RecordingStream {
    reader: BufReader<File>,
    // Stream events on-demand
}
```

### Export Formats

```rust
// Future: Export to other formats
trait RecordingExporter {
    fn export(&self, recording: &Recording) -> Result<(), ExportError>;
}

struct GifExporter;
struct Mp4Exporter;
```

## Testing Strategy

```
Unit Tests (in recording.rs)
├── Recording creation
├── Event addition
├── File I/O
├── Compression
├── Statistics
├── Player controls
├── Speed control
├── Seeking
├── Progress tracking
└── Format compliance

Integration Tests (future)
├── With Terminal
├── With PTY
├── With UI controls
└── End-to-end recording/playback

Examples
└── recording_demo.rs
```
