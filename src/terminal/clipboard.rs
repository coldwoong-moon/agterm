//! OSC 52 Clipboard Protocol Implementation
//!
//! Implements the OSC 52 escape sequence for clipboard integration.
//! Specification: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands
//!
//! Format: ESC ] 52 ; Pc ; Pd BEL (or ST)
//! - Pc: clipboard selection ('c' = clipboard, 'p' = primary, 's' = secondary)
//! - Pd: base64-encoded data, or '?' for read request

use arboard::Clipboard;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

/// Manages clipboard operations and OSC 52 protocol handling
pub struct ClipboardManager {
    clipboard: Option<Clipboard>,
}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new() -> Self {
        Self {
            clipboard: Clipboard::new().ok(),
        }
    }

    /// Handle OSC 52 escape sequence
    ///
    /// # Arguments
    /// * `selection` - Clipboard selection type ('c' = clipboard, 'p' = primary, 's' = secondary)
    /// * `data` - base64-encoded text to set, or '?' to read
    ///
    /// # Returns
    /// * `Some(String)` - Response sequence if read was requested
    /// * `None` - No response needed (write operation)
    pub fn handle_osc52(&mut self, selection: char, data: &str) -> Option<String> {
        if data == "?" {
            // Read request - return current clipboard content as OSC 52 sequence
            self.get_clipboard().map(|text| {
                let encoded = BASE64.encode(text.as_bytes());
                format!("\x1b]52;{selection};{encoded}\x07")
            })
        } else {
            // Write request - decode and set clipboard
            if let Ok(decoded_bytes) = BASE64.decode(data) {
                if let Ok(text) = String::from_utf8(decoded_bytes) {
                    self.set_clipboard(&text);
                }
            }
            None
        }
    }

    /// Get text from clipboard
    pub fn get_clipboard(&mut self) -> Option<String> {
        self.clipboard.as_mut()?.get_text().ok()
    }

    /// Set text to clipboard
    pub fn set_clipboard(&mut self, text: &str) -> bool {
        if let Some(cb) = self.clipboard.as_mut() {
            cb.set_text(text.to_string()).is_ok()
        } else {
            false
        }
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encoding() {
        let text = "Hello, World!";
        let encoded = BASE64.encode(text.as_bytes());
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn test_base64_decoding() {
        let encoded = "SGVsbG8sIFdvcmxkIQ==";
        let decoded = BASE64.decode(encoded).unwrap();
        let text = String::from_utf8(decoded).unwrap();
        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn test_osc52_write() {
        let mut manager = ClipboardManager::new();
        let encoded = "SGVsbG8sIFdvcmxkIQ=="; // "Hello, World!"
        let response = manager.handle_osc52('c', encoded);
        // Write should not return a response
        assert_eq!(response, None);
    }

    #[test]
    fn test_osc52_read_request() {
        let mut manager = ClipboardManager::new();
        // Set some text first
        manager.set_clipboard("Test");
        // Read request should return OSC 52 sequence
        let response = manager.handle_osc52('c', "?");
        if let Some(resp) = response {
            assert!(resp.starts_with("\x1b]52;c;"));
            assert!(resp.ends_with("\x07"));
        }
    }

    #[test]
    fn test_korean_text() {
        let text = "안녕하세요";
        let encoded = BASE64.encode(text.as_bytes());
        let decoded = BASE64.decode(&encoded).unwrap();
        let result = String::from_utf8(decoded).unwrap();
        assert_eq!(result, text);
    }

    #[test]
    fn test_multiline_text() {
        let text = "Line 1\nLine 2\nLine 3";
        let encoded = BASE64.encode(text.as_bytes());
        let decoded = BASE64.decode(&encoded).unwrap();
        let result = String::from_utf8(decoded).unwrap();
        assert_eq!(result, text);
    }

    #[test]
    fn test_empty_text() {
        let text = "";
        let encoded = BASE64.encode(text.as_bytes());
        let decoded = BASE64.decode(&encoded).unwrap();
        let result = String::from_utf8(decoded).unwrap();
        assert_eq!(result, text);
    }

    #[test]
    fn test_special_characters() {
        let text = "Special: !@#$%^&*()_+-=[]{}|;':\",./<>?";
        let encoded = BASE64.encode(text.as_bytes());
        let decoded = BASE64.decode(&encoded).unwrap();
        let result = String::from_utf8(decoded).unwrap();
        assert_eq!(result, text);
    }
}
