//! Image Protocol Demo
//!
//! This example demonstrates the image protocol parsing functionality.
//! Run with: cargo run --example image_protocol_demo

use agterm::image_protocol::{ImageData, ImageFormat, ImageProtocol, ImageRenderer};

fn main() {
    println!("=== AgTerm Image Protocol Demo ===\n");

    // Create protocol handler
    let mut protocol = ImageProtocol::new();

    // Demo 1: Format Detection
    println!("1. Image Format Detection:");
    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
    let gif_data = b"GIF89a".to_vec();

    println!("  PNG detected: {:?}", ImageFormat::from_bytes(&png_data));
    println!("  JPEG detected: {:?}", ImageFormat::from_bytes(&jpeg_data));
    println!("  GIF detected: {:?}", ImageFormat::from_bytes(&gif_data));
    println!();

    // Demo 2: iTerm2 Protocol
    println!("2. iTerm2 Protocol Parsing:");
    let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_header);
    let sequence = format!("File=name=test.png;width=80;height=24;inline=1:{}", encoded);

    match protocol.parse_iterm2(&sequence) {
        Ok(image) => {
            println!("  Format: {:?}", image.format);
            println!("  Cell size: {:?}", image.cell_size);
            println!("  Placement ID: {:?}", image.placement_id);
            println!("  Preserve aspect: {}", image.preserve_aspect);
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Demo 3: Kitty Protocol
    println!("3. Kitty Protocol Parsing:");
    let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_header);

    match protocol.parse_kitty("a=t,f=100,s=800,v=600,c=80,r=24", &encoded) {
        Ok(image) => {
            println!("  Format: {:?}", image.format);
            println!("  Dimensions: {:?}", image.dimensions);
            println!("  Cell size: {:?}", image.cell_size);
            println!("  Placement ID: {:?}", image.placement_id);
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Demo 4: Image Builder
    println!("4. Image Data Builder:");
    let image = ImageData::with_format(ImageFormat::PNG, vec![0x89, 0x50, 0x4E, 0x47])
        .with_dimensions(1920, 1080)
        .with_cell_size(80, 24)
        .with_position(10, 5);

    println!("  Format: {:?}", image.format);
    println!("  Dimensions: {:?}", image.dimensions);
    println!("  Cell size: {:?}", image.cell_size);
    println!("  Position: {:?}", image.position);
    println!();

    // Demo 5: Image Renderer
    println!("5. Image Renderer:");
    let renderer = ImageRenderer::new(10.0, 20.0); // 10px wide, 20px tall cells

    let (px_width, px_height) = renderer.cells_to_pixels(80, 24);
    println!("  80x24 cells = {}x{} pixels", px_width, px_height);

    let (cols, rows) = renderer.pixels_to_cells(800.0, 480.0);
    println!("  800x480 pixels = {}x{} cells", cols, rows);

    let (scaled_cols, scaled_rows) = renderer.scale_preserve_aspect(1920, 1080, 80, 40);
    println!(
        "  1920x1080 image scaled to fit 80x40 = {}x{} cells",
        scaled_cols, scaled_rows
    );
    println!();

    // Demo 6: Cache Management
    println!("6. Image Cache:");
    println!("  Cached images: {}", protocol.image_count());
    if protocol.image_count() > 0 {
        if let Some(image) = protocol.get_image(0) {
            println!("  Image 0: {:?}", image.format);
        }
    }
    println!();

    println!("=== Demo Complete ===");
}
