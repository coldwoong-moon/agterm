//! Character encoding support for terminal emulator
//!
//! Provides encoding/decoding capabilities for various character sets commonly
//! used in terminal applications and legacy systems.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use thiserror::Error;

/// Supported character encodings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Encoding {
    /// UTF-8 (default)
    Utf8,
    /// UTF-16 (platform endianness)
    Utf16,
    /// UTF-16 Big Endian
    Utf16Be,
    /// UTF-16 Little Endian
    Utf16Le,
    /// ISO-8859-1 (Latin-1)
    Latin1,
    /// EUC-KR (Korean)
    EucKr,
    /// EUC-JP (Japanese)
    EucJp,
    /// Shift-JIS (Japanese)
    ShiftJis,
    /// GB2312 (Simplified Chinese)
    Gb2312,
    /// Big5 (Traditional Chinese)
    Big5,
}

impl Default for Encoding {
    fn default() -> Self {
        Encoding::Utf8
    }
}

impl Encoding {
    /// Returns the canonical name of the encoding
    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Utf8 => "UTF-8",
            Encoding::Utf16 => "UTF-16",
            Encoding::Utf16Be => "UTF-16BE",
            Encoding::Utf16Le => "UTF-16LE",
            Encoding::Latin1 => "ISO-8859-1",
            Encoding::EucKr => "EUC-KR",
            Encoding::EucJp => "EUC-JP",
            Encoding::ShiftJis => "Shift-JIS",
            Encoding::Gb2312 => "GB2312",
            Encoding::Big5 => "Big5",
        }
    }

    /// Parse encoding from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.to_lowercase();
        match s.as_str() {
            "utf-8" | "utf8" => Some(Encoding::Utf8),
            "utf-16" | "utf16" => Some(Encoding::Utf16),
            "utf-16be" | "utf16be" => Some(Encoding::Utf16Be),
            "utf-16le" | "utf16le" => Some(Encoding::Utf16Le),
            "iso-8859-1" | "iso8859-1" | "latin-1" | "latin1" => Some(Encoding::Latin1),
            "euc-kr" | "euckr" => Some(Encoding::EucKr),
            "euc-jp" | "eucjp" => Some(Encoding::EucJp),
            "shift-jis" | "shift_jis" | "shiftjis" | "sjis" => Some(Encoding::ShiftJis),
            "gb2312" | "gb-2312" => Some(Encoding::Gb2312),
            "big5" | "big-5" => Some(Encoding::Big5),
            _ => None,
        }
    }

    /// Returns list of all available encodings
    pub fn available() -> &'static [Self] {
        &[
            Encoding::Utf8,
            Encoding::Utf16,
            Encoding::Utf16Be,
            Encoding::Utf16Le,
            Encoding::Latin1,
            Encoding::EucKr,
            Encoding::EucJp,
            Encoding::ShiftJis,
            Encoding::Gb2312,
            Encoding::Big5,
        ]
    }

    /// Decode bytes to string using this encoding
    pub fn decode<'a>(&self, bytes: &'a [u8]) -> Result<Cow<'a, str>, EncodingError> {
        match self {
            Encoding::Utf8 => {
                // Use lossy conversion for UTF-8 to handle invalid sequences gracefully
                Ok(String::from_utf8_lossy(bytes))
            }
            Encoding::Utf16 => {
                self.decode_utf16(bytes, native_endian())
            }
            Encoding::Utf16Be => {
                self.decode_utf16(bytes, Endian::Big)
            }
            Encoding::Utf16Le => {
                self.decode_utf16(bytes, Endian::Little)
            }
            Encoding::Latin1 => {
                // ISO-8859-1 is a single-byte encoding where each byte maps directly to Unicode
                Ok(Cow::Owned(bytes.iter().map(|&b| b as char).collect()))
            }
            Encoding::EucKr | Encoding::EucJp | Encoding::ShiftJis | Encoding::Gb2312 | Encoding::Big5 => {
                // For CJK encodings, we need external crate support
                // For now, fallback to UTF-8 lossy and return error indicating need for implementation
                Err(EncodingError::UnsupportedEncoding(self.as_str()))
            }
        }
    }

    /// Encode string to bytes using this encoding
    pub fn encode(&self, s: &str) -> Result<Vec<u8>, EncodingError> {
        match self {
            Encoding::Utf8 => Ok(s.as_bytes().to_vec()),
            Encoding::Utf16 => {
                self.encode_utf16(s, native_endian())
            }
            Encoding::Utf16Be => {
                self.encode_utf16(s, Endian::Big)
            }
            Encoding::Utf16Le => {
                self.encode_utf16(s, Endian::Little)
            }
            Encoding::Latin1 => {
                // Encode to Latin-1, replacing characters outside the range with '?'
                Ok(s.chars()
                    .map(|c| {
                        let code = c as u32;
                        if code <= 0xFF {
                            code as u8
                        } else {
                            b'?'
                        }
                    })
                    .collect())
            }
            Encoding::EucKr | Encoding::EucJp | Encoding::ShiftJis | Encoding::Gb2312 | Encoding::Big5 => {
                // For CJK encodings, we need external crate support
                Err(EncodingError::UnsupportedEncoding(self.as_str()))
            }
        }
    }

    /// Helper to decode UTF-16 with specified endianness
    fn decode_utf16(&self, bytes: &[u8], endian: Endian) -> Result<Cow<'static, str>, EncodingError> {
        if bytes.len() % 2 != 0 {
            return Err(EncodingError::InvalidSequence("UTF-16 requires even number of bytes"));
        }

        let u16_vec: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| match endian {
                Endian::Big => u16::from_be_bytes([chunk[0], chunk[1]]),
                Endian::Little => u16::from_le_bytes([chunk[0], chunk[1]]),
            })
            .collect();

        String::from_utf16(&u16_vec)
            .map(|s| Cow::Owned(s))
            .map_err(|_| EncodingError::InvalidSequence("Invalid UTF-16 sequence"))
    }

    /// Helper to encode UTF-16 with specified endianness
    fn encode_utf16(&self, s: &str, endian: Endian) -> Result<Vec<u8>, EncodingError> {
        let u16_vec: Vec<u16> = s.encode_utf16().collect();
        let mut bytes = Vec::with_capacity(u16_vec.len() * 2);

        for &code_unit in &u16_vec {
            let [b1, b2] = match endian {
                Endian::Big => code_unit.to_be_bytes(),
                Endian::Little => code_unit.to_le_bytes(),
            };
            bytes.push(b1);
            bytes.push(b2);
        }

        Ok(bytes)
    }

    /// Attempts to auto-detect encoding from byte sequence
    /// Returns confidence score (0.0 - 1.0) and detected encoding
    pub fn auto_detect(bytes: &[u8]) -> Option<(Self, f32)> {
        if bytes.is_empty() {
            return None;
        }

        // Check for UTF-8 BOM
        if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            return Some((Encoding::Utf8, 1.0));
        }

        // Check for UTF-16 BOM
        if bytes.len() >= 2 {
            if bytes.starts_with(&[0xFE, 0xFF]) {
                return Some((Encoding::Utf16Be, 1.0));
            }
            if bytes.starts_with(&[0xFF, 0xFE]) {
                return Some((Encoding::Utf16Le, 1.0));
            }
        }

        // Try UTF-8 validation
        if std::str::from_utf8(bytes).is_ok() {
            return Some((Encoding::Utf8, 0.9));
        }

        // Check for null bytes which might indicate UTF-16
        let null_count = bytes.iter().filter(|&&b| b == 0).count();
        if null_count > bytes.len() / 10 {
            // If many nulls, likely UTF-16
            // Check pattern to determine endianness
            let even_nulls = bytes.iter().step_by(2).filter(|&&b| b == 0).count();
            let odd_nulls = bytes.iter().skip(1).step_by(2).filter(|&&b| b == 0).count();

            if even_nulls > odd_nulls {
                return Some((Encoding::Utf16Be, 0.7));
            } else if odd_nulls > even_nulls {
                return Some((Encoding::Utf16Le, 0.7));
            }
        }

        // Default fallback
        Some((Encoding::Utf8, 0.3))
    }
}

/// Byte order for multi-byte encodings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endian {
    Big,
    Little,
}

/// Returns the native endianness of the platform
fn native_endian() -> Endian {
    if cfg!(target_endian = "big") {
        Endian::Big
    } else {
        Endian::Little
    }
}

/// Encoding-related errors
#[derive(Error, Debug)]
pub enum EncodingError {
    #[error("Invalid byte sequence: {0}")]
    InvalidSequence(&'static str),

    #[error("Unsupported encoding: {0} (requires additional dependencies)")]
    UnsupportedEncoding(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_as_str() {
        assert_eq!(Encoding::Utf8.as_str(), "UTF-8");
        assert_eq!(Encoding::EucKr.as_str(), "EUC-KR");
        assert_eq!(Encoding::ShiftJis.as_str(), "Shift-JIS");
    }

    #[test]
    fn test_encoding_from_str() {
        assert_eq!(Encoding::from_str("utf-8"), Some(Encoding::Utf8));
        assert_eq!(Encoding::from_str("UTF8"), Some(Encoding::Utf8));
        assert_eq!(Encoding::from_str("euc-kr"), Some(Encoding::EucKr));
        assert_eq!(Encoding::from_str("shift_jis"), Some(Encoding::ShiftJis));
        assert_eq!(Encoding::from_str("unknown"), None);
    }

    #[test]
    fn test_utf8_decode() {
        let text = "Hello, 世界!";
        let bytes = text.as_bytes();
        let decoded = Encoding::Utf8.decode(bytes).unwrap();
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_utf8_encode() {
        let text = "Hello, 世界!";
        let encoded = Encoding::Utf8.encode(text).unwrap();
        assert_eq!(encoded, text.as_bytes());
    }

    #[test]
    fn test_latin1_decode() {
        let bytes = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xE9]; // "Helloé"
        let decoded = Encoding::Latin1.decode(&bytes).unwrap();
        assert_eq!(decoded, "Helloé");
    }

    #[test]
    fn test_latin1_encode() {
        let text = "Helloé";
        let encoded = Encoding::Latin1.encode(text).unwrap();
        assert_eq!(encoded, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xE9]);

        // Test character outside Latin-1 range
        let text = "Hello世界";
        let encoded = Encoding::Latin1.encode(text).unwrap();
        assert_eq!(encoded, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, b'?', b'?']);
    }

    #[test]
    fn test_utf16le_decode() {
        // "Hi" in UTF-16LE
        let bytes = vec![0x48, 0x00, 0x69, 0x00];
        let decoded = Encoding::Utf16Le.decode(&bytes).unwrap();
        assert_eq!(decoded, "Hi");
    }

    #[test]
    fn test_utf16le_encode() {
        let text = "Hi";
        let encoded = Encoding::Utf16Le.encode(text).unwrap();
        assert_eq!(encoded, vec![0x48, 0x00, 0x69, 0x00]);
    }

    #[test]
    fn test_utf16be_decode() {
        // "Hi" in UTF-16BE
        let bytes = vec![0x00, 0x48, 0x00, 0x69];
        let decoded = Encoding::Utf16Be.decode(&bytes).unwrap();
        assert_eq!(decoded, "Hi");
    }

    #[test]
    fn test_utf16be_encode() {
        let text = "Hi";
        let encoded = Encoding::Utf16Be.encode(text).unwrap();
        assert_eq!(encoded, vec![0x00, 0x48, 0x00, 0x69]);
    }

    #[test]
    fn test_utf16_invalid_length() {
        let bytes = vec![0x48, 0x00, 0x69]; // Odd number of bytes
        let result = Encoding::Utf16Le.decode(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_detect_utf8_bom() {
        let bytes = vec![0xEF, 0xBB, 0xBF, 0x48, 0x65, 0x6C, 0x6C, 0x6F];
        let (encoding, confidence) = Encoding::auto_detect(&bytes).unwrap();
        assert_eq!(encoding, Encoding::Utf8);
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_auto_detect_utf16be_bom() {
        let bytes = vec![0xFE, 0xFF, 0x00, 0x48];
        let (encoding, confidence) = Encoding::auto_detect(&bytes).unwrap();
        assert_eq!(encoding, Encoding::Utf16Be);
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_auto_detect_utf16le_bom() {
        let bytes = vec![0xFF, 0xFE, 0x48, 0x00];
        let (encoding, confidence) = Encoding::auto_detect(&bytes).unwrap();
        assert_eq!(encoding, Encoding::Utf16Le);
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_auto_detect_valid_utf8() {
        let bytes = "Hello, world!".as_bytes();
        let (encoding, confidence) = Encoding::auto_detect(bytes).unwrap();
        assert_eq!(encoding, Encoding::Utf8);
        assert_eq!(confidence, 0.9);
    }

    #[test]
    fn test_available_encodings() {
        let encodings = Encoding::available();
        assert!(encodings.contains(&Encoding::Utf8));
        assert!(encodings.contains(&Encoding::EucKr));
        assert!(encodings.len() == 10);
    }

    #[test]
    fn test_unsupported_encodings() {
        let text = "Hello";

        // EUC-KR encoding should return unsupported error
        let result = Encoding::EucKr.encode(text);
        assert!(matches!(result, Err(EncodingError::UnsupportedEncoding(_))));

        // ShiftJIS decoding should return unsupported error
        let bytes = vec![0x82, 0xA0];
        let result = Encoding::ShiftJis.decode(&bytes);
        assert!(matches!(result, Err(EncodingError::UnsupportedEncoding(_))));
    }
}
