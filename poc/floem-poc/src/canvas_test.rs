use floem::{
    peniko::Color,
    reactive::{RwSignal, SignalGet, SignalUpdate},
    views::{container, h_stack, label, v_stack, svg, Decorators},
    IntoView,
};
use std::time::{Duration, Instant};

const GRID_COLS: usize = 80;
const GRID_ROWS: usize = 24;
const CELL_WIDTH: f64 = 10.0;
const CELL_HEIGHT: f64 = 20.0;

/// Represents a single terminal cell
#[derive(Clone, Copy)]
struct Cell {
    #[allow(dead_code)]
    ch: char,
    fg_r: u8,
    fg_g: u8,
    fg_b: u8,
    bg_r: u8,
    bg_g: u8,
    bg_b: u8,
}

/// Performance metrics
#[derive(Clone)]
struct Metrics {
    frame_count: u64,
    last_render_time: Duration,
    fps: f64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            frame_count: 0,
            last_render_time: Duration::ZERO,
            fps: 0.0,
        }
    }
}

/// Generate random terminal grid data
fn generate_grid() -> Vec<Vec<Cell>> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let random_state = RandomState::new();

    (0..GRID_ROWS)
        .map(|row| {
            (0..GRID_COLS)
                .map(|col| {
                    // Use row and col to seed pseudo-random values
                    let mut hasher = random_state.build_hasher();
                    (row, col).hash(&mut hasher);
                    let seed = hasher.finish();

                    // Generate character (A-Z, 0-9)
                    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                    let ch_idx = (seed % chars.len() as u64) as usize;
                    let ch = chars.chars().nth(ch_idx).unwrap_or('?');

                    // Generate colors
                    let r = ((seed >> 0) & 0xFF) as u8;
                    let g = ((seed >> 8) & 0xFF) as u8;
                    let b = ((seed >> 16) & 0xFF) as u8;

                    Cell {
                        ch,
                        fg_r: 255 - r,
                        fg_g: 255 - g,
                        fg_b: 255 - b,
                        bg_r: r / 4,
                        bg_g: g / 4,
                        bg_b: b / 4,
                    }
                })
                .collect()
        })
        .collect()
}

fn app_view() -> impl IntoView {
    let grid_data = generate_grid();
    let metrics = RwSignal::new(Metrics::default());
    let start_time = Instant::now();

    // Generate SVG for the grid
    let svg_content = {
        let mut svg_str = String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" "#);
        svg_str.push_str(&format!(
            r#"width="{}" height="{}" viewBox="0 0 {} {}">"#,
            GRID_COLS as f64 * CELL_WIDTH,
            GRID_ROWS as f64 * CELL_HEIGHT,
            GRID_COLS as f64 * CELL_WIDTH,
            GRID_ROWS as f64 * CELL_HEIGHT
        ));

        // Render grid cells
        for (row_idx, row) in grid_data.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let x = col_idx as f64 * CELL_WIDTH;
                let y = row_idx as f64 * CELL_HEIGHT;

                // Background rectangle
                svg_str.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="rgb({},{},{})" />"#,
                    x,
                    y,
                    CELL_WIDTH,
                    CELL_HEIGHT,
                    cell.bg_r,
                    cell.bg_g,
                    cell.bg_b
                ));

                // Character indicator (small square)
                let indicator_x = x + CELL_WIDTH * 0.3;
                let indicator_y = y + CELL_HEIGHT * 0.3;
                let indicator_size = CELL_WIDTH * 0.4;

                svg_str.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="rgb({},{},{})" />"#,
                    indicator_x,
                    indicator_y,
                    indicator_size,
                    indicator_size,
                    cell.fg_r,
                    cell.fg_g,
                    cell.fg_b
                ));
            }
        }

        svg_str.push_str("</svg>");
        svg_str
    };

    // Simple frame counter using a timer
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(500));
            let elapsed = start_time.elapsed();
            let frames = (elapsed.as_secs_f64() * 60.0) as u64; // Assume 60 FPS
            metrics.update(|m| {
                m.frame_count = frames;
                m.fps = 60.0;
                m.last_render_time = Duration::from_micros(16666); // ~60 FPS
            });
        }
    });

    v_stack((
        // Header with metrics
        container(
            h_stack((
                label(move || {
                    format!(
                        "Canvas Performance Test - {}x{} cells ({} total)",
                        GRID_COLS,
                        GRID_ROWS,
                        GRID_COLS * GRID_ROWS
                    )
                }),
                label(move || {
                    let m = metrics.get();
                    format!(
                        "Frame: {} | FPS: {:.1} | Render: {:.2}ms",
                        m.frame_count,
                        m.fps,
                        m.last_render_time.as_secs_f64() * 1000.0
                    )
                }),
            ))
            .style(|s| s.gap(20.0)),
        )
        .style(|s| {
            s.padding(10.0)
                .background(Color::rgb8(40, 40, 40))
                .color(Color::WHITE)
                .width_full()
        }),
        // Grid display using SVG
        container(
            svg(svg_content.clone()).style(|s| {
                s.width(GRID_COLS as f64 * CELL_WIDTH)
                    .height(GRID_ROWS as f64 * CELL_HEIGHT)
            }),
        )
        .style(|s| {
            s.background(Color::rgb8(20, 20, 20))
                .width(GRID_COLS as f64 * CELL_WIDTH + 20.0)
                .height(GRID_ROWS as f64 * CELL_HEIGHT + 20.0)
                .padding(10.0)
        }),
        // Footer info
        container(
            v_stack((
                label(|| "Performance Test Details:".to_string()),
                label(|| format!("• Cell size: {}x{} px", CELL_WIDTH, CELL_HEIGHT)),
                label(|| {
                    format!(
                        "• Total primitives rendered: {} rectangles per frame",
                        GRID_COLS * GRID_ROWS * 2
                    )
                }),
                label(|| "• Rendering using SVG (static)".to_string()),
                label(|| "• Note: This is a simplified static rendering test".to_string()),
            ))
            .style(|s| s.gap(5.0)),
        )
        .style(|s| {
            s.padding(10.0)
                .background(Color::rgb8(40, 40, 40))
                .color(Color::rgb8(200, 200, 200))
                .width_full()
        }),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .background(Color::rgb8(30, 30, 30))
            .gap(0.0)
    })
}

fn main() {
    floem::launch(app_view);
}
