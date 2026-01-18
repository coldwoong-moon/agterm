//! Terminal bell sound playback module
//!
//! Provides cross-platform system beep sound using rodio audio library.

use rodio::Source;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

/// Simple beep sound generator
pub struct BellSound {
    /// Audio output stream (kept alive for the duration of the app)
    _stream: Option<rodio::OutputStream>,
    /// Audio stream handle for playing sounds
    stream_handle: Option<Arc<Mutex<rodio::OutputStreamHandle>>>,
}

impl BellSound {
    /// Create a new bell sound player
    pub fn new() -> Self {
        // Try to initialize audio output
        match rodio::OutputStream::try_default() {
            Ok((_stream, stream_handle)) => {
                log::info!("Bell sound initialized successfully");
                Self {
                    _stream: Some(_stream),
                    stream_handle: Some(Arc::new(Mutex::new(stream_handle))),
                }
            }
            Err(e) => {
                log::warn!("Failed to initialize bell sound: {e}");
                Self {
                    _stream: None,
                    stream_handle: None,
                }
            }
        }
    }

    /// Play a bell sound with the specified volume (0.0 to 1.0)
    pub fn play(&self, volume: f32) {
        if let Some(handle) = &self.stream_handle {
            // Generate a simple beep sound (sine wave at 800Hz for 100ms)
            let beep_data = generate_beep(800.0, 0.1, volume);

            // Play the sound in a separate thread to avoid blocking
            let handle_clone = Arc::clone(handle);
            std::thread::spawn(move || {
                if let Ok(handle) = handle_clone.lock() {
                    if let Ok(source) = rodio::Decoder::new(Cursor::new(beep_data)) {
                        if let Err(e) = handle.play_raw(source.convert_samples()) {
                            log::warn!("Failed to play bell sound: {e}");
                        }
                    }
                }
            });
        }
    }

    /// Check if sound is available
    pub fn is_available(&self) -> bool {
        self.stream_handle.is_some()
    }
}

impl Default for BellSound {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a simple beep sound as WAV data
fn generate_beep(frequency: f32, duration: f32, volume: f32) -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44100;
    let num_samples = (SAMPLE_RATE as f32 * duration) as usize;

    // Generate sine wave samples
    let mut samples: Vec<i16> = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin();

        // Apply volume and convert to i16
        let scaled_sample = (sample * volume * 32767.0) as i16;
        samples.push(scaled_sample);
    }

    // Create WAV file in memory
    create_wav(&samples, SAMPLE_RATE)
}

/// Create a WAV file header and data
fn create_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let num_samples = samples.len();
    let num_channels = 1u16;
    let bits_per_sample = 16u16;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = (num_samples * 2) as u32;
    let file_size = 36 + data_size;

    let mut wav_data = Vec::with_capacity((file_size + 8) as usize);

    // RIFF header
    wav_data.extend_from_slice(b"RIFF");
    wav_data.extend_from_slice(&file_size.to_le_bytes());
    wav_data.extend_from_slice(b"WAVE");

    // fmt chunk
    wav_data.extend_from_slice(b"fmt ");
    wav_data.extend_from_slice(&16u32.to_le_bytes()); // Chunk size
    wav_data.extend_from_slice(&1u16.to_le_bytes()); // Audio format (PCM)
    wav_data.extend_from_slice(&num_channels.to_le_bytes());
    wav_data.extend_from_slice(&sample_rate.to_le_bytes());
    wav_data.extend_from_slice(&byte_rate.to_le_bytes());
    wav_data.extend_from_slice(&block_align.to_le_bytes());
    wav_data.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav_data.extend_from_slice(b"data");
    wav_data.extend_from_slice(&data_size.to_le_bytes());

    // Sample data
    for sample in samples {
        wav_data.extend_from_slice(&sample.to_le_bytes());
    }

    wav_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beep_generation() {
        let beep_data = generate_beep(800.0, 0.1, 0.5);
        assert!(!beep_data.is_empty());
        // WAV header should start with "RIFF"
        assert_eq!(&beep_data[0..4], b"RIFF");
    }

    #[test]
    fn test_bell_sound_creation() {
        let bell = BellSound::new();
        // Should not panic
        assert!(true);
    }
}
