//! Example demonstrating terminal session recording and playback

use agterm::recording::{Recording, RecordingPlayer, PlayerState};
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Terminal Recording Demo ===\n");

    // Create a new recording
    println!("1. Creating recording (80x24)...");
    let mut recording = Recording::new(80, 24);

    // Start recording
    recording.start();
    println!("   Recording started");

    // Simulate terminal output
    recording.add_output(Duration::from_secs(0), b"$ echo 'Hello, AgTerm!'");
    recording.add_output(Duration::from_millis(100), b"\r\nHello, AgTerm!");

    // Simulate user input
    recording.add_input(Duration::from_millis(500), b"ls -la\n");

    // Simulate more output
    recording.add_output(Duration::from_millis(600), b"\r\ntotal 48");
    recording.add_output(Duration::from_millis(700), b"\r\ndrwxr-xr-x  12 user  staff   384 Jan 18 10:30 .");
    recording.add_output(Duration::from_millis(800), b"\r\ndrwxr-xr-x   5 user  staff   160 Jan 18 09:15 ..");

    // Simulate terminal resize
    recording.add_resize(Duration::from_secs(1), 120, 40);
    recording.add_output(Duration::from_millis(1100), b"\r\n$ ");

    // Stop recording
    recording.stop();
    println!("   Recording stopped");

    // Show recording stats
    let stats = recording.stats();
    println!("\n2. Recording Statistics:");
    println!("   Duration: {:?}", stats.duration);
    println!("   Output events: {}", stats.output_events);
    println!("   Output bytes: {}", stats.output_bytes);
    println!("   Input events: {}", stats.input_events);
    println!("   Input bytes: {}", stats.input_bytes);
    println!("   Resize events: {}", stats.resize_events);

    // Save recording to file
    println!("\n3. Saving recording...");
    let temp_file = "/tmp/agterm_demo.cast";
    match recording.save_to_file(temp_file) {
        Ok(_) => println!("   Saved to: {}", temp_file),
        Err(e) => println!("   Error saving: {}", e),
    }

    // Load recording from file
    println!("\n4. Loading recording...");
    let loaded_recording = match Recording::load_from_file(temp_file) {
        Ok(r) => {
            println!("   Loaded {} events", r.len());
            r
        }
        Err(e) => {
            println!("   Error loading: {}", e);
            return;
        }
    };

    // Create player
    println!("\n5. Creating player...");
    let mut player = RecordingPlayer::new(loaded_recording);
    println!("   Player created");
    println!("   Duration: {:?}", player.duration());

    // Test playback speed
    println!("\n6. Testing playback controls:");
    println!("   Setting speed to 2.0x");
    player.set_speed(2.0);
    println!("   Current speed: {}x", player.speed());

    // Test seek
    println!("\n7. Testing seek:");
    player.seek(Duration::from_millis(500));
    println!("   Seeked to: {:?}", player.current_time());
    println!("   Progress: {:.1}%", player.progress() * 100.0);

    // Test play/pause
    println!("\n8. Testing play/pause:");
    player.play();
    println!("   State: {:?}", player.state());
    assert_eq!(player.state(), PlayerState::Playing);

    thread::sleep(Duration::from_millis(100));

    player.pause();
    println!("   State: {:?}", player.state());
    assert_eq!(player.state(), PlayerState::Paused);

    player.stop();
    println!("   State: {:?}", player.state());
    assert_eq!(player.state(), PlayerState::Stopped);

    // Test skip
    println!("\n9. Testing skip:");
    player.skip_forward(Duration::from_millis(200));
    println!("   Skipped forward to: {:?}", player.current_time());

    player.skip_backward(Duration::from_millis(100));
    println!("   Skipped backward to: {:?}", player.current_time());

    // Compression test
    println!("\n10. Testing compression:");
    let mut recording_with_dupes = Recording::new(80, 24);
    recording_with_dupes.start();
    recording_with_dupes.add_resize(Duration::from_secs(0), 80, 24);
    recording_with_dupes.add_resize(Duration::from_millis(100), 80, 24); // Duplicate
    recording_with_dupes.add_resize(Duration::from_millis(200), 120, 40);
    recording_with_dupes.add_resize(Duration::from_millis(300), 120, 40); // Duplicate
    recording_with_dupes.stop();

    let before_count = recording_with_dupes.len();
    recording_with_dupes.compress();
    let after_count = recording_with_dupes.len();

    println!("   Events before compression: {}", before_count);
    println!("   Events after compression: {}", after_count);
    println!("   Saved {} events", before_count - after_count);

    println!("\n=== Demo Complete ===");
    println!("Recording module is working correctly!");
}
