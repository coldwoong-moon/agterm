# Recording Feature UI Integration Guide

This guide shows how to integrate the recording feature into AgTerm's Iced-based UI.

## UI Components

### Recording Controls

```
┌─────────────────────────────────────────────────────────┐
│  AgTerm Terminal                                    [●] │  ← Recording indicator
├─────────────────────────────────────────────────────────┤
│                                                         │
│  $ ls -la                                               │
│  drwxr-xr-x  12 user  staff   384 Jan 18 10:30 .       │
│  drwxr-xr-x   5 user  staff   160 Jan 18 09:15 ..      │
│                                                         │
└─────────────────────────────────────────────────────────┘

Playback Controls:
┌─────────────────────────────────────────────────────────┐
│  [◄◄] [▶/❚❚] [►►]   ●━━━━━○━━━━━━━━━━━━━●  [1.0x ▼]    │
│  Skip  Play   Skip    Progress Bar         Speed        │
│  -5s          +5s     0:05 / 0:10                       │
└─────────────────────────────────────────────────────────┘
```

## State Management

### Application State

```rust
use agterm::recording::{Recording, RecordingPlayer, PlayerState};
use std::time::Duration;

pub struct AgTerm {
    // Existing fields
    terminal: Terminal,
    pty_manager: PtyManager,

    // Recording state
    recording_state: RecordingState,
}

pub enum RecordingState {
    Idle,
    Recording {
        recording: Recording,
    },
    Playing {
        player: RecordingPlayer,
    },
}

impl Default for RecordingState {
    fn default() -> Self {
        Self::Idle
    }
}
```

## Messages

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // Existing messages
    TerminalOutput(Vec<u8>),
    UserInput(String),
    WindowResized(u16, u16),

    // Recording messages
    StartRecording,
    StopRecording,
    SaveRecording(PathBuf),
    LoadRecording(PathBuf),

    // Playback messages
    PlayPause,
    Stop,
    Seek(f64),          // Seek to percentage (0.0 to 1.0)
    SkipForward,
    SkipBackward,
    SetSpeed(f64),
    PlaybackUpdate,     // Called every frame
}
```

## Message Handlers

### Recording

```rust
impl AgTerm {
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartRecording => {
                // Create new recording
                let (cols, rows) = self.terminal.size();
                let mut recording = Recording::new(cols, rows);
                recording.start();

                self.recording_state = RecordingState::Recording { recording };

                // Show recording indicator
                self.show_notification("Recording started");

                Command::none()
            }

            Message::StopRecording => {
                if let RecordingState::Recording { mut recording } =
                    std::mem::take(&mut self.recording_state)
                {
                    recording.stop();

                    // Open save dialog
                    Command::perform(
                        Self::save_recording_dialog(recording),
                        Message::SaveRecording,
                    )
                } else {
                    Command::none()
                }
            }

            Message::SaveRecording(path) => {
                if let RecordingState::Recording { recording } = &self.recording_state {
                    match recording.save_to_file(&path) {
                        Ok(_) => {
                            self.show_notification(&format!(
                                "Recording saved: {}",
                                path.display()
                            ));
                        }
                        Err(e) => {
                            self.show_error(&format!("Save failed: {}", e));
                        }
                    }
                }
                self.recording_state = RecordingState::Idle;
                Command::none()
            }

            Message::LoadRecording(path) => {
                match Recording::load_from_file(&path) {
                    Ok(recording) => {
                        let player = RecordingPlayer::new(recording);

                        // Resize terminal to match recording
                        let (cols, rows) = (
                            player.recording().metadata.width,
                            player.recording().metadata.height,
                        );
                        self.terminal.resize(cols, rows);

                        self.recording_state = RecordingState::Playing { player };
                        self.show_notification("Recording loaded");

                        // Start update loop
                        Command::perform(
                            async { Duration::from_millis(16) },
                            |_| Message::PlaybackUpdate,
                        )
                    }
                    Err(e) => {
                        self.show_error(&format!("Load failed: {}", e));
                        Command::none()
                    }
                }
            }

            // Handle other messages...
            _ => Command::none(),
        }
    }
}
```

### Playback

```rust
impl AgTerm {
    fn handle_playback_message(&mut self, message: Message) -> Command<Message> {
        if let RecordingState::Playing { ref mut player } = self.recording_state {
            match message {
                Message::PlayPause => {
                    match player.state() {
                        PlayerState::Playing => player.pause(),
                        PlayerState::Paused | PlayerState::Stopped => player.play(),
                    }
                    Command::none()
                }

                Message::Stop => {
                    player.stop();
                    self.terminal.clear();
                    Command::none()
                }

                Message::Seek(percentage) => {
                    let target = player.duration().mul_f64(percentage);
                    player.seek(target);
                    Command::none()
                }

                Message::SkipForward => {
                    player.skip_forward(Duration::from_secs(5));
                    Command::none()
                }

                Message::SkipBackward => {
                    player.skip_backward(Duration::from_secs(5));
                    Command::none()
                }

                Message::SetSpeed(speed) => {
                    player.set_speed(speed);
                    Command::none()
                }

                Message::PlaybackUpdate => {
                    // Update player and get events
                    let events = player.update();

                    // Process events
                    for event in events {
                        match event {
                            RecordingEvent::Output { data, .. } => {
                                self.terminal.process_data(data.as_bytes());
                            }
                            RecordingEvent::Resize { cols, rows, .. } => {
                                self.terminal.resize(cols, rows);
                            }
                            _ => {}
                        }
                    }

                    // Schedule next update if still playing
                    if player.state() == PlayerState::Playing && !player.is_finished() {
                        Command::perform(
                            async { Duration::from_millis(16) },
                            |_| Message::PlaybackUpdate,
                        )
                    } else if player.is_finished() {
                        self.show_notification("Playback finished");
                        Command::none()
                    } else {
                        Command::none()
                    }
                }

                _ => Command::none(),
            }
        } else {
            Command::none()
        }
    }
}
```

### Capturing Events

```rust
impl AgTerm {
    // Called when PTY has output
    fn handle_pty_output(&mut self, output: Vec<u8>) {
        // Process in terminal
        self.terminal.process_data(&output);

        // Record if recording
        if let RecordingState::Recording { ref mut recording } = self.recording_state {
            recording.record_output(&output);
        }
    }

    // Called when user types
    fn handle_user_input(&mut self, input: String) {
        let bytes = input.as_bytes();

        // Send to PTY
        self.pty_manager.write(&self.active_pty_id, bytes).ok();

        // Record if recording
        if let RecordingState::Recording { ref mut recording } = self.recording_state {
            recording.record_input(bytes);
        }
    }

    // Called when window resized
    fn handle_resize(&mut self, cols: u16, rows: u16) {
        // Resize PTY
        self.pty_manager.resize(&self.active_pty_id, rows, cols).ok();

        // Resize terminal
        self.terminal.resize(cols, rows);

        // Record if recording
        if let RecordingState::Recording { ref mut recording } = self.recording_state {
            recording.record_resize(cols, rows);
        }
    }
}
```

## UI Views

### Recording Indicator

```rust
use iced::{widget::*, Color, Length};

impl AgTerm {
    fn recording_indicator(&self) -> Element<Message> {
        match &self.recording_state {
            RecordingState::Recording { recording } => {
                let duration = recording.elapsed();

                row![
                    // Red recording dot
                    container(
                        text("●")
                            .size(20)
                            .style(Color::from_rgb(0.8, 0.0, 0.0))
                    ),

                    // Recording time
                    text(format!(
                        "REC {:02}:{:02}",
                        duration.as_secs() / 60,
                        duration.as_secs() % 60
                    ))
                    .size(14)
                ]
                .spacing(5)
                .into()
            }
            _ => row![].into(),
        }
    }
}
```

### Playback Controls

```rust
impl AgTerm {
    fn playback_controls(&self) -> Element<Message> {
        if let RecordingState::Playing { player } = &self.recording_state {
            let current = player.current_time();
            let total = player.duration();
            let progress = player.progress();
            let speed = player.speed();
            let is_playing = player.state() == PlayerState::Playing;

            column![
                // Transport controls
                row![
                    button("◄◄ -5s")
                        .on_press(Message::SkipBackward),

                    button(if is_playing { "❚❚" } else { "▶" })
                        .on_press(Message::PlayPause),

                    button("+5s ►►")
                        .on_press(Message::SkipForward),

                    button("⏹")
                        .on_press(Message::Stop),
                ]
                .spacing(10),

                // Progress bar
                slider(0.0..=1.0, progress, Message::Seek)
                    .width(Length::Fill),

                // Time display
                row![
                    text(format!(
                        "{:02}:{:02}",
                        current.as_secs() / 60,
                        current.as_secs() % 60
                    )),

                    text(" / "),

                    text(format!(
                        "{:02}:{:02}",
                        total.as_secs() / 60,
                        total.as_secs() % 60
                    )),

                    horizontal_space(Length::Fill),

                    // Speed control
                    pick_list(
                        vec![0.5, 1.0, 1.5, 2.0, 4.0],
                        Some(speed),
                        Message::SetSpeed,
                    ),
                ]
                .spacing(5),
            ]
            .spacing(10)
            .padding(10)
            .into()
        } else {
            column![].into()
        }
    }
}
```

### Main View

```rust
impl AgTerm {
    pub fn view(&self) -> Element<Message> {
        column![
            // Header with recording indicator
            row![
                text("AgTerm"),
                horizontal_space(Length::Fill),
                self.recording_indicator(),
            ]
            .padding(10),

            // Terminal display
            self.terminal_view(),

            // Playback controls (if in playback mode)
            self.playback_controls(),

            // Status bar
            self.status_bar(),
        ]
        .into()
    }
}
```

## Menu Integration

### File Menu

```rust
fn file_menu(&self) -> Element<Message> {
    column![
        // ... existing items

        horizontal_rule(1),

        match &self.recording_state {
            RecordingState::Idle => {
                column![
                    button("Start Recording")
                        .on_press(Message::StartRecording),
                    button("Load Recording...")
                        .on_press(Message::OpenLoadDialog),
                ]
            }
            RecordingState::Recording { .. } => {
                column![
                    button("Stop Recording")
                        .on_press(Message::StopRecording),
                ]
            }
            RecordingState::Playing { .. } => {
                column![
                    button("Close Playback")
                        .on_press(Message::Stop),
                ]
            }
        }
    ]
    .into()
}
```

## Keyboard Shortcuts

```rust
impl AgTerm {
    fn handle_keyboard(&mut self, event: keyboard::Event) -> Command<Message> {
        use keyboard::{KeyCode, Modifiers};

        if let keyboard::Event::KeyPressed { key_code, modifiers } = event {
            match (key_code, modifiers) {
                // Ctrl+R: Start/Stop recording
                (KeyCode::R, Modifiers::CTRL) => {
                    match &self.recording_state {
                        RecordingState::Idle => {
                            self.update(Message::StartRecording)
                        }
                        RecordingState::Recording { .. } => {
                            self.update(Message::StopRecording)
                        }
                        _ => Command::none(),
                    }
                }

                // Space: Play/Pause (only in playback mode)
                (KeyCode::Space, Modifiers::empty())
                    if matches!(self.recording_state, RecordingState::Playing { .. }) =>
                {
                    self.update(Message::PlayPause)
                }

                // Arrow keys: Skip (only in playback mode)
                (KeyCode::Left, Modifiers::empty())
                    if matches!(self.recording_state, RecordingState::Playing { .. }) =>
                {
                    self.update(Message::SkipBackward)
                }

                (KeyCode::Right, Modifiers::empty())
                    if matches!(self.recording_state, RecordingState::Playing { .. }) =>
                {
                    self.update(Message::SkipForward)
                }

                _ => Command::none(),
            }
        } else {
            Command::none()
        }
    }
}
```

## File Dialogs

```rust
use rfd::AsyncFileDialog;

impl AgTerm {
    async fn save_recording_dialog(recording: Recording) -> PathBuf {
        let file = AsyncFileDialog::new()
            .set_title("Save Recording")
            .add_filter("asciicast", &["cast"])
            .set_file_name("session.cast")
            .save_file()
            .await;

        if let Some(file) = file {
            let path = file.path().to_path_buf();
            if let Err(e) = recording.save_to_file(&path) {
                eprintln!("Failed to save recording: {}", e);
            }
            path
        } else {
            PathBuf::new()
        }
    }

    async fn load_recording_dialog() -> Option<PathBuf> {
        let file = AsyncFileDialog::new()
            .set_title("Load Recording")
            .add_filter("asciicast", &["cast"])
            .pick_file()
            .await;

        file.map(|f| f.path().to_path_buf())
    }
}
```

## Statistics Overlay

```rust
fn recording_stats_overlay(&self) -> Element<Message> {
    if let RecordingState::Recording { recording } = &self.recording_state {
        let stats = recording.stats();

        container(
            column![
                text("Recording Statistics").size(18),
                horizontal_rule(1),
                text(format!("Duration: {:?}", stats.duration)),
                text(format!("Events: {}",
                    stats.output_events + stats.input_events + stats.resize_events)),
                text(format!("Output: {} bytes", stats.output_bytes)),
                text(format!("Input: {} bytes", stats.input_bytes)),
            ]
            .spacing(5)
            .padding(10)
        )
        .style(container::Appearance {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.9).into()),
            border_radius: 5.0.into(),
            ..Default::default()
        })
        .padding(10)
        .into()
    } else {
        column![].into()
    }
}
```

## Error Handling

```rust
impl AgTerm {
    fn show_error(&mut self, message: &str) {
        self.notification = Some(Notification {
            message: message.to_string(),
            level: NotificationLevel::Error,
            duration: Duration::from_secs(5),
        });
    }

    fn show_notification(&mut self, message: &str) {
        self.notification = Some(Notification {
            message: message.to_string(),
            level: NotificationLevel::Info,
            duration: Duration::from_secs(3),
        });
    }
}
```

## Complete Integration Example

```rust
// In your main application file
use agterm::recording::{Recording, RecordingPlayer, RecordingEvent, PlayerState};

#[derive(Default)]
pub struct AgTerm {
    recording_state: RecordingState,
    // ... other fields
}

impl Application for AgTerm {
    type Message = Message;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            // Recording messages
            Message::StartRecording => { /* ... */ }
            Message::StopRecording => { /* ... */ }

            // Playback messages
            Message::PlayPause => { /* ... */ }
            Message::PlaybackUpdate => { /* ... */ }

            // Terminal events (capture for recording)
            Message::TerminalOutput(data) => {
                self.handle_pty_output(data);
                Command::none()
            }

            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Self::Message> {
        // Build UI with recording controls
        column![
            self.recording_indicator(),
            self.terminal_view(),
            self.playback_controls(),
        ]
        .into()
    }
}
```

## Testing UI Integration

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_state_transitions() {
        let mut app = AgTerm::default();

        // Start recording
        app.update(Message::StartRecording);
        assert!(matches!(app.recording_state, RecordingState::Recording { .. }));

        // Stop recording
        app.update(Message::StopRecording);
        assert!(matches!(app.recording_state, RecordingState::Idle));
    }

    #[test]
    fn test_playback_state_transitions() {
        let mut app = AgTerm::default();

        // Load recording
        let recording = Recording::new(80, 24);
        let player = RecordingPlayer::new(recording);
        app.recording_state = RecordingState::Playing { player };

        // Play/pause
        app.update(Message::PlayPause);
        if let RecordingState::Playing { player } = &app.recording_state {
            assert_eq!(player.state(), PlayerState::Playing);
        }
    }
}
```

This integration guide provides a complete reference for adding the recording feature to AgTerm's UI with proper state management, controls, and user feedback.
