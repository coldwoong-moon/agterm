//! Terminal Image Protocol Support
//!
//! This module implements support for inline images in terminal emulators through various protocols:
//! - iTerm2 Inline Images Protocol (base64 encoded)
//! - Kitty Graphics Protocol (streaming and direct)
//! - SIXEL Graphics (legacy support)
//!
//! # Protocol Overview
//!
//! ## iTerm2 Protocol
//! Format: `ESC ] 1337 ; File=[args] : [base64-data] ^G`
//! - Widely supported in modern terminals
//! - Simple base64 encoding
//! - Supports PNG, JPEG, GIF
//!
//! ## Kitty Protocol
//! Format: `ESC _G[key=value,...];[data]ESC\`
//! - More efficient chunked transfer
//! - Better control over positioning
//! - Supports transparency
//!
//! ## SIXEL
//! Format: `ESC P [params] q [sixel-data] ESC \`
//! - Legacy protocol from DEC terminals
//! - Limited color palette
//! - Good for simple graphics

use std::collections::HashMap;
use thiserror::Error;

/// Image format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    /// PNG format (Portable Network Graphics)
    PNG,
    /// JPEG format
    JPEG,
    /// GIF format (animated GIFs not yet supported)
    GIF,
    /// SIXEL format (legacy terminal graphics)
    SIXEL,
}

impl ImageFormat {
    /// Detect format from magic bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        // PNG: 89 50 4E 47
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Some(ImageFormat::PNG);
        }

        // JPEG: FF D8 FF
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(ImageFormat::JPEG);
        }

        // GIF: "GIF87a" or "GIF89a"
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return Some(ImageFormat::GIF);
        }

        None
    }

    /// Get MIME type for format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::PNG => "image/png",
            ImageFormat::JPEG => "image/jpeg",
            ImageFormat::GIF => "image/gif",
            ImageFormat::SIXEL => "image/x-sixel",
        }
    }

    /// Check if format is supported for decoding
    pub fn is_supported(&self) -> bool {
        matches!(
            self,
            ImageFormat::PNG | ImageFormat::JPEG | ImageFormat::GIF
        )
    }
}

/// Image data container
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Image format
    pub format: ImageFormat,
    /// Raw image bytes
    pub data: Vec<u8>,
    /// Image dimensions in pixels (width, height)
    pub dimensions: Option<(u32, u32)>,
    /// Display size in terminal cells (cols, rows)
    pub cell_size: Option<(u16, u16)>,
    /// Placement ID for tracking (Kitty protocol)
    pub placement_id: Option<u32>,
    /// Whether to preserve aspect ratio when scaling
    pub preserve_aspect: bool,
    /// Position in terminal (col, row)
    pub position: Option<(u16, u16)>,
}

impl ImageData {
    /// Create new image data from raw bytes
    pub fn new(data: Vec<u8>) -> Result<Self, ImageError> {
        let format = ImageFormat::from_bytes(&data)
            .ok_or_else(|| ImageError::UnsupportedFormat("Unknown image format".to_string()))?;

        Ok(Self {
            format,
            data,
            dimensions: None,
            cell_size: None,
            placement_id: None,
            preserve_aspect: true,
            position: None,
        })
    }

    /// Create image data with known format
    pub fn with_format(format: ImageFormat, data: Vec<u8>) -> Self {
        Self {
            format,
            data,
            dimensions: None,
            cell_size: None,
            placement_id: None,
            preserve_aspect: true,
            position: None,
        }
    }

    /// Set image dimensions
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.dimensions = Some((width, height));
        self
    }

    /// Set cell size for display
    pub fn with_cell_size(mut self, cols: u16, rows: u16) -> Self {
        self.cell_size = Some((cols, rows));
        self
    }

    /// Set placement ID
    pub fn with_placement_id(mut self, id: u32) -> Self {
        self.placement_id = Some(id);
        self
    }

    /// Set position in terminal
    pub fn with_position(mut self, col: u16, row: u16) -> Self {
        self.position = Some((col, row));
        self
    }

    /// Get data size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if format is supported
    pub fn is_supported(&self) -> bool {
        self.format.is_supported()
    }
}

/// Image protocol errors
#[derive(Error, Debug)]
pub enum ImageError {
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid base64 encoding: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("Invalid protocol data: {0}")]
    InvalidProtocol(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Image decode error: {0}")]
    DecodeError(String),

    #[error("Invalid dimensions: {0}")]
    InvalidDimensions(String),
}

/// iTerm2 image protocol parameters
#[derive(Debug, Clone, Default)]
pub struct ITermParams {
    /// Filename (optional)
    pub name: Option<String>,
    /// Width in cells or pixels
    pub width: Option<String>,
    /// Height in cells or pixels
    pub height: Option<String>,
    /// Preserve aspect ratio (default: true)
    pub preserve_aspect_ratio: bool,
    /// Inline display (vs. download)
    pub inline: bool,
}

impl ITermParams {
    /// Parse iTerm2 parameters from key=value pairs
    pub fn parse(params: &str) -> Self {
        let mut result = Self {
            inline: true,
            preserve_aspect_ratio: true,
            ..Default::default()
        };

        for param in params.split(';') {
            if let Some((key, value)) = param.split_once('=') {
                match key {
                    "name" => result.name = Some(value.to_string()),
                    "width" => result.width = Some(value.to_string()),
                    "height" => result.height = Some(value.to_string()),
                    "preserveAspectRatio" => {
                        result.preserve_aspect_ratio = value == "1" || value == "true"
                    }
                    "inline" => result.inline = value == "1" || value == "true",
                    _ => {}
                }
            }
        }

        result
    }

    /// Parse dimension string (e.g., "80px", "40%", "auto")
    pub fn parse_dimension(dim: &str) -> Option<u16> {
        if dim == "auto" {
            return None;
        }

        if let Some(px) = dim.strip_suffix("px") {
            return px.parse().ok();
        }

        dim.parse().ok()
    }
}

/// Kitty graphics protocol parameters
#[derive(Debug, Clone, Default)]
pub struct KittyParams {
    /// Action (transmit, display, query, delete)
    pub action: Option<char>,
    /// Format (24=RGB, 32=RGBA, 100=PNG)
    pub format: Option<u32>,
    /// Transmission medium (direct, file, temp file, shared memory)
    pub transmission: Option<char>,
    /// Width in pixels
    pub width: Option<u32>,
    /// Height in pixels
    pub height: Option<u32>,
    /// Display width in cells
    pub columns: Option<u16>,
    /// Display height in cells
    pub rows: Option<u16>,
    /// Image ID
    pub image_id: Option<u32>,
    /// Placement ID
    pub placement_id: Option<u32>,
    /// Compression (zlib)
    pub compressed: bool,
    /// Whether this is more data coming
    pub more: bool,
}

impl KittyParams {
    /// Parse Kitty protocol parameters from key=value pairs
    pub fn parse(params: &str) -> Self {
        let mut result = Self::default();

        for param in params.split(',') {
            if let Some((key, value)) = param.split_once('=') {
                match key {
                    "a" => result.action = value.chars().next(),
                    "f" => result.format = value.parse().ok(),
                    "t" => result.transmission = value.chars().next(),
                    "s" => result.width = value.parse().ok(),
                    "v" => result.height = value.parse().ok(),
                    "c" => result.columns = value.parse().ok(),
                    "r" => result.rows = value.parse().ok(),
                    "i" => result.image_id = value.parse().ok(),
                    "p" => result.placement_id = value.parse().ok(),
                    "o" => result.compressed = value == "z",
                    "m" => result.more = value == "1",
                    _ => {}
                }
            }
        }

        result
    }
}

/// Image protocol handler
#[derive(Debug, Default)]
pub struct ImageProtocol {
    /// Cache of decoded images by placement ID
    images: HashMap<u32, ImageData>,
    /// Next available placement ID
    next_placement_id: u32,
    /// Accumulated chunks for multi-part transmissions (Kitty)
    chunks: HashMap<u32, Vec<u8>>,
}

impl ImageProtocol {
    /// Create a new image protocol handler
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse iTerm2 image protocol sequence
    ///
    /// Format: ESC ] 1337 ; File=[args] : [base64-data] ^G
    pub fn parse_iterm2(&mut self, sequence: &str) -> Result<ImageData, ImageError> {
        // Expected format: "File=[params]:[base64]"
        let without_prefix = sequence
            .strip_prefix("File=")
            .ok_or_else(|| ImageError::InvalidProtocol("Missing 'File=' prefix".to_string()))?;

        let (params_str, data_str) = without_prefix
            .split_once(':')
            .ok_or_else(|| ImageError::InvalidProtocol("Missing ':' separator".to_string()))?;

        // Parse parameters
        let params = ITermParams::parse(params_str);

        // Decode base64 data
        let data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data_str)?;

        // Create image data
        let mut image = ImageData::new(data)?;
        image.preserve_aspect = params.preserve_aspect_ratio;

        // Parse dimensions if provided
        let mut cols: u16 = 0;
        let mut rows: u16 = 0;

        if let Some(width_str) = &params.width {
            if let Some(width) = ITermParams::parse_dimension(width_str) {
                cols = width;
            }
        }

        if let Some(height_str) = &params.height {
            if let Some(height) = ITermParams::parse_dimension(height_str) {
                rows = height;
            }
        }

        if cols > 0 || rows > 0 {
            image = image.with_cell_size(cols, rows);
        }

        // Assign placement ID
        let placement_id = self.next_placement_id;
        self.next_placement_id += 1;
        image = image.with_placement_id(placement_id);

        // Cache the image
        self.images.insert(placement_id, image.clone());

        Ok(image)
    }

    /// Parse Kitty graphics protocol sequence
    ///
    /// Format: ESC _G[key=value,...];[data]ESC\
    pub fn parse_kitty(&mut self, params_str: &str, data: &str) -> Result<ImageData, ImageError> {
        let params = KittyParams::parse(params_str);

        // Decode base64 data
        let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data)?;

        // Get or create image ID
        let image_id = params.image_id.unwrap_or_else(|| {
            let id = self.next_placement_id;
            self.next_placement_id += 1;
            id
        });

        // Handle chunked transmission
        if params.more {
            // Accumulate chunks
            self.chunks
                .entry(image_id)
                .or_default()
                .extend_from_slice(&decoded);
            // Return placeholder - final image will be created when more=false
            return Err(ImageError::InvalidProtocol(
                "Incomplete chunked transmission".to_string(),
            ));
        }

        // Get complete data (from chunks or direct)
        let complete_data = if let Some(mut chunks) = self.chunks.remove(&image_id) {
            chunks.extend_from_slice(&decoded);
            chunks
        } else {
            decoded
        };

        // Create image data
        let mut image = ImageData::new(complete_data)?;

        // Set dimensions
        if let (Some(width), Some(height)) = (params.width, params.height) {
            image = image.with_dimensions(width, height);
        }

        // Set cell size
        if let (Some(cols), Some(rows)) = (params.columns, params.rows) {
            image = image.with_cell_size(cols, rows);
        }

        // Set placement ID
        if let Some(placement_id) = params.placement_id {
            image = image.with_placement_id(placement_id);
        } else {
            image = image.with_placement_id(image_id);
        }

        // Cache the image
        self.images.insert(image_id, image.clone());

        Ok(image)
    }

    /// Parse SIXEL graphics sequence (basic support)
    ///
    /// Format: ESC P [params] q [sixel-data] ESC \
    pub fn parse_sixel(&mut self, _data: &str) -> Result<ImageData, ImageError> {
        // SIXEL is complex and requires full parser implementation
        // For now, return unsupported
        Err(ImageError::UnsupportedFormat(
            "SIXEL format not yet implemented".to_string(),
        ))
    }

    /// Get cached image by placement ID
    pub fn get_image(&self, placement_id: u32) -> Option<&ImageData> {
        self.images.get(&placement_id)
    }

    /// Remove cached image
    pub fn remove_image(&mut self, placement_id: u32) -> Option<ImageData> {
        self.images.remove(&placement_id)
    }

    /// Clear all cached images
    pub fn clear(&mut self) {
        self.images.clear();
        self.chunks.clear();
    }

    /// Get number of cached images
    pub fn image_count(&self) -> usize {
        self.images.len()
    }
}

/// Image renderer for calculating display properties
#[derive(Debug)]
pub struct ImageRenderer {
    /// Character cell width in pixels
    cell_width: f32,
    /// Character cell height in pixels
    cell_height: f32,
}

impl ImageRenderer {
    /// Create a new image renderer
    pub fn new(cell_width: f32, cell_height: f32) -> Self {
        Self {
            cell_width,
            cell_height,
        }
    }

    /// Calculate display size in pixels for given cell dimensions
    pub fn cells_to_pixels(&self, cols: u16, rows: u16) -> (f32, f32) {
        (
            cols as f32 * self.cell_width,
            rows as f32 * self.cell_height,
        )
    }

    /// Calculate cell dimensions needed for given pixel size
    pub fn pixels_to_cells(&self, width: f32, height: f32) -> (u16, u16) {
        (
            (width / self.cell_width).ceil() as u16,
            (height / self.cell_height).ceil() as u16,
        )
    }

    /// Calculate scaled dimensions preserving aspect ratio
    pub fn scale_preserve_aspect(
        &self,
        image_width: u32,
        image_height: u32,
        max_cols: u16,
        max_rows: u16,
    ) -> (u16, u16) {
        let max_width = max_cols as f32 * self.cell_width;
        let max_height = max_rows as f32 * self.cell_height;

        let width_ratio = max_width / image_width as f32;
        let height_ratio = max_height / image_height as f32;
        let scale = width_ratio.min(height_ratio);

        let scaled_width = (image_width as f32 * scale).round();
        let scaled_height = (image_height as f32 * scale).round();

        self.pixels_to_cells(scaled_width, scaled_height)
    }

    /// Calculate display rectangle for image at given position
    pub fn calculate_display_rect(
        &self,
        image: &ImageData,
        terminal_cols: u16,
        terminal_rows: u16,
    ) -> Option<DisplayRect> {
        let (cols, rows) = if let Some((cols, rows)) = image.cell_size {
            (cols, rows)
        } else if let Some((width, height)) = image.dimensions {
            // Auto-fit to terminal if no cell size specified
            self.scale_preserve_aspect(width, height, terminal_cols, terminal_rows)
        } else {
            return None;
        };

        let (col, row) = image.position.unwrap_or((0, 0));
        let (px_width, px_height) = self.cells_to_pixels(cols, rows);

        Some(DisplayRect {
            x: col as f32 * self.cell_width,
            y: row as f32 * self.cell_height,
            width: px_width,
            height: px_height,
            cols,
            rows,
        })
    }
}

/// Display rectangle for image rendering
#[derive(Debug, Clone, Copy)]
pub struct DisplayRect {
    /// X position in pixels
    pub x: f32,
    /// Y position in pixels
    pub y: f32,
    /// Width in pixels
    pub width: f32,
    /// Height in pixels
    pub height: f32,
    /// Width in cells
    pub cols: u16,
    /// Height in cells
    pub rows: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_detection() {
        // PNG magic bytes
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(ImageFormat::from_bytes(&png_data), Some(ImageFormat::PNG));

        // JPEG magic bytes
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(
            ImageFormat::from_bytes(&jpeg_data),
            Some(ImageFormat::JPEG)
        );

        // GIF magic bytes
        let gif_data = b"GIF89a".to_vec();
        assert_eq!(ImageFormat::from_bytes(&gif_data), Some(ImageFormat::GIF));

        // Unknown format
        let unknown_data = vec![0x00, 0x00, 0x00, 0x00];
        assert_eq!(ImageFormat::from_bytes(&unknown_data), None);
    }

    #[test]
    fn test_image_format_mime_type() {
        assert_eq!(ImageFormat::PNG.mime_type(), "image/png");
        assert_eq!(ImageFormat::JPEG.mime_type(), "image/jpeg");
        assert_eq!(ImageFormat::GIF.mime_type(), "image/gif");
        assert_eq!(ImageFormat::SIXEL.mime_type(), "image/x-sixel");
    }

    #[test]
    fn test_iterm_params_parsing() {
        let params = ITermParams::parse("name=test.png;width=80;height=24;inline=1");
        assert_eq!(params.name, Some("test.png".to_string()));
        assert_eq!(params.width, Some("80".to_string()));
        assert_eq!(params.height, Some("24".to_string()));
        assert!(params.inline);
    }

    #[test]
    fn test_iterm_dimension_parsing() {
        assert_eq!(ITermParams::parse_dimension("80px"), Some(80));
        assert_eq!(ITermParams::parse_dimension("40"), Some(40));
        assert_eq!(ITermParams::parse_dimension("auto"), None);
    }

    #[test]
    fn test_kitty_params_parsing() {
        let params = KittyParams::parse("a=t,f=100,s=800,v=600,c=80,r=24");
        assert_eq!(params.action, Some('t'));
        assert_eq!(params.format, Some(100));
        assert_eq!(params.width, Some(800));
        assert_eq!(params.height, Some(600));
        assert_eq!(params.columns, Some(80));
        assert_eq!(params.rows, Some(24));
    }

    #[test]
    fn test_image_data_creation() {
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let image = ImageData::new(png_data.clone()).unwrap();
        assert_eq!(image.format, ImageFormat::PNG);
        assert_eq!(image.data, png_data);
        assert!(image.preserve_aspect);
    }

    #[test]
    fn test_image_data_builder() {
        let data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let image = ImageData::new(data)
            .unwrap()
            .with_dimensions(800, 600)
            .with_cell_size(80, 24)
            .with_placement_id(42)
            .with_position(10, 5);

        assert_eq!(image.dimensions, Some((800, 600)));
        assert_eq!(image.cell_size, Some((80, 24)));
        assert_eq!(image.placement_id, Some(42));
        assert_eq!(image.position, Some((10, 5)));
    }

    #[test]
    fn test_iterm2_protocol_parsing() {
        let mut protocol = ImageProtocol::new();

        // Create a simple base64 encoded PNG header
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, png_header);
        let sequence = format!("File=name=test.png;inline=1:{}", encoded);

        let result = protocol.parse_iterm2(&sequence);
        assert!(result.is_ok());

        let image = result.unwrap();
        assert_eq!(image.format, ImageFormat::PNG);
        assert_eq!(image.placement_id, Some(0));
    }

    #[test]
    fn test_kitty_protocol_parsing() {
        let mut protocol = ImageProtocol::new();

        // Create a simple base64 encoded PNG header
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, png_header);

        let result = protocol.parse_kitty("a=t,f=100,s=800,v=600", &encoded);
        assert!(result.is_ok());

        let image = result.unwrap();
        assert_eq!(image.format, ImageFormat::PNG);
        assert_eq!(image.dimensions, Some((800, 600)));
    }

    #[test]
    fn test_image_renderer_cells_to_pixels() {
        let renderer = ImageRenderer::new(10.0, 20.0);
        let (width, height) = renderer.cells_to_pixels(80, 24);
        assert_eq!(width, 800.0);
        assert_eq!(height, 480.0);
    }

    #[test]
    fn test_image_renderer_pixels_to_cells() {
        let renderer = ImageRenderer::new(10.0, 20.0);
        let (cols, rows) = renderer.pixels_to_cells(800.0, 480.0);
        assert_eq!(cols, 80);
        assert_eq!(rows, 24);
    }

    #[test]
    fn test_image_renderer_scale_preserve_aspect() {
        let renderer = ImageRenderer::new(10.0, 20.0);

        // Image is 1600x1200, terminal is 80x24 (800x480 pixels)
        let (cols, rows) = renderer.scale_preserve_aspect(1600, 1200, 80, 24);

        // Should scale to fit within 80x24, preserving aspect ratio
        // Aspect ratio is 4:3
        // max_width = 800px, max_height = 480px
        // width_ratio = 800/1600 = 0.5, height_ratio = 480/1200 = 0.4
        // scale = min(0.5, 0.4) = 0.4 (height constrains)
        // scaled: 640x480 pixels = 64x24 cells
        assert_eq!(cols, 64);
        assert_eq!(rows, 24);
    }

    #[test]
    fn test_image_protocol_caching() {
        let mut protocol = ImageProtocol::new();

        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, png_header);
        let sequence = format!("File=inline=1:{}", encoded);

        let image = protocol.parse_iterm2(&sequence).unwrap();
        let placement_id = image.placement_id.unwrap();

        // Should be cached
        assert_eq!(protocol.image_count(), 1);
        assert!(protocol.get_image(placement_id).is_some());

        // Remove and verify
        protocol.remove_image(placement_id);
        assert_eq!(protocol.image_count(), 0);
        assert!(protocol.get_image(placement_id).is_none());
    }

    #[test]
    fn test_display_rect_calculation() {
        let renderer = ImageRenderer::new(10.0, 20.0);

        let image = ImageData::with_format(ImageFormat::PNG, vec![])
            .with_dimensions(800, 600)
            .with_cell_size(40, 20)
            .with_position(5, 3);

        let rect = renderer.calculate_display_rect(&image, 80, 24).unwrap();

        assert_eq!(rect.x, 50.0); // 5 * 10
        assert_eq!(rect.y, 60.0); // 3 * 20
        assert_eq!(rect.width, 400.0); // 40 * 10
        assert_eq!(rect.height, 400.0); // 20 * 20
        assert_eq!(rect.cols, 40);
        assert_eq!(rect.rows, 20);
    }

    #[test]
    fn test_invalid_base64() {
        let mut protocol = ImageProtocol::new();
        let sequence = "File=inline=1:invalid!!!base64";
        let result = protocol.parse_iterm2(&sequence);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ImageError::Base64Error(_)));
    }

    #[test]
    fn test_invalid_protocol_format() {
        let mut protocol = ImageProtocol::new();

        // Missing separator
        let result = protocol.parse_iterm2("File=no_separator");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ImageError::InvalidProtocol(_)
        ));

        // Missing prefix
        let result = protocol.parse_iterm2("Invalid=test:data");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ImageError::InvalidProtocol(_)
        ));
    }
}
