//! Terminal screen buffer with ANSI escape code parsing

use iced::Color;
use std::cmp::{max, min};
use std::collections::VecDeque;
use std::sync::Arc;
use unicode_width::UnicodeWidthChar;
use vte::{Params, Parser, Perform};

mod memory;
mod scrollback;
pub use memory::{MemoryStats, StringInterner};
// Reserved for future scrollback buffer implementation
#[allow(unused_imports)]
pub use scrollback::{ScrollbackBuffer, ScrollbackConfig};

use std::collections::HashSet;

/// Maximum scrollback buffer lines
const MAX_SCROLLBACK: usize = 10000;

/// Dirty flag system for incremental rendering optimization
///
/// Tracks which lines have changed since the last render to avoid
/// redrawing the entire screen on every frame.
#[derive(Debug, Clone)]
pub struct DirtyTracker {
    /// Set of line indices that have been modified
    dirty_lines: HashSet<usize>,
    /// Flag indicating that a full screen redraw is needed
    full_redraw: bool,
    /// Flag indicating only the cursor position changed
    cursor_only: bool,
}

impl Default for DirtyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl DirtyTracker {
    /// Create a new dirty tracker with full redraw initially required
    pub fn new() -> Self {
        Self {
            dirty_lines: HashSet::new(),
            full_redraw: true,
            cursor_only: false,
        }
    }

    /// Mark a specific line as dirty
    pub fn mark_line(&mut self, line: usize) {
        if !self.full_redraw {
            self.dirty_lines.insert(line);
        }
        self.cursor_only = false;
    }

    /// Mark multiple lines as dirty
    pub fn mark_lines(&mut self, lines: impl Iterator<Item = usize>) {
        if !self.full_redraw {
            self.dirty_lines.extend(lines);
        }
        self.cursor_only = false;
    }

    /// Mark a range of lines as dirty
    pub fn mark_range(&mut self, start: usize, end: usize) {
        if !self.full_redraw {
            for line in start..=end {
                self.dirty_lines.insert(line);
            }
        }
        self.cursor_only = false;
    }

    /// Mark all lines as dirty (full redraw)
    pub fn mark_all(&mut self) {
        self.full_redraw = true;
        self.dirty_lines.clear();
        self.cursor_only = false;
    }

    /// Mark only cursor update needed (most efficient)
    pub fn mark_cursor(&mut self) {
        if !self.full_redraw && self.dirty_lines.is_empty() {
            self.cursor_only = true;
        }
    }

    /// Clear all dirty flags after render
    pub fn clear(&mut self) {
        self.dirty_lines.clear();
        self.full_redraw = false;
        self.cursor_only = false;
    }

    /// Check if a specific line is dirty
    pub fn is_line_dirty(&self, line: usize) -> bool {
        self.full_redraw || self.dirty_lines.contains(&line)
    }

    /// Check if any redraw is needed
    pub fn needs_redraw(&self) -> bool {
        self.full_redraw || !self.dirty_lines.is_empty() || self.cursor_only
    }

    /// Check if full redraw is required
    pub fn needs_full_redraw(&self) -> bool {
        self.full_redraw
    }

    /// Check if only cursor update is needed
    pub fn is_cursor_only(&self) -> bool {
        self.cursor_only && !self.full_redraw && self.dirty_lines.is_empty()
    }

    /// Get the set of dirty lines
    pub fn dirty_lines(&self) -> &HashSet<usize> {
        &self.dirty_lines
    }

    /// Get count of dirty lines
    pub fn dirty_count(&self) -> usize {
        if self.full_redraw {
            usize::MAX
        } else {
            self.dirty_lines.len()
        }
    }
}

/// ANSI color (16-color palette + 256-color + RGB)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnsiColor {
    /// Standard 16-color palette (0-15)
    Indexed(u8),
    /// 256-color palette (0-255)
    Palette256(u8),
    /// RGB color
    Rgb(u8, u8, u8),
}

impl AnsiColor {
    /// Convert ANSI color to Iced Color
    pub fn to_color(&self) -> Color {
        match self {
            AnsiColor::Indexed(idx) => indexed_to_color(*idx),
            AnsiColor::Palette256(idx) => palette256_to_color(*idx),
            AnsiColor::Rgb(r, g, b) => {
                Color::from_rgb(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0)
            }
        }
    }
}

/// Convert 16-color index to Iced Color
fn indexed_to_color(idx: u8) -> Color {
    match idx {
        0 => Color::from_rgb(0.0, 0.0, 0.0),  // Black
        1 => Color::from_rgb(0.8, 0.2, 0.2),  // Red
        2 => Color::from_rgb(0.2, 0.8, 0.2),  // Green
        3 => Color::from_rgb(0.8, 0.8, 0.2),  // Yellow
        4 => Color::from_rgb(0.2, 0.2, 0.8),  // Blue
        5 => Color::from_rgb(0.8, 0.2, 0.8),  // Magenta
        6 => Color::from_rgb(0.2, 0.8, 0.8),  // Cyan
        7 => Color::from_rgb(0.8, 0.8, 0.8),  // White
        8 => Color::from_rgb(0.5, 0.5, 0.5),  // Bright Black (Gray)
        9 => Color::from_rgb(1.0, 0.3, 0.3),  // Bright Red
        10 => Color::from_rgb(0.3, 1.0, 0.3), // Bright Green
        11 => Color::from_rgb(1.0, 1.0, 0.3), // Bright Yellow
        12 => Color::from_rgb(0.3, 0.3, 1.0), // Bright Blue
        13 => Color::from_rgb(1.0, 0.3, 1.0), // Bright Magenta
        14 => Color::from_rgb(0.3, 1.0, 1.0), // Bright Cyan
        15 => Color::from_rgb(1.0, 1.0, 1.0), // Bright White
        _ => Color::from_rgb(0.8, 0.8, 0.8),  // Default
    }
}

/// Convert 256-color palette index to Iced Color
fn palette256_to_color(idx: u8) -> Color {
    match idx {
        // 0-15: Standard colors
        0..=15 => indexed_to_color(idx),
        // 16-231: 6x6x6 color cube
        16..=231 => {
            let idx = idx - 16;
            let r = (idx / 36) * 51;
            let g = ((idx % 36) / 6) * 51;
            let b = (idx % 6) * 51;
            Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        }
        // 232-255: Grayscale
        232..=255 => {
            let gray = 8 + (idx - 232) * 10;
            Color::from_rgb(
                gray as f32 / 255.0,
                gray as f32 / 255.0,
                gray as f32 / 255.0,
            )
        }
    }
}

/// Mouse reporting mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseMode {
    /// No mouse reporting
    #[default]
    None,
    /// X10 Mouse Reporting (CSI ?9h or CSI ?1000h)
    X10,
    /// Button-Event Mouse Tracking (CSI ?1002h)
    ButtonEvent,
    /// Any-Event Mouse Tracking (CSI ?1003h)
    AnyEvent,
}

/// Mouse encoding mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseEncoding {
    /// Default X10/UTF-8 encoding
    #[default]
    Default,
    /// SGR Extended Mouse Mode (CSI ?1006h)
    Sgr,
}

/// Terminal cell with character and styling
///
/// Memory optimizations:
/// - Uses `Arc<String>` for hyperlinks to enable string interning
/// - Compact flag representation using bitfields would save ~8 bytes per cell
/// - Size: ~56 bytes (char=4, AnsiColor=~8, flags=7, `Arc<String>`=8)
#[derive(Clone, Debug)]
pub struct Cell {
    pub c: char,
    pub fg: Option<AnsiColor>,
    /// Background color (reserved for future rendering enhancement)
    #[allow(dead_code)]
    pub bg: Option<AnsiColor>,
    pub bold: bool,
    pub underline: bool,
    /// Reverse video (reserved for future rendering enhancement)
    #[allow(dead_code)]
    pub reverse: bool,
    pub dim: bool,
    pub italic: bool,
    pub strikethrough: bool,
    /// This cell is the first cell of a wide character (CJK, emoji, etc.)
    pub wide: bool,
    /// This cell is a placeholder for the second cell of a wide character
    pub placeholder: bool,
    /// Hyperlink URL (for OSC 8 or auto-detected URLs) - uses Arc for string interning
    pub hyperlink: Option<Arc<String>>,
    /// Image data for this cell (for future image rendering support)
    #[allow(dead_code)]
    pub image: Option<ImageData>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: None,
            bg: None,
            bold: false,
            underline: false,
            reverse: false,
            dim: false,
            italic: false,
            strikethrough: false,
            wide: false,
            placeholder: false,
            hyperlink: None,
            image: None,
        }
    }
}

/// Image protocol types supported by terminal emulators
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageProtocol {
    /// Sixel graphics (DEC VT340+)
    Sixel,
    /// Kitty graphics protocol
    Kitty,
    /// iTerm2 inline images protocol
    ITerm2,
}

/// Image data structure for terminal image display
#[derive(Clone, Debug)]
pub struct ImageData {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Raw image data (protocol-specific format)
    pub data: Vec<u8>,
    /// Protocol used for this image
    pub protocol: ImageProtocol,
    /// Image ID for tracking (protocol-specific)
    pub id: Option<u32>,
}

impl ImageData {
    /// Create a new image data instance
    pub fn new(width: u32, height: u32, data: Vec<u8>, protocol: ImageProtocol) -> Self {
        Self {
            width,
            height,
            data,
            protocol,
            id: None,
        }
    }

    /// Create with explicit ID
    pub fn with_id(
        width: u32,
        height: u32,
        data: Vec<u8>,
        protocol: ImageProtocol,
        id: u32,
    ) -> Self {
        Self {
            width,
            height,
            data,
            protocol,
            id: Some(id),
        }
    }

    /// Check if image data exceeds a given size limit
    pub fn exceeds_size_limit(&self, max_size_bytes: usize) -> bool {
        self.data.len() > max_size_bytes
    }

    /// Get the approximate memory size in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.len()
    }
}

/// Saved state for alternate screen buffer switching.
///
/// Stores complete terminal state when entering alternate screen,
/// allowing full restoration when returning to main screen.
/// State saved when entering alternate screen mode
///
/// Preserves the main screen buffer and cursor state for restoration when
/// leaving alternate screen mode (used by vim, less, htop, etc.)
#[derive(Clone, Debug)]
#[allow(dead_code)] // Fields used through derived traits
struct AlternateScreenState {
    /// Saved main screen buffer
    main_buffer: Vec<Vec<Cell>>,
    /// Cursor position (row, col)
    cursor_pos: (usize, usize),
    /// Current text attributes
    current_fg: Option<AnsiColor>,
    current_bg: Option<AnsiColor>,
    bold: bool,
    underline: bool,
    reverse: bool,
    dim: bool,
    italic: bool,
    strikethrough: bool,
    /// Scroll region
    scroll_region: Option<(usize, usize)>,
    /// Saved cursor for DECSC/DECRC
    saved_cursor: Option<(usize, usize)>,
    /// Saved cursor state (DECSC)
    saved_cursor_state: Option<(usize, usize, Option<AnsiColor>, Option<AnsiColor>, bool)>,
    /// Saved scrollback buffer (compressed)
    saved_scrollback: VecDeque<CompressedLine>,
    /// Alternate screen is active
    is_active: bool,
    /// Scrollback disabled in alternate screen (always true for alternate screen)
    scrollback_disabled: bool,
}

/// Run-length encoded segment for compression
#[derive(Clone, Debug)]
struct RleSegment {
    /// The cell to repeat
    cell: Cell,
    /// Number of times this cell repeats
    count: usize,
}

/// Compressed line representation using run-length encoding
#[derive(Clone, Debug)]
pub struct CompressedLine {
    /// Run-length encoded segments
    segments: Vec<RleSegment>,
    /// Original line length (for validation)
    original_length: usize,
    /// Uncompressed size in bytes (approximate)
    uncompressed_size: usize,
    /// Compressed size in bytes (approximate)
    compressed_size: usize,
}

impl CompressedLine {
    /// Compress a line of cells using run-length encoding
    fn compress(line: &[Cell]) -> Self {
        let mut segments = Vec::new();

        if line.is_empty() {
            return Self {
                segments,
                original_length: 0,
                uncompressed_size: 0,
                compressed_size: 0,
            };
        }

        let mut current_cell = line[0].clone();
        let mut count = 1;

        for cell in line.iter().skip(1) {
            // Check if cells are identical for RLE
            if cells_equal(&current_cell, cell) {
                count += 1;
            } else {
                segments.push(RleSegment {
                    cell: current_cell.clone(),
                    count,
                });
                current_cell = cell.clone();
                count = 1;
            }
        }

        // Push the last segment
        segments.push(RleSegment {
            cell: current_cell,
            count,
        });

        let original_length = line.len();
        let uncompressed_size = std::mem::size_of_val(line);
        let compressed_size =
            segments.len() * (std::mem::size_of::<Cell>() + std::mem::size_of::<usize>());

        Self {
            segments,
            original_length,
            uncompressed_size,
            compressed_size,
        }
    }

    /// Decompress the line back to a vector of cells
    fn decompress(&self) -> Vec<Cell> {
        let mut line = Vec::with_capacity(self.original_length);

        for segment in &self.segments {
            for _ in 0..segment.count {
                line.push(segment.cell.clone());
            }
        }

        line
    }

    /// Get compression ratio (compressed_size / uncompressed_size)
    fn compression_ratio(&self) -> f64 {
        if self.uncompressed_size == 0 {
            return 1.0;
        }
        self.compressed_size as f64 / self.uncompressed_size as f64
    }

    /// Get space saved in bytes (used for compression statistics)
    #[allow(dead_code)]
    fn space_saved(&self) -> isize {
        self.uncompressed_size as isize - self.compressed_size as isize
    }

    /// Get uncompressed size in bytes
    #[allow(dead_code)] // Used in debug/profiling contexts
    pub fn uncompressed_size(&self) -> usize {
        self.uncompressed_size
    }

    /// Get compressed size in bytes
    #[allow(dead_code)] // Used in debug/profiling contexts
    pub fn compressed_size(&self) -> usize {
        self.compressed_size
    }
}

/// Check if two cells are equal for compression purposes
fn cells_equal(a: &Cell, b: &Cell) -> bool {
    a.c == b.c
        && a.fg == b.fg
        && a.bg == b.bg
        && a.bold == b.bold
        && a.underline == b.underline
        && a.reverse == b.reverse
        && a.dim == b.dim
        && a.italic == b.italic
        && a.strikethrough == b.strikethrough
        && a.wide == b.wide
        && a.placeholder == b.placeholder
        && match (&a.hyperlink, &b.hyperlink) {
            (None, None) => true,
            (Some(a_link), Some(b_link)) => Arc::ptr_eq(a_link, b_link) || a_link == b_link,
            _ => false,
        }
        // Ignore image comparison for performance - images are rare
        && a.image.is_none() && b.image.is_none()
}

/// Compression statistics for monitoring
#[derive(Clone, Debug, Default)]
pub struct CompressionStats {
    /// Total lines compressed
    pub total_lines: usize,
    /// Total uncompressed size in bytes
    pub total_uncompressed: usize,
    /// Total compressed size in bytes
    pub total_compressed: usize,
    /// Best compression ratio achieved
    pub best_ratio: f64,
    /// Worst compression ratio achieved
    pub worst_ratio: f64,
    /// Average compression ratio
    pub avg_ratio: f64,
}

impl CompressionStats {
    /// Create new empty stats
    fn new() -> Self {
        Self {
            total_lines: 0,
            total_uncompressed: 0,
            total_compressed: 0,
            best_ratio: f64::MAX,
            worst_ratio: 0.0,
            avg_ratio: 0.0,
        }
    }

    /// Update stats with a new compressed line
    fn update(&mut self, compressed: &CompressedLine) {
        self.total_lines += 1;
        self.total_uncompressed += compressed.uncompressed_size;
        self.total_compressed += compressed.compressed_size;

        let ratio = compressed.compression_ratio();
        self.best_ratio = self.best_ratio.min(ratio);
        self.worst_ratio = self.worst_ratio.max(ratio);

        // Update average
        if self.total_uncompressed > 0 {
            self.avg_ratio = self.total_compressed as f64 / self.total_uncompressed as f64;
        }
    }

    /// Get total space saved in bytes
    pub fn space_saved(&self) -> isize {
        self.total_uncompressed as isize - self.total_compressed as isize
    }

    /// Get space saved as percentage
    pub fn space_saved_percent(&self) -> f64 {
        if self.total_uncompressed == 0 {
            return 0.0;
        }
        (self.space_saved() as f64 / self.total_uncompressed as f64) * 100.0
    }
}

/// Terminal screen buffer with VTE parser
pub struct TerminalScreen {
    cols: usize,
    rows: usize,
    /// Screen buffer (rows x cols) - only visible lines
    buffer: Vec<Vec<Cell>>,
    /// Scrollback buffer (historical lines) - compressed for memory efficiency
    scrollback: VecDeque<CompressedLine>,
    /// Compression statistics
    compression_stats: CompressionStats,
    /// Cursor position
    cursor_row: usize,
    cursor_col: usize,
    /// Current text attributes
    current_fg: Option<AnsiColor>,
    current_bg: Option<AnsiColor>,
    bold: bool,
    underline: bool,
    reverse: bool,
    dim: bool,
    italic: bool,
    strikethrough: bool,
    /// VTE parser
    parser: Parser,
    /// Scroll region (top, bottom) - None means full screen
    scroll_region: Option<(usize, usize)>,
    /// Saved cursor position (for save/restore)
    saved_cursor: Option<(usize, usize)>,
    /// Saved cursor state (DECSC) - position, fg, bg, bold
    saved_cursor_state: Option<(usize, usize, Option<AnsiColor>, Option<AnsiColor>, bool)>,
    /// OSC sequence fields
    /// Window title (OSC 0 or OSC 2)
    window_title: Option<String>,
    /// Icon name (OSC 1)
    icon_name: Option<String>,
    /// Current working directory from shell (OSC 7)
    cwd_from_shell: Option<String>,
    /// Clipboard request data (OSC 52)
    clipboard_request: Option<String>,
    /// Alternate screen buffer (for applications like vim, less, etc.)
    alternate_buffer: Option<Vec<Vec<Cell>>>,
    /// Alternate scrollback buffer (compressed)
    alternate_scrollback: Option<VecDeque<CompressedLine>>,
    /// Whether we're currently using the alternate screen
    use_alternate_screen: bool,
    /// Saved main screen state (cursor pos, attributes, scroll region, etc.)
    alternate_saved_state: Option<AlternateScreenState>,
    /// Mouse reporting mode
    mouse_mode: MouseMode,
    /// Mouse encoding mode
    mouse_encoding: MouseEncoding,
    /// Cursor visibility (DECTCEM)
    cursor_visible: bool,
    /// Auto-wrap mode (DECAWM) - default true
    auto_wrap_mode: bool,
    /// Bracketed paste mode (CSI ?2004h/l) - tracked for terminal compatibility
    #[allow(dead_code)]
    bracketed_paste_mode: bool,
    /// Application cursor keys mode (DECCKM - CSI ?1h/l)
    application_cursor_keys: bool,
    /// Pending responses to be sent to PTY (for DA, DSR, CPR, etc.)
    pending_responses: Vec<String>,
    /// Bell (BEL) triggered flag - set when \x07 is received
    bell_triggered: bool,
    /// Cursor blink enabled (CSI ?12h/l)
    cursor_blink_enabled: bool,
    /// Color palette (256 colors) - RGB values for customization
    /// Index 0-15: standard colors, 16-255: extended palette
    color_palette: Vec<(u8, u8, u8)>,
    /// Default foreground color (OSC 10)
    default_fg_color: Option<(u8, u8, u8)>,
    /// Default background color (OSC 11)
    default_bg_color: Option<(u8, u8, u8)>,
    /// String interner for memory optimization (hyperlinks, URLs)
    string_interner: StringInterner,
    /// Cleanup counter - periodically clean up unused interned strings
    interner_cleanup_counter: usize,
    /// Dirty tracking for incremental rendering
    dirty_tracker: DirtyTracker,
}

impl TerminalScreen {
    /// Create a new terminal screen
    pub fn new(cols: usize, rows: usize) -> Self {
        let cols = max(1, cols);
        let rows = max(1, rows);

        Self {
            cols,
            rows,
            buffer: vec![vec![Cell::default(); cols]; rows],
            scrollback: VecDeque::new(),
            compression_stats: CompressionStats::new(),
            cursor_row: 0,
            cursor_col: 0,
            current_fg: None,
            current_bg: None,
            bold: false,
            underline: false,
            reverse: false,
            dim: false,
            italic: false,
            strikethrough: false,
            parser: Parser::new(),
            scroll_region: None,
            saved_cursor: None,
            saved_cursor_state: None,
            window_title: None,
            icon_name: None,
            cwd_from_shell: None,
            clipboard_request: None,
            alternate_buffer: None,
            alternate_scrollback: None,
            use_alternate_screen: false,
            alternate_saved_state: None,
            mouse_mode: MouseMode::None,
            mouse_encoding: MouseEncoding::Default,
            cursor_visible: true,
            auto_wrap_mode: true,
            bracketed_paste_mode: false,
            application_cursor_keys: false,
            pending_responses: Vec::new(),
            bell_triggered: false,
            cursor_blink_enabled: true,
            color_palette: Self::initialize_default_palette(),
            default_fg_color: None,
            default_bg_color: None,
            string_interner: StringInterner::new(),
            interner_cleanup_counter: 0,
            dirty_tracker: DirtyTracker::new(),
        }
    }

    /// Initialize the default 256-color palette
    fn initialize_default_palette() -> Vec<(u8, u8, u8)> {
        let mut palette = Vec::with_capacity(256);

        // 0-15: Standard 16 colors
        palette.extend_from_slice(&[
            (0, 0, 0),       // 0: Black
            (204, 51, 51),   // 1: Red
            (51, 204, 51),   // 2: Green
            (204, 204, 51),  // 3: Yellow
            (51, 51, 204),   // 4: Blue
            (204, 51, 204),  // 5: Magenta
            (51, 204, 204),  // 6: Cyan
            (204, 204, 204), // 7: White
            (127, 127, 127), // 8: Bright Black (Gray)
            (255, 76, 76),   // 9: Bright Red
            (76, 255, 76),   // 10: Bright Green
            (255, 255, 76),  // 11: Bright Yellow
            (76, 76, 255),   // 12: Bright Blue
            (255, 76, 255),  // 13: Bright Magenta
            (76, 255, 255),  // 14: Bright Cyan
            (255, 255, 255), // 15: Bright White
        ]);

        // 16-231: 6x6x6 color cube
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    palette.push((r * 51, g * 51, b * 51));
                }
            }
        }

        // 232-255: Grayscale
        for i in 0..24 {
            let gray = 8 + i * 10;
            palette.push((gray, gray, gray));
        }

        palette
    }

    /// Process incoming bytes through VTE parser
    pub fn process(&mut self, bytes: &[u8]) {
        // We need to temporarily take the parser to avoid borrow checker issues
        let mut parser = std::mem::replace(&mut self.parser, Parser::new());
        for byte in bytes {
            parser.advance(self, *byte);
        }
        self.parser = parser;
    }

    /// Resize the terminal with content preservation
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let cols = max(1, cols);
        let rows = max(1, rows);

        if cols == self.cols && rows == self.rows {
            return;
        }

        // Save old dimensions
        let old_cols = self.cols;
        let old_rows = self.rows;

        // Handle column resize first (reflow lines)
        let mut new_buffer = Vec::new();

        if cols != old_cols {
            // Reflow each line to new width
            for row in &self.buffer {
                let mut new_row = Vec::with_capacity(cols);

                if cols > old_cols {
                    // Columns increased: extend with empty cells
                    new_row.extend_from_slice(&row[..old_cols]);
                    new_row.resize(cols, Cell::default());
                } else {
                    // Columns decreased: truncate long lines
                    new_row.extend_from_slice(&row[..cols]);
                }

                new_buffer.push(new_row);
            }
        } else {
            // No column change, just copy buffer
            new_buffer = self.buffer.clone();
        }

        // Handle row resize
        if rows < old_rows {
            // Rows decreased: move top lines to scrollback (compressed)
            let lines_to_save = old_rows - rows;
            for i in 0..lines_to_save {
                if i < new_buffer.len() {
                    let compressed = CompressedLine::compress(&new_buffer[i]);
                    self.compression_stats.update(&compressed);
                    self.scrollback.push_back(compressed);
                }
            }

            // Keep only the bottom 'rows' lines
            if new_buffer.len() > rows {
                new_buffer.drain(0..lines_to_save);
            }

            // Limit scrollback
            while self.scrollback.len() > MAX_SCROLLBACK {
                self.scrollback.pop_front();
            }
        } else if rows > old_rows {
            // Rows increased: restore from scrollback if available
            let lines_to_restore = min(rows - old_rows, self.scrollback.len());

            // Restore lines from scrollback (from the end) - decompress
            let mut restored_lines = Vec::new();
            for _ in 0..lines_to_restore {
                if let Some(compressed_line) = self.scrollback.pop_back() {
                    let mut line = compressed_line.decompress();
                    // Reflow restored line to new column width
                    if cols != old_cols {
                        line.resize(cols, Cell::default());
                    }
                    restored_lines.push(line);
                }
            }

            // Insert restored lines at the beginning
            restored_lines.reverse();
            new_buffer.splice(0..0, restored_lines);

            // If still need more rows, add empty ones at the end
            while new_buffer.len() < rows {
                new_buffer.push(vec![Cell::default(); cols]);
            }
        }

        // Ensure buffer is exactly 'rows' lines
        new_buffer.resize(rows, vec![Cell::default(); cols]);

        // Update terminal state
        self.buffer = new_buffer;
        self.cols = cols;
        self.rows = rows;

        // Adjust cursor position to be within new bounds
        self.cursor_row = min(self.cursor_row, rows.saturating_sub(1));
        self.cursor_col = min(self.cursor_col, cols.saturating_sub(1));

        // Reset scroll region as it's no longer valid
        self.scroll_region = None;

        // Also handle alternate screen buffer if active
        if self.use_alternate_screen {
            if let Some(ref mut alt_buffer) = self.alternate_buffer {
                // Resize alternate buffer similarly
                let mut resized_alt = Vec::new();
                for row in alt_buffer.iter() {
                    let mut new_row = Vec::with_capacity(cols);
                    if cols > old_cols {
                        let copy_len = min(row.len(), old_cols);
                        new_row.extend_from_slice(&row[..copy_len]);
                        new_row.resize(cols, Cell::default());
                    } else {
                        let copy_len = min(row.len(), cols);
                        new_row.extend_from_slice(&row[..copy_len]);
                    }
                    resized_alt.push(new_row);
                }
                resized_alt.resize(rows, vec![Cell::default(); cols]);
                *alt_buffer = resized_alt;
            }
        }

        // Mark all as dirty after resize
        self.dirty_tracker.mark_all();
    }

    /// Get all lines (scrollback + visible) for rendering
    pub fn get_all_lines(&self) -> Vec<Vec<Cell>> {
        // Decompress scrollback lines lazily
        let mut all_lines: Vec<Vec<Cell>> = self
            .scrollback
            .iter()
            .map(|compressed| compressed.decompress())
            .collect();
        all_lines.extend(self.buffer.clone());
        all_lines
    }

    /// Auto-detect URLs in all lines and update cell hyperlinks
    pub fn detect_urls(&mut self) {
        // URL regex pattern: http://, https://, file://
        let url_pattern =
            regex::Regex::new(r"(?i)(https?://[^\s<>{}|\\\^\[\]`]+|file://[^\s<>{}|\\\^\[\]`]+)")
                .unwrap();

        // Process all lines in buffer
        for row in &mut self.buffer {
            // Convert row to string for regex matching
            let line_text: String = row
                .iter()
                .filter(|cell| !cell.placeholder)
                .map(|cell| cell.c)
                .collect();

            // Find all URL matches
            for mat in url_pattern.find_iter(&line_text) {
                let url = mat.as_str().to_string();
                let start_col = mat.start();
                let end_col = mat.end();

                // Intern the URL string to share memory across cells
                let interned_url = self.string_interner.intern(url);

                // Update cells with hyperlink
                let mut char_index = 0;
                for cell in row.iter_mut() {
                    if cell.placeholder {
                        continue;
                    }
                    if char_index >= start_col && char_index < end_col {
                        cell.hyperlink = Some(Arc::clone(&interned_url));
                    }
                    char_index += 1;
                }
            }
        }

        // Process scrollback buffer - decompress, update, recompress
        for compressed_line in self.scrollback.iter_mut() {
            let mut row = compressed_line.decompress();

            let line_text: String = row
                .iter()
                .filter(|cell| !cell.placeholder)
                .map(|cell| cell.c)
                .collect();

            let mut modified = false;
            for mat in url_pattern.find_iter(&line_text) {
                let url = mat.as_str().to_string();
                let start_col = mat.start();
                let end_col = mat.end();

                // Intern the URL string
                let interned_url = self.string_interner.intern(url);

                let mut char_index = 0;
                for cell in row.iter_mut() {
                    if cell.placeholder {
                        continue;
                    }
                    if char_index >= start_col && char_index < end_col {
                        cell.hyperlink = Some(Arc::clone(&interned_url));
                        modified = true;
                    }
                    char_index += 1;
                }
            }

            // Recompress the line if modified
            if modified {
                *compressed_line = CompressedLine::compress(&row);
            }
        }

        // Periodically clean up unused interned strings (every 100 calls)
        self.interner_cleanup_counter += 1;
        if self.interner_cleanup_counter >= 100 {
            self.string_interner.cleanup();
            self.interner_cleanup_counter = 0;
        }
    }

    /// Get cursor position (row, col)
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }
    /// Enter alternate screen buffer
    ///
    /// # Arguments
    /// * `clear` - Whether to clear the alternate screen
    /// * `save_cursor` - Whether to save and reset cursor position to 0,0
    pub fn enter_alternate_screen(&mut self, clear: bool, save_cursor: bool) {
        self.switch_to_alternate_screen(clear, save_cursor);
    }

    /// Leave alternate screen buffer and restore main screen
    ///
    /// # Arguments
    /// * `restore_cursor` - Whether to restore cursor position (always true for full restore)
    pub fn leave_alternate_screen(&mut self, restore_cursor: bool) {
        if restore_cursor {
            self.switch_to_normal_screen();
        }
    }

    /// Check if alternate screen is currently active
    pub fn is_alternate_screen(&self) -> bool {
        self.use_alternate_screen
    }

    /// Get current screen buffer (reference)
    pub fn current_buffer(&self) -> &Vec<Vec<Cell>> {
        &self.buffer
    }


    /// Get terminal dimensions (cols, rows)
    pub fn dimensions(&self) -> (usize, usize) {
        (self.cols, self.rows)
    }

    /// Get scrollback buffer size
    pub fn scrollback_size(&self) -> usize {
        self.scrollback.len()
    }

    /// Get compression statistics for scrollback buffer
    pub fn compression_stats(&self) -> &CompressionStats {
        &self.compression_stats
    }

    /// Get dirty tracker (for rendering optimization)
    pub fn dirty_tracker(&self) -> &DirtyTracker {
        &self.dirty_tracker
    }

    /// Get mutable dirty tracker (for rendering optimization)
    pub fn dirty_tracker_mut(&mut self) -> &mut DirtyTracker {
        &mut self.dirty_tracker
    }

    /// Clear dirty flags after rendering
    pub fn clear_dirty(&mut self) {
        self.dirty_tracker.clear();
    }

    /// Get window title (OSC 0 or OSC 2) - reserved for future window title bar integration
    #[allow(dead_code)]
    pub fn window_title(&self) -> Option<&str> {
        self.window_title.as_deref()
    }

    /// Get icon name (OSC 1) - reserved for future icon integration
    #[allow(dead_code)]
    pub fn icon_name(&self) -> Option<&str> {
        self.icon_name.as_deref()
    }

    /// Get current working directory from shell (OSC 7) - reserved for future tab label enhancement
    #[allow(dead_code)]
    pub fn cwd_from_shell(&self) -> Option<&str> {
        self.cwd_from_shell.as_deref()
    }

    /// Get clipboard request data (OSC 52) - reserved for future clipboard integration
    #[allow(dead_code)]
    pub fn clipboard_request(&self) -> Option<&str> {
        self.clipboard_request.as_deref()
    }

    /// Clear clipboard request after it has been read - reserved for future clipboard integration
    #[allow(dead_code)]
    pub fn clear_clipboard_request(&mut self) {
        self.clipboard_request = None;
    }

    /// Take clipboard request and clear it (consume)
    pub fn take_clipboard_request(&mut self) -> Option<String> {
        self.clipboard_request.take()
    }

    /// Parse file:// URI to extract path
    fn parse_file_uri(&self, uri: &str) -> Option<String> {
        if let Some(path_part) = uri.strip_prefix("file://") {
            // Handle both file://hostname/path and file:///path
            if let Some(slash_pos) = path_part.find('/') {
                // file://hostname/path -> extract /path
                Some(path_part[slash_pos..].to_string())
            } else if path_part.is_empty() {
                // file:/// -> next part is the path (malformed, but handle it)
                None
            } else {
                // file://path (no hostname) -> use as is
                Some(format!("/{path_part}"))
            }
        } else if uri.starts_with("file:/") {
            // file:/path (missing one slash)
            Some(uri.strip_prefix("file:").unwrap().to_string())
        } else {
            None
        }
    }

    /// Parse RGB color specification
    /// Format: "rgb:RR/GG/BB" or "rgb:RRRR/GGGG/BBBB" (hex values)
    fn parse_rgb_color(&self, color_spec: &str) -> Option<(u8, u8, u8)> {
        if let Some(rgb_part) = color_spec.strip_prefix("rgb:") {
            let parts: Vec<&str> = rgb_part.split('/').collect();
            if parts.len() == 3 {
                // Parse hex values - handle both 2-digit (RR) and 4-digit (RRRR) formats
                let r = u8::from_str_radix(parts[0].get(0..2)?, 16).ok()?;
                let g = u8::from_str_radix(parts[1].get(0..2)?, 16).ok()?;
                let b = u8::from_str_radix(parts[2].get(0..2)?, 16).ok()?;
                return Some((r, g, b));
            }
        }
        None
    }

    /// Get current mouse reporting mode - reserved for future mouse interaction support
    #[allow(dead_code)]
    pub fn mouse_mode(&self) -> MouseMode {
        self.mouse_mode
    }

    /// Get current mouse encoding mode - reserved for future mouse interaction support
    #[allow(dead_code)]
    pub fn mouse_encoding(&self) -> MouseEncoding {
        self.mouse_encoding
    }

    /// Check if mouse reporting is enabled - reserved for future mouse interaction support
    #[allow(dead_code)]
    pub fn is_mouse_reporting_enabled(&self) -> bool {
        self.mouse_mode != MouseMode::None
    }

    /// Get cursor visibility state
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Check if cursor blink is enabled
    pub fn cursor_blink_enabled(&self) -> bool {
        self.cursor_blink_enabled
    }

    /// Get the color palette entry
    pub fn get_palette_color(&self, index: u8) -> Option<(u8, u8, u8)> {
        self.color_palette.get(index as usize).copied()
    }

    /// Get default foreground color
    pub fn default_fg_color(&self) -> Option<(u8, u8, u8)> {
        self.default_fg_color
    }

    /// Get default background color
    pub fn default_bg_color(&self) -> Option<(u8, u8, u8)> {
        self.default_bg_color
    }

    /// Check if bell was triggered and clear the flag (consume)
    pub fn take_bell_triggered(&mut self) -> bool {
        let triggered = self.bell_triggered;
        self.bell_triggered = false;
        triggered
    }

    /// Get auto-wrap mode state - reserved for future line wrapping logic enhancement
    #[allow(dead_code)]
    pub fn auto_wrap_mode(&self) -> bool {
        self.auto_wrap_mode
    }

    /// Get bracketed paste mode state - reserved for future paste handling
    #[allow(dead_code)]
    pub fn bracketed_paste_mode(&self) -> bool {
        self.bracketed_paste_mode
    }

    /// Get application cursor keys mode state (DECCKM)
    pub fn application_cursor_keys(&self) -> bool {
        self.application_cursor_keys
    }

    /// Take pending responses (for sending to PTY)
    /// This drains the pending_responses vec and returns it
    pub fn take_pending_responses(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_responses)
    }

    /// Get memory usage statistics for the terminal screen
    pub fn memory_stats(&self) -> MemoryStats {
        let buffer_bytes = self
            .buffer
            .iter()
            .map(|line| memory::line_memory_size(line))
            .sum();

        let scrollback_bytes: usize = self
            .scrollback
            .iter()
            .map(|compressed| compressed.compressed_size)
            .sum();

        let interner_bytes = self.string_interner.memory_usage();
        let (interned_strings, interner_hits, interner_misses) = self.string_interner.stats();

        let total_bytes = buffer_bytes + scrollback_bytes + interner_bytes;

        MemoryStats {
            buffer_lines: self.buffer.len(),
            scrollback_lines: self.scrollback.len(),
            buffer_bytes,
            scrollback_bytes,
            interner_bytes,
            interned_strings,
            interner_hits,
            interner_misses,
            total_bytes,
        }
    }

    /// Get a human-readable string of memory usage
    pub fn memory_usage_string(&self) -> String {
        self.memory_stats().to_string()
    }

    /// Manually trigger string interner cleanup
    pub fn cleanup_interner(&mut self) {
        self.string_interner.cleanup();
    }

    /// Scroll screen up by n lines
    fn scroll_up(&mut self, n: usize) {
        let (top, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));

        for _ in 0..n {
            // Save top line to scrollback (compressed)
            if top == 0 {
                let compressed = CompressedLine::compress(&self.buffer[top]);
                self.compression_stats.update(&compressed);
                self.scrollback.push_back(compressed);
                if self.scrollback.len() > MAX_SCROLLBACK {
                    self.scrollback.pop_front();
                }
            }

            // Shift lines up within scroll region
            for row in top..bottom {
                self.buffer[row] = self.buffer[row + 1].clone();
            }

            // Clear bottom line
            self.buffer[bottom] = vec![Cell::default(); self.cols];
        }

        // Mark the affected region as dirty
        self.dirty_tracker.mark_range(top, bottom);
    }

    /// Scroll screen down by n lines
    fn scroll_down(&mut self, n: usize) {
        let (top, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));

        for _ in 0..n {
            // Shift lines down within scroll region
            for row in (top + 1..=bottom).rev() {
                self.buffer[row] = self.buffer[row - 1].clone();
            }

            // Clear top line
            self.buffer[top] = vec![Cell::default(); self.cols];
        }

        // Mark the affected region as dirty
        self.dirty_tracker.mark_range(top, bottom);
    }

    /// Insert n blank lines at cursor position (IL)
    /// Lines below cursor are pushed down within scroll region
    fn insert_lines(&mut self, n: usize) {
        let (top, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));
        let cursor_row = self.cursor_row;

        // IL only works within the scroll region
        if cursor_row < top || cursor_row > bottom {
            return;
        }

        let n = min(n, bottom - cursor_row + 1);

        // Move lines down within scroll region
        for _ in 0..n {
            // Shift lines down from cursor to bottom
            for row in ((cursor_row + 1)..=bottom).rev() {
                self.buffer[row] = self.buffer[row - 1].clone();
            }

            // Insert blank line at cursor position
            self.buffer[cursor_row] = vec![Cell::default(); self.cols];
        }
    }

    /// Delete n lines at cursor position (DL)
    /// Lines below cursor are pulled up within scroll region
    fn delete_lines(&mut self, n: usize) {
        let (top, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));
        let cursor_row = self.cursor_row;

        // DL only works within the scroll region
        if cursor_row < top || cursor_row > bottom {
            return;
        }

        let n = min(n, bottom - cursor_row + 1);

        // Move lines up within scroll region
        for _ in 0..n {
            // Shift lines up from cursor to bottom
            for row in cursor_row..bottom {
                self.buffer[row] = self.buffer[row + 1].clone();
            }

            // Clear the bottom line
            self.buffer[bottom] = vec![Cell::default(); self.cols];
        }
    }

    /// Move cursor to next line (with scrolling if needed)
    fn new_line(&mut self) {
        let (_, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));

        if self.cursor_row == bottom {
            self.scroll_up(1);
        } else if self.cursor_row < self.rows - 1 {
            self.cursor_row += 1;
        }
    }

    /// Carriage return
    fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    /// Backspace
    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    /// Tab (move to next tab stop, assume every 8 columns)
    fn tab(&mut self) {
        let next_tab = ((self.cursor_col / 8) + 1) * 8;
        self.cursor_col = min(next_tab, self.cols - 1);
    }

    /// Erase in display (ED)
    fn erase_in_display(&mut self, mode: u16) {
        match mode {
            0 => {
                // Clear from cursor to end of screen
                // Clear current line from cursor to end
                for col in self.cursor_col..self.cols {
                    self.buffer[self.cursor_row][col] = Cell::default();
                }
                // Clear all lines below
                for row in (self.cursor_row + 1)..self.rows {
                    self.buffer[row] = vec![Cell::default(); self.cols];
                }
            }
            1 => {
                // Clear from beginning to cursor
                // Clear all lines above
                for row in 0..self.cursor_row {
                    self.buffer[row] = vec![Cell::default(); self.cols];
                }
                // Clear current line from beginning to cursor
                for col in 0..=self.cursor_col {
                    self.buffer[self.cursor_row][col] = Cell::default();
                }
            }
            2 | 3 => {
                // Clear entire screen (3 also clears scrollback)
                for row in 0..self.rows {
                    self.buffer[row] = vec![Cell::default(); self.cols];
                }
                if mode == 3 {
                    self.scrollback.clear();
                }
            }
            _ => {}
        }
    }

    /// Erase in line (EL)
    fn erase_in_line(&mut self, mode: u16) {
        match mode {
            0 => {
                // Clear from cursor to end of line
                for col in self.cursor_col..self.cols {
                    self.buffer[self.cursor_row][col] = Cell::default();
                }
            }
            1 => {
                // Clear from beginning to cursor
                for col in 0..=self.cursor_col {
                    self.buffer[self.cursor_row][col] = Cell::default();
                }
            }
            2 => {
                // Clear entire line
                self.buffer[self.cursor_row] = vec![Cell::default(); self.cols];
            }
            _ => {}
        }
    }

    /// Insert characters (ICH) - CSI n @
    /// Insert n blank characters at cursor position, shifting existing characters to the right
    fn insert_characters(&mut self, n: usize) {
        if self.cursor_row >= self.rows || self.cursor_col >= self.cols {
            return;
        }

        let n = max(1, min(n, self.cols - self.cursor_col));
        let row = self.cursor_row;
        let col = self.cursor_col;

        // Shift existing characters to the right
        for i in (col..(self.cols - n)).rev() {
            self.buffer[row][i + n] = self.buffer[row][i].clone();
        }

        // Insert blank characters at cursor position
        for i in col..(col + n) {
            self.buffer[row][i] = Cell::default();
        }
    }

    /// Delete characters (DCH) - CSI n P
    /// Delete n characters at cursor position, shifting remaining characters to the left
    fn delete_characters(&mut self, n: usize) {
        if self.cursor_row >= self.rows || self.cursor_col >= self.cols {
            return;
        }

        let n = max(1, min(n, self.cols - self.cursor_col));
        let row = self.cursor_row;
        let col = self.cursor_col;

        // Shift remaining characters to the left
        for i in col..(self.cols - n) {
            self.buffer[row][i] = self.buffer[row][i + n].clone();
        }

        // Fill the end of the line with blank characters
        for i in (self.cols - n)..self.cols {
            self.buffer[row][i] = Cell::default();
        }
    }

    /// Switch to alternate screen buffer (CSI ?47h, ?1047h, ?1049h).
    ///
    /// Saves complete main screen state including:
    /// - Buffer contents and scrollback
    /// - Cursor position and attributes
    /// - Scroll region
    /// - DECSC saved cursor state
    ///
    /// The `clear_screen` parameter determines if the alternate screen should be cleared.
    /// The `save_cursor` parameter determines if cursor should be reset to 0,0 (true for ?1049h only).
    fn switch_to_alternate_screen(&mut self, clear_screen: bool, save_cursor: bool) {
        if !self.use_alternate_screen {
            // Save complete main screen state
            self.alternate_buffer = Some(self.buffer.clone());
            self.alternate_scrollback = Some(self.scrollback.clone());
            self.alternate_saved_state = Some(AlternateScreenState {
                main_buffer: self.buffer.clone(),
                cursor_pos: (self.cursor_row, self.cursor_col),
                current_fg: self.current_fg,
                current_bg: self.current_bg,
                bold: self.bold,
                underline: self.underline,
                reverse: self.reverse,
                dim: self.dim,
                italic: self.italic,
                strikethrough: self.strikethrough,
                scroll_region: self.scroll_region,
                saved_cursor: self.saved_cursor,
                saved_cursor_state: self.saved_cursor_state,
                saved_scrollback: self.scrollback.clone(),
                is_active: true,
                scrollback_disabled: true,
            });

            // Create alternate screen buffer
            if clear_screen {
                // Clear alternate screen (CSI ?1047h, ?1049h)
                self.buffer = vec![vec![Cell::default(); self.cols]; self.rows];
            } else {
                // Keep current content (CSI ?47h - simple buffer switch)
                // This is rarely used, but some apps might rely on it
                self.buffer = vec![vec![Cell::default(); self.cols]; self.rows];
            }

            self.scrollback = VecDeque::new();

            // Reset cursor to 0,0 only for ?1049h mode (save_cursor=true)
            if save_cursor {
                self.cursor_row = 0;
                self.cursor_col = 0;
            }
            self.scroll_region = None;
            self.saved_cursor = None;
            self.saved_cursor_state = None;

            // Reset text attributes to defaults in alternate screen
            self.current_fg = None;
            self.current_bg = None;
            self.bold = false;
            self.underline = false;
            self.reverse = false;
            self.dim = false;
            self.italic = false;
            self.strikethrough = false;

            self.use_alternate_screen = true;
        }
    }

    /// Switch back to normal screen buffer (CSI ?47l, ?1047l, ?1049l).
    ///
    /// Fully restores main screen state including:
    /// - Buffer contents and scrollback
    /// - Cursor position and attributes
    /// - Scroll region
    /// - DECSC saved cursor state
    fn switch_to_normal_screen(&mut self) {
        if self.use_alternate_screen {
            // Restore buffer and scrollback
            if let Some(saved_buffer) = self.alternate_buffer.take() {
                self.buffer = saved_buffer;
            }
            if let Some(saved_scrollback) = self.alternate_scrollback.take() {
                self.scrollback = saved_scrollback;
            }

            // Restore complete terminal state
            if let Some(state) = self.alternate_saved_state.take() {
                self.cursor_row = min(state.cursor_pos.0, self.rows.saturating_sub(1));
                self.cursor_col = min(state.cursor_pos.1, self.cols.saturating_sub(1));
                self.current_fg = state.current_fg;
                self.current_bg = state.current_bg;
                self.bold = state.bold;
                self.underline = state.underline;
                self.reverse = state.reverse;
                self.dim = state.dim;
                self.italic = state.italic;
                self.strikethrough = state.strikethrough;
                self.scroll_region = state.scroll_region;
                self.saved_cursor = state.saved_cursor;
                self.saved_cursor_state = state.saved_cursor_state;
            }

            self.use_alternate_screen = false;
        }
    }

    /// Set graphics rendition (SGR)
    fn set_sgr(&mut self, params: &Params) {
        let mut iter = params.iter();

        while let Some(param) = iter.next() {
            let value = param[0];
            match value {
                0 => {
                    // Reset all attributes
                    self.current_fg = None;
                    self.current_bg = None;
                    self.bold = false;
                    self.underline = false;
                    self.reverse = false;
                    self.dim = false;
                    self.italic = false;
                    self.strikethrough = false;
                }
                1 => self.bold = true,
                2 => self.dim = true,
                3 => self.italic = true,
                4 => self.underline = true,
                7 => self.reverse = true,
                9 => self.strikethrough = true,
                22 => {
                    // Normal intensity (not bold and not dim)
                    self.bold = false;
                    self.dim = false;
                }
                23 => self.italic = false,
                24 => self.underline = false,
                27 => self.reverse = false,
                29 => self.strikethrough = false,
                // Foreground colors (30-37, 90-97)
                30..=37 => self.current_fg = Some(AnsiColor::Indexed((value - 30) as u8)),
                38 => {
                    // Extended foreground color
                    if let Some(next) = iter.next() {
                        match next[0] {
                            2 => {
                                // RGB
                                if let (Some(r), Some(g), Some(b)) =
                                    (iter.next(), iter.next(), iter.next())
                                {
                                    self.current_fg =
                                        Some(AnsiColor::Rgb(r[0] as u8, g[0] as u8, b[0] as u8));
                                }
                            }
                            5 => {
                                // 256-color palette
                                if let Some(idx) = iter.next() {
                                    self.current_fg = Some(AnsiColor::Palette256(idx[0] as u8));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                39 => self.current_fg = None, // Default foreground
                // Background colors (40-47, 100-107)
                40..=47 => self.current_bg = Some(AnsiColor::Indexed((value - 40) as u8)),
                48 => {
                    // Extended background color
                    if let Some(next) = iter.next() {
                        match next[0] {
                            2 => {
                                // RGB
                                if let (Some(r), Some(g), Some(b)) =
                                    (iter.next(), iter.next(), iter.next())
                                {
                                    self.current_bg =
                                        Some(AnsiColor::Rgb(r[0] as u8, g[0] as u8, b[0] as u8));
                                }
                            }
                            5 => {
                                // 256-color palette
                                if let Some(idx) = iter.next() {
                                    self.current_bg = Some(AnsiColor::Palette256(idx[0] as u8));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                49 => self.current_bg = None, // Default background
                // Bright foreground colors (90-97)
                90..=97 => self.current_fg = Some(AnsiColor::Indexed((value - 90 + 8) as u8)),
                // Bright background colors (100-107)
                100..=107 => self.current_bg = Some(AnsiColor::Indexed((value - 100 + 8) as u8)),
                _ => {}
            }
        }
    }
}

impl Perform for TerminalScreen {
    fn print(&mut self, c: char) {
        // Get character width (1 for regular, 2 for wide characters like CJK)
        let width = UnicodeWidthChar::width(c).unwrap_or(1);

        // Handle line wrapping based on auto_wrap_mode
        // For wide characters, check if we have enough space (need 2 columns)
        if self.cursor_col >= self.cols || (width == 2 && self.cursor_col >= self.cols - 1) {
            if self.auto_wrap_mode {
                // Auto-wrap enabled: move to next line
                self.cursor_col = 0;
                self.new_line();
            } else {
                // Auto-wrap disabled: stay at last column (overwrite)
                self.cursor_col = self.cols - 1;
                // For wide characters at the edge, skip them
                if width == 2 {
                    return;
                }
            }
        }

        // Write character
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            // If this is overwriting a wide character placeholder, clear the wide char too
            if self.buffer[self.cursor_row][self.cursor_col].placeholder && self.cursor_col > 0 {
                self.buffer[self.cursor_row][self.cursor_col - 1] = Cell::default();
            }

            // If this is overwriting a wide character, clear its placeholder too
            if self.buffer[self.cursor_row][self.cursor_col].wide && self.cursor_col + 1 < self.cols
            {
                self.buffer[self.cursor_row][self.cursor_col + 1] = Cell::default();
            }

            if width == 2 {
                // Wide character: write to first cell and placeholder to second cell
                self.buffer[self.cursor_row][self.cursor_col] = Cell {
                    c,
                    fg: self.current_fg,
                    bg: self.current_bg,
                    bold: self.bold,
                    underline: self.underline,
                    reverse: self.reverse,
                    dim: self.dim,
                    italic: self.italic,
                    strikethrough: self.strikethrough,
                    wide: true,
                    placeholder: false,
                    hyperlink: None,
                    image: None,
                };

                // Write placeholder to next cell if there's space
                if self.cursor_col + 1 < self.cols {
                    self.buffer[self.cursor_row][self.cursor_col + 1] = Cell {
                        c: ' ', // Placeholder character
                        fg: self.current_fg,
                        bg: self.current_bg,
                        bold: self.bold,
                        underline: self.underline,
                        reverse: self.reverse,
                        dim: self.dim,
                        italic: self.italic,
                        strikethrough: self.strikethrough,
                        wide: false,
                        placeholder: true,
                        hyperlink: None,
                        image: None,
                    };
                }

                self.cursor_col += 2;
            } else {
                // Regular character: write normally
                self.buffer[self.cursor_row][self.cursor_col] = Cell {
                    c,
                    fg: self.current_fg,
                    bg: self.current_bg,
                    bold: self.bold,
                    underline: self.underline,
                    reverse: self.reverse,
                    dim: self.dim,
                    italic: self.italic,
                    strikethrough: self.strikethrough,
                    wide: false,
                    placeholder: false,
                    hyperlink: None,
                    image: None,
                };
                self.cursor_col += 1;
            }

            // Mark the line as dirty
            self.dirty_tracker.mark_line(self.cursor_row);
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                // Line Feed (LF)
                self.new_line();
            }
            b'\r' => {
                // Carriage Return (CR)
                self.carriage_return();
            }
            b'\x08' => {
                // Backspace (BS)
                self.backspace();
            }
            b'\t' => {
                // Tab
                self.tab();
            }
            0x07 => {
                // Bell (BEL) - set flag for notification
                self.bell_triggered = true;
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // DCS sequences - not implemented yet
    }

    fn put(&mut self, _byte: u8) {
        // DCS data - not implemented yet
    }

    fn unhook(&mut self) {
        // End of DCS - not implemented yet
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // OSC sequences: ESC ] <command> ; <data> ST (or BEL)
        // Parse OSC command number and data
        if params.is_empty() {
            return;
        }

        // First param is the command number
        let command_str = String::from_utf8_lossy(params[0]);
        let command = command_str.parse::<u16>().unwrap_or(0);

        match command {
            0 => {
                // OSC 0 ; title - Set icon name and window title
                if params.len() > 1 {
                    let title = String::from_utf8_lossy(params[1]).to_string();
                    self.window_title = Some(title.clone());
                    self.icon_name = Some(title);
                }
            }
            1 => {
                // OSC 1 ; name - Set icon name
                if params.len() > 1 {
                    self.icon_name = Some(String::from_utf8_lossy(params[1]).to_string());
                }
            }
            2 => {
                // OSC 2 ; title - Set window title
                if params.len() > 1 {
                    self.window_title = Some(String::from_utf8_lossy(params[1]).to_string());
                }
            }
            7 => {
                // OSC 7 ; file://hostname/path - Set current working directory
                if params.len() > 1 {
                    let cwd_uri = String::from_utf8_lossy(params[1]).to_string();
                    // Extract path from file:// URI
                    // Format: file://hostname/path or file:///path
                    if let Some(path) = self.parse_file_uri(&cwd_uri) {
                        self.cwd_from_shell = Some(path);
                    }
                }
            }
            4 => {
                // OSC 4 ; <index> ; <color spec> - Set/query color palette entry
                // Query format: OSC 4 ; <index> ; ? ST
                // Set format: OSC 4 ; <index> ; rgb:<rr>/<gg>/<bb> ST
                if params.len() > 2 {
                    let index_str = String::from_utf8_lossy(params[1]);
                    if let Ok(index) = index_str.parse::<u8>() {
                        let color_spec = String::from_utf8_lossy(params[2]);

                        if color_spec == "?" {
                            // Query color palette
                            if let Some((r, g, b)) = self.color_palette.get(index as usize) {
                                let response = format!(
                                    "\x1b]4;{index};rgb:{r:02x}/{g:02x}/{b:02x}\x07"
                                );
                                self.pending_responses.push(response);
                            }
                        } else if color_spec.starts_with("rgb:") {
                            // Set color palette
                            if let Some(rgb) = self.parse_rgb_color(&color_spec) {
                                if (index as usize) < self.color_palette.len() {
                                    self.color_palette[index as usize] = rgb;
                                }
                            }
                        }
                    }
                }
            }
            10 => {
                // OSC 10 ; <color spec> - Set/query default foreground color
                // Query format: OSC 10 ; ? ST
                // Set format: OSC 10 ; rgb:<rr>/<gg>/<bb> ST
                if params.len() > 1 {
                    let color_spec = String::from_utf8_lossy(params[1]);

                    if color_spec == "?" {
                        // Query default foreground
                        let (r, g, b) = self.default_fg_color.unwrap_or((204, 204, 204));
                        let response = format!("\x1b]10;rgb:{r:02x}/{g:02x}/{b:02x}\x07");
                        self.pending_responses.push(response);
                    } else if color_spec.starts_with("rgb:") {
                        // Set default foreground
                        if let Some(rgb) = self.parse_rgb_color(&color_spec) {
                            self.default_fg_color = Some(rgb);
                        }
                    }
                }
            }
            11 => {
                // OSC 11 ; <color spec> - Set/query default background color
                // Query format: OSC 11 ; ? ST
                // Set format: OSC 11 ; rgb:<rr>/<gg>/<bb> ST
                if params.len() > 1 {
                    let color_spec = String::from_utf8_lossy(params[1]);

                    if color_spec == "?" {
                        // Query default background
                        let (r, g, b) = self.default_bg_color.unwrap_or((0, 0, 0));
                        let response = format!("\x1b]11;rgb:{r:02x}/{g:02x}/{b:02x}\x07");
                        self.pending_responses.push(response);
                    } else if color_spec.starts_with("rgb:") {
                        // Set default background
                        if let Some(rgb) = self.parse_rgb_color(&color_spec) {
                            self.default_bg_color = Some(rgb);
                        }
                    }
                }
            }
            52 => {
                // OSC 52 ; c ; <base64-data> - Clipboard operations
                // c = clipboard selection (usually 'c' for clipboard, 'p' for primary)
                // base64-data = clipboard content in base64
                if params.len() > 2 {
                    let selection = String::from_utf8_lossy(params[1]);
                    let data = String::from_utf8_lossy(params[2]).to_string();

                    if data == "?" {
                        // OSC 52 query - respond with clipboard content
                        // Read current clipboard and send back as base64
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if let Ok(text) = clipboard.get_text() {
                                use base64::{engine::general_purpose::STANDARD, Engine as _};
                                let encoded = STANDARD.encode(text.as_bytes());
                                // Respond with OSC 52 ; c ; <base64> ST
                                let response = format!("\x1b]52;{selection};{encoded}\x1b\\");
                                self.pending_responses.push(response);
                            }
                        }
                    } else if !data.is_empty() {
                        // OSC 52 set - store clipboard data
                        self.clipboard_request = Some(data);
                    }
                }
            }
            _ => {
                // Unknown OSC command - ignore
            }
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        // Check for private mode sequences (CSI ? ...)
        let is_private = intermediates.contains(&b'?');

        if is_private && action == 'h' {
            // Set private mode (DEC Private Mode Set - DECSET)
            for param in params.iter() {
                match param[0] {
                    1 => {
                        // DECCKM - Application Cursor Keys (CSI ?1h)
                        self.application_cursor_keys = true;
                    }
                    9 | 1000 => {
                        // X10 Mouse Reporting (CSI ?9h or CSI ?1000h)
                        self.mouse_mode = MouseMode::X10;
                    }
                    12 => {
                        // AT&T 610 - Start Blinking Cursor (CSI ?12h)
                        self.cursor_blink_enabled = true;
                    }
                    1002 => {
                        // Button-Event Mouse Tracking (CSI ?1002h)
                        self.mouse_mode = MouseMode::ButtonEvent;
                    }
                    1003 => {
                        // Any-Event Mouse Tracking (CSI ?1003h)
                        self.mouse_mode = MouseMode::AnyEvent;
                    }
                    25 => {
                        // DECTCEM - Show Cursor (CSI ?25h)
                        self.cursor_visible = true;
                    }
                    1006 => {
                        // SGR Extended Mouse Mode (CSI ?1006h)
                        self.mouse_encoding = MouseEncoding::Sgr;
                    }
                    47 => {
                        // CSI ?47h - Switch to alternate screen (basic mode)
                        // Used by some older applications
                        self.switch_to_alternate_screen(false, false);
                    }
                    1047 => {
                        // CSI ?1047h - Switch to alternate screen and clear it
                        // More common in modern terminals
                        self.switch_to_alternate_screen(true, false);
                    }
                    1049 => {
                        // CSI ?1049h - Save cursor, switch to alternate screen and clear
                        // Most complete mode - used by vim, less, htop, etc.
                        // The cursor is saved in the state and reset to 0,0 in alternate screen
                        self.switch_to_alternate_screen(true, true);
                    }
                    2004 => {
                        // CSI ?2004h - Enable bracketed paste mode
                        self.bracketed_paste_mode = true;
                    }
                    _ => {
                        // Other private modes not implemented yet
                    }
                }
            }
            return;
        }

        if is_private && action == 'l' {
            // Reset private mode (DEC Private Mode Reset - DECRST)
            for param in params.iter() {
                match param[0] {
                    1 => {
                        // DECCKM - Normal Cursor Keys (CSI ?1l)
                        self.application_cursor_keys = false;
                    }
                    9 | 1000 | 1002 | 1003 => {
                        // Disable mouse reporting (CSI ?9l, ?1000l, ?1002l, ?1003l)
                        self.mouse_mode = MouseMode::None;
                    }
                    12 => {
                        // AT&T 610 - Stop Blinking Cursor (CSI ?12l)
                        self.cursor_blink_enabled = false;
                    }
                    25 => {
                        // DECTCEM - Hide Cursor (CSI ?25l)
                        self.cursor_visible = false;
                    }
                    1006 => {
                        // Disable SGR Extended Mouse Mode (CSI ?1006l)
                        self.mouse_encoding = MouseEncoding::Default;
                    }
                    47 | 1047 | 1049 => {
                        // CSI ?47l, ?1047l, ?1049l - Switch back to normal screen
                        // All three modes restore the main screen completely
                        // The saved state includes cursor position and all attributes
                        self.switch_to_normal_screen();
                    }
                    2004 => {
                        // CSI ?2004l - Disable bracketed paste mode
                        self.bracketed_paste_mode = false;
                    }
                    _ => {
                        // Other private modes not implemented yet
                    }
                }
            }
            return;
        }

        // Standard CSI sequences
        match action {
            'A' => {
                // Cursor Up (CUU)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                let (top, _) = self.scroll_region.unwrap_or((0, self.rows - 1));
                self.cursor_row = max(top, self.cursor_row.saturating_sub(n));
            }
            'B' => {
                // Cursor Down (CUD)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                let (_, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));
                self.cursor_row = min(bottom, self.cursor_row + n);
            }
            'C' => {
                // Cursor Forward (CUF)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.cursor_col = min(self.cols - 1, self.cursor_col + n);
            }
            'D' => {
                // Cursor Back (CUB)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            'H' | 'f' => {
                // Cursor Position (CUP) or Horizontal Vertical Position (HVP)
                let mut iter = params.iter();
                let row = iter
                    .next()
                    .map(|p| p[0].saturating_sub(1) as usize)
                    .unwrap_or(0);
                let col = iter
                    .next()
                    .map(|p| p[0].saturating_sub(1) as usize)
                    .unwrap_or(0);
                self.cursor_row = min(row, self.rows - 1);
                self.cursor_col = min(col, self.cols - 1);
            }
            'J' => {
                // Erase in Display (ED)
                let mode = params.iter().next().map(|p| p[0]).unwrap_or(0);
                self.erase_in_display(mode);
            }
            'K' => {
                // Erase in Line (EL)
                let mode = params.iter().next().map(|p| p[0]).unwrap_or(0);
                self.erase_in_line(mode);
            }
            'm' => {
                // Select Graphic Rendition (SGR)
                self.set_sgr(params);
            }
            'r' => {
                // Set Scrolling Region (DECSTBM)
                let mut iter = params.iter();
                let top = iter
                    .next()
                    .map(|p| p[0].saturating_sub(1) as usize)
                    .unwrap_or(0);
                let bottom = iter
                    .next()
                    .map(|p| p[0].saturating_sub(1) as usize)
                    .unwrap_or(self.rows - 1);
                self.scroll_region = Some((min(top, self.rows - 1), min(bottom, self.rows - 1)));
            }
            's' => {
                // Save Cursor Position (SCOSC)
                self.saved_cursor = Some((self.cursor_row, self.cursor_col));
            }
            'u' => {
                // Restore Cursor Position (SCORC)
                if let Some((row, col)) = self.saved_cursor {
                    self.cursor_row = min(row, self.rows - 1);
                    self.cursor_col = min(col, self.cols - 1);
                }
            }
            'S' => {
                // Scroll Up (SU)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.scroll_up(n);
            }
            'T' => {
                // Scroll Down (SD)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.scroll_down(n);
            }
            'L' => {
                // Insert Lines (IL)
                let n = params
                    .iter()
                    .next()
                    .map(|p| max(1, p[0] as usize))
                    .unwrap_or(1);
                self.insert_lines(n);
            }
            'M' => {
                // Delete Lines (DL)
                let n = params
                    .iter()
                    .next()
                    .map(|p| max(1, p[0] as usize))
                    .unwrap_or(1);
                self.delete_lines(n);
            }
            '@' => {
                // Insert Characters (ICH)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.insert_characters(n);
            }
            'P' => {
                // Delete Characters (DCH)
                let n = params.iter().next().map(|p| p[0] as usize).unwrap_or(1);
                self.delete_characters(n);
            }
            'c' => {
                // Device Attributes (DA)
                if intermediates.contains(&b'>') {
                    // Secondary DA (DA2) - CSI > c
                    // Response: CSI > 0 ; 0 ; 0 c (VT100 compatible)
                    self.pending_responses.push("\x1b[>0;0;0c".to_string());
                } else {
                    // Primary DA (DA1) - CSI c
                    // Response: CSI ? 1 ; 2 c (VT100 with Advanced Video Option)
                    self.pending_responses.push("\x1b[?1;2c".to_string());
                }
            }
            'n' => {
                // Device Status Report (DSR)
                let n = params.iter().next().map(|p| p[0]).unwrap_or(0);
                match n {
                    5 => {
                        // DSR - Device Status Report
                        // Response: CSI 0 n (Terminal OK)
                        self.pending_responses.push("\x1b[0n".to_string());
                    }
                    6 => {
                        // CPR - Cursor Position Report
                        // Response: CSI <row> ; <col> R
                        // Note: VT100 uses 1-based indexing
                        let row = self.cursor_row + 1;
                        let col = self.cursor_col + 1;
                        self.pending_responses
                            .push(format!("\x1b[{row};{col}R"));
                    }
                    _ => {
                        // Unknown DSR request - ignore
                    }
                }
            }
            _ => {
                // Unknown CSI sequence - ignore
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte {
            b'7' => {
                // DECSC - Save Cursor (ESC 7)
                // Save cursor position and attributes
                self.saved_cursor_state = Some((
                    self.cursor_row,
                    self.cursor_col,
                    self.current_fg,
                    self.current_bg,
                    self.bold,
                ));
            }
            b'8' => {
                // DECRC - Restore Cursor (ESC 8)
                // Restore cursor position and attributes
                if let Some((row, col, fg, bg, bold)) = self.saved_cursor_state {
                    self.cursor_row = min(row, self.rows - 1);
                    self.cursor_col = min(col, self.cols - 1);
                    self.current_fg = fg;
                    self.current_bg = bg;
                    self.bold = bold;
                }
            }
            b'M' => {
                // RI - Reverse Index (ESC M)
                // Move cursor up one line, scroll down if at top of scroll region
                let (top, _) = self.scroll_region.unwrap_or((0, self.rows - 1));

                if self.cursor_row == top {
                    // At top of scroll region, scroll down
                    self.scroll_down(1);
                } else if self.cursor_row > 0 {
                    // Not at top, just move up
                    self.cursor_row -= 1;
                }
            }
            b'D' => {
                // IND - Index (ESC D)
                // Move cursor down one line, scroll up if at bottom of scroll region
                let (_, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));

                if self.cursor_row == bottom {
                    // At bottom of scroll region, scroll up
                    self.scroll_up(1);
                } else if self.cursor_row < self.rows - 1 {
                    // Not at bottom, just move down
                    self.cursor_row += 1;
                }
            }
            b'E' => {
                // NEL - Next Line (ESC E)
                // Move to first column of next line
                self.cursor_col = 0;
                let (_, bottom) = self.scroll_region.unwrap_or((0, self.rows - 1));

                if self.cursor_row == bottom {
                    // At bottom of scroll region, scroll up
                    self.scroll_up(1);
                } else if self.cursor_row < self.rows - 1 {
                    // Not at bottom, just move down
                    self.cursor_row += 1;
                }
            }
            b'c' => {
                // RIS - Reset to Initial State (ESC c)
                // Full terminal reset
                self.buffer = vec![vec![Cell::default(); self.cols]; self.rows];
                self.scrollback.clear();
                self.cursor_row = 0;
                self.cursor_col = 0;
                self.current_fg = None;
                self.current_bg = None;
                self.bold = false;
                self.underline = false;
                self.reverse = false;
                self.dim = false;
                self.italic = false;
                self.strikethrough = false;
                self.scroll_region = None;
                self.saved_cursor = None;
                self.saved_cursor_state = None;
                self.cursor_visible = true;
                self.auto_wrap_mode = true;
            }
            _ => {
                // Unknown ESC sequence - ignore
            }
        }
    }
}

// Bracket matching support
impl crate::terminal::bracket::GetChar for TerminalScreen {
    fn get_char(&self, line: usize, col: usize) -> Option<char> {
        // Get character from visible buffer only (not scrollback)
        self.buffer.get(line)?.get(col).map(|cell| cell.c)
    }

    fn line_count(&self) -> usize {
        self.buffer.len()
    }

    fn line_width(&self, line: usize) -> usize {
        self.buffer.get(line).map(|row| row.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_mode_default() {
        let screen = TerminalScreen::new(80, 24);
        assert_eq!(screen.mouse_mode(), MouseMode::None);
        assert_eq!(screen.mouse_encoding(), MouseEncoding::Default);
        assert!(!screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_mode_x10_enable() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?1000h - Enable X10 mouse reporting
        screen.process(b"\x1b[?1000h");
        assert_eq!(screen.mouse_mode(), MouseMode::X10);
        assert!(screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_mode_x10_disable() {
        let mut screen = TerminalScreen::new(80, 24);
        // Enable then disable
        screen.process(b"\x1b[?1000h");
        assert_eq!(screen.mouse_mode(), MouseMode::X10);
        screen.process(b"\x1b[?1000l");
        assert_eq!(screen.mouse_mode(), MouseMode::None);
        assert!(!screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_mode_button_event() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?1002h - Enable button-event tracking
        screen.process(b"\x1b[?1002h");
        assert_eq!(screen.mouse_mode(), MouseMode::ButtonEvent);
        assert!(screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_mode_any_event() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?1003h - Enable any-event tracking
        screen.process(b"\x1b[?1003h");
        assert_eq!(screen.mouse_mode(), MouseMode::AnyEvent);
        assert!(screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_encoding_sgr() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?1006h - Enable SGR extended mouse mode
        screen.process(b"\x1b[?1006h");
        assert_eq!(screen.mouse_encoding(), MouseEncoding::Sgr);
        // Disable
        screen.process(b"\x1b[?1006l");
        assert_eq!(screen.mouse_encoding(), MouseEncoding::Default);
    }

    #[test]
    fn test_mouse_mode_multiple_enables() {
        let mut screen = TerminalScreen::new(80, 24);
        // Enable X10
        screen.process(b"\x1b[?1000h");
        assert_eq!(screen.mouse_mode(), MouseMode::X10);
        // Upgrade to ButtonEvent
        screen.process(b"\x1b[?1002h");
        assert_eq!(screen.mouse_mode(), MouseMode::ButtonEvent);
        // Upgrade to AnyEvent
        screen.process(b"\x1b[?1003h");
        assert_eq!(screen.mouse_mode(), MouseMode::AnyEvent);
    }

    #[test]
    fn test_mouse_mode_with_sgr_encoding() {
        let mut screen = TerminalScreen::new(80, 24);
        // Enable X10 mouse + SGR encoding
        screen.process(b"\x1b[?1000h\x1b[?1006h");
        assert_eq!(screen.mouse_mode(), MouseMode::X10);
        assert_eq!(screen.mouse_encoding(), MouseEncoding::Sgr);
        assert!(screen.is_mouse_reporting_enabled());
    }

    #[test]
    fn test_mouse_mode_csi_9() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?9h - Alternative X10 mouse reporting
        screen.process(b"\x1b[?9h");
        assert_eq!(screen.mouse_mode(), MouseMode::X10);
        screen.process(b"\x1b[?9l");
        assert_eq!(screen.mouse_mode(), MouseMode::None);
    }

    #[test]
    fn test_cursor_visible_default() {
        let screen = TerminalScreen::new(80, 24);
        assert!(screen.cursor_visible());
    }

    #[test]
    fn test_cursor_hide() {
        let mut screen = TerminalScreen::new(80, 24);
        // CSI ?25l - Hide cursor (DECTCEM)
        screen.process(b"\x1b[?25l");
        assert!(!screen.cursor_visible());
    }

    #[test]
    fn test_cursor_show() {
        let mut screen = TerminalScreen::new(80, 24);
        // Hide first
        screen.process(b"\x1b[?25l");
        assert!(!screen.cursor_visible());
        // CSI ?25h - Show cursor (DECTCEM)
        screen.process(b"\x1b[?25h");
        assert!(screen.cursor_visible());
    }

    #[test]
    fn test_cursor_toggle() {
        let mut screen = TerminalScreen::new(80, 24);
        // Default is visible
        assert!(screen.cursor_visible());
        // Hide
        screen.process(b"\x1b[?25l");
        assert!(!screen.cursor_visible());
        // Show
        screen.process(b"\x1b[?25h");
        assert!(screen.cursor_visible());
        // Hide again
        screen.process(b"\x1b[?25l");
        assert!(!screen.cursor_visible());
    }

    #[test]
    fn test_esc_decsc_decrc() {
        let mut screen = TerminalScreen::new(80, 24);

        // Move cursor and set attributes
        screen.process(b"\x1b[10;20H"); // Move to (9, 19)
        screen.process(b"\x1b[1m"); // Set bold
        screen.process(b"\x1b[31m"); // Set red foreground
        screen.process(b"\x1b[42m"); // Set green background

        assert_eq!(screen.cursor_position(), (9, 19));
        assert!(screen.bold);
        assert_eq!(screen.current_fg, Some(AnsiColor::Indexed(1)));
        assert_eq!(screen.current_bg, Some(AnsiColor::Indexed(2)));

        // Save cursor (ESC 7)
        screen.process(b"\x1b7");

        // Move cursor and change attributes
        screen.process(b"\x1b[5;10H"); // Move to (4, 9)
        screen.process(b"\x1b[0m"); // Reset attributes

        assert_eq!(screen.cursor_position(), (4, 9));
        assert!(!screen.bold);
        assert_eq!(screen.current_fg, None);
        assert_eq!(screen.current_bg, None);

        // Restore cursor (ESC 8)
        screen.process(b"\x1b8");

        // Should be back to saved position and attributes
        assert_eq!(screen.cursor_position(), (9, 19));
        assert!(screen.bold);
        assert_eq!(screen.current_fg, Some(AnsiColor::Indexed(1)));
        assert_eq!(screen.current_bg, Some(AnsiColor::Indexed(2)));
    }

    #[test]
    fn test_esc_reverse_index() {
        let mut screen = TerminalScreen::new(10, 5);

        // Fill screen with text
        for i in 0..5 {
            screen.process(format!("Line {}\r\n", i).as_bytes());
        }

        // Move to middle line
        screen.process(b"\x1b[3;1H");
        assert_eq!(screen.cursor_position(), (2, 0));

        // Reverse index (ESC M) - should move up one line
        screen.process(b"\x1bM");
        assert_eq!(screen.cursor_position(), (1, 0));

        // Reverse index again
        screen.process(b"\x1bM");
        assert_eq!(screen.cursor_position(), (0, 0));

        // Reverse index at top - should scroll down
        screen.process(b"\x1bM");
        assert_eq!(screen.cursor_position(), (0, 0)); // Stay at top
                                                      // (scrolling behavior is tested by checking the buffer contents)
    }

    #[test]
    fn test_esc_index() {
        let mut screen = TerminalScreen::new(10, 5);

        // Fill screen with text
        for i in 0..5 {
            screen.process(format!("Line {}\r\n", i).as_bytes());
        }

        // Move to second line
        screen.process(b"\x1b[2;1H");
        assert_eq!(screen.cursor_position(), (1, 0));

        // Index (ESC D) - should move down one line
        screen.process(b"\x1bD");
        assert_eq!(screen.cursor_position(), (2, 0));

        // Move to bottom line
        screen.process(b"\x1b[5;1H");
        assert_eq!(screen.cursor_position(), (4, 0));

        // Index at bottom - should scroll up
        screen.process(b"\x1bD");
        assert_eq!(screen.cursor_position(), (4, 0)); // Stay at bottom
    }

    #[test]
    fn test_esc_next_line() {
        let mut screen = TerminalScreen::new(10, 5);

        // Move to position
        screen.process(b"\x1b[2;5H");
        assert_eq!(screen.cursor_position(), (1, 4));

        // Next line (ESC E) - should move to next line, column 0
        screen.process(b"\x1bE");
        assert_eq!(screen.cursor_position(), (2, 0));

        // Next line again
        screen.process(b"\x1bE");
        assert_eq!(screen.cursor_position(), (3, 0));

        // Move to bottom
        screen.process(b"\x1b[5;10H");
        assert_eq!(screen.cursor_position(), (4, 9));

        // Next line at bottom - should scroll and stay at column 0
        screen.process(b"\x1bE");
        assert_eq!(screen.cursor_position(), (4, 0));
    }

    #[test]
    fn test_esc_reset() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set up screen with various state
        screen.process(b"Some text\r\n");
        screen.process(b"\x1b[10;20H"); // Move cursor
        screen.process(b"\x1b[1;4;31;42m"); // Bold, underline, red fg, green bg
        screen.process(b"\x1b[5;10r"); // Set scroll region
        screen.process(b"\x1b[s"); // Save cursor
        screen.process(b"\x1b[?25l"); // Hide cursor

        assert_eq!(screen.cursor_position(), (9, 19));
        assert!(screen.bold);
        assert!(screen.underline);
        assert!(!screen.cursor_visible);

        // Reset (ESC c)
        screen.process(b"\x1bc");

        // Everything should be reset
        assert_eq!(screen.cursor_position(), (0, 0));
        assert!(!screen.bold);
        assert!(!screen.underline);
        assert!(!screen.reverse);
        assert_eq!(screen.current_fg, None);
        assert_eq!(screen.current_bg, None);
        assert!(screen.cursor_visible);
        assert_eq!(screen.scroll_region, None);
        assert_eq!(screen.saved_cursor, None);
        assert_eq!(screen.saved_cursor_state, None);

        // Buffer should be cleared
        let buffer = &screen.buffer;
        for row in buffer.iter() {
            for cell in row.iter() {
                assert_eq!(cell.c, ' ');
            }
        }
    }

    #[test]
    fn test_esc_sequences_with_scroll_region() {
        let mut screen = TerminalScreen::new(10, 10);

        // Fill with text
        for i in 0..10 {
            screen.process(format!("Line {}\r\n", i).as_bytes());
        }

        // Set scroll region to lines 3-7 (indices 2-6)
        screen.process(b"\x1b[3;7r");

        // Move to top of scroll region
        screen.process(b"\x1b[3;1H");
        assert_eq!(screen.cursor_position(), (2, 0));

        // Reverse index at top of scroll region - should scroll down within region
        screen.process(b"\x1bM");
        assert_eq!(screen.cursor_position(), (2, 0));

        // Move to bottom of scroll region
        screen.process(b"\x1b[7;1H");
        assert_eq!(screen.cursor_position(), (6, 0));

        // Index at bottom of scroll region - should scroll up within region
        screen.process(b"\x1bD");
        assert_eq!(screen.cursor_position(), (6, 0));
    }

    #[test]
    fn test_insert_lines_basic() {
        let mut screen = TerminalScreen::new(10, 5);
        // Write lines of text
        screen.process(b"Line 0\r\nLine 1\r\nLine 2\r\nLine 3\r\nLine 4");

        // Move cursor to line 1 (second line)
        screen.process(b"\x1b[2;1H");
        assert_eq!(screen.cursor_position(), (1, 0));

        // Insert 1 line at cursor position (CSI L)
        screen.process(b"\x1b[L");

        // Line 1 should now be blank, and old Line 1 should be at Line 2
        let buffer = &screen.buffer;
        assert_eq!(buffer[0][0].c, 'L'); // Line 0 unchanged
        assert_eq!(buffer[1][0].c, ' '); // New blank line
        assert_eq!(buffer[2][0].c, 'L'); // Old Line 1
        assert_eq!(buffer[3][0].c, 'L'); // Old Line 2
        assert_eq!(buffer[4][0].c, 'L'); // Old Line 3
    }

    #[test]
    fn test_insert_lines_multiple() {
        let mut screen = TerminalScreen::new(10, 5);
        // Write lines of text
        screen.process(b"Line 0\r\nLine 1\r\nLine 2\r\nLine 3\r\nLine 4");

        // Move cursor to line 1
        screen.process(b"\x1b[2;1H");

        // Insert 2 lines (CSI 2L)
        screen.process(b"\x1b[2L");

        let buffer = &screen.buffer;
        assert_eq!(buffer[0][0].c, 'L'); // Line 0 unchanged
        assert_eq!(buffer[1][0].c, ' '); // New blank line
        assert_eq!(buffer[2][0].c, ' '); // New blank line
        assert_eq!(buffer[3][0].c, 'L'); // Old Line 1
        assert_eq!(buffer[4][0].c, 'L'); // Old Line 2
    }

    #[test]
    fn test_delete_lines_basic() {
        let mut screen = TerminalScreen::new(10, 5);
        // Write lines of text
        screen.process(b"Line 0\r\nLine 1\r\nLine 2\r\nLine 3\r\nLine 4");

        // Move cursor to line 1 (second line)
        screen.process(b"\x1b[2;1H");
        assert_eq!(screen.cursor_position(), (1, 0));

        // Delete 1 line at cursor position (CSI M)
        screen.process(b"\x1b[M");

        // Line 1 should be deleted, and lines below should move up
        let buffer = &screen.buffer;
        assert_eq!(buffer[0][0].c, 'L'); // Line 0 unchanged
        assert_eq!(buffer[1][0].c, 'L'); // Old Line 2 moved up
        assert_eq!(buffer[1][5].c, '2');
        assert_eq!(buffer[2][0].c, 'L'); // Old Line 3
        assert_eq!(buffer[2][5].c, '3');
        assert_eq!(buffer[3][0].c, 'L'); // Old Line 4
        assert_eq!(buffer[3][5].c, '4');
        assert_eq!(buffer[4][0].c, ' '); // Last line is now blank
    }

    #[test]
    fn test_delete_lines_multiple() {
        let mut screen = TerminalScreen::new(10, 5);
        // Write lines of text
        screen.process(b"Line 0\r\nLine 1\r\nLine 2\r\nLine 3\r\nLine 4");

        // Move cursor to line 1
        screen.process(b"\x1b[2;1H");

        // Delete 2 lines (CSI 2M)
        screen.process(b"\x1b[2M");

        let buffer = &screen.buffer;
        assert_eq!(buffer[0][0].c, 'L'); // Line 0 unchanged
        assert_eq!(buffer[1][0].c, 'L'); // Old Line 3 moved up
        assert_eq!(buffer[1][5].c, '3');
        assert_eq!(buffer[2][0].c, 'L'); // Old Line 4
        assert_eq!(buffer[2][5].c, '4');
        assert_eq!(buffer[3][0].c, ' '); // Blank line
        assert_eq!(buffer[4][0].c, ' '); // Blank line
    }
}

#[test]
fn test_application_cursor_keys_default() {
    let screen = TerminalScreen::new(80, 24);
    assert!(!screen.application_cursor_keys());
}

#[test]
fn test_application_cursor_keys_enable() {
    let mut screen = TerminalScreen::new(80, 24);
    // CSI ?1h - Enable application cursor keys (DECCKM)
    screen.process(b"\x1b[?1h");
    assert!(screen.application_cursor_keys());
}

#[test]
fn test_application_cursor_keys_disable() {
    let mut screen = TerminalScreen::new(80, 24);
    // Enable first
    screen.process(b"\x1b[?1h");
    assert!(screen.application_cursor_keys());
    // CSI ?1l - Disable application cursor keys
    screen.process(b"\x1b[?1l");
    assert!(!screen.application_cursor_keys());
}

#[test]
fn test_application_cursor_keys_toggle() {
    let mut screen = TerminalScreen::new(80, 24);
    // Default is false (normal mode)
    assert!(!screen.application_cursor_keys());
    // Enable
    screen.process(b"\x1b[?1h");
    assert!(screen.application_cursor_keys());
    // Disable
    screen.process(b"\x1b[?1l");
    assert!(!screen.application_cursor_keys());
    // Enable again
    screen.process(b"\x1b[?1h");
    assert!(screen.application_cursor_keys());
}

#[test]
fn test_wide_character_korean() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write Korean character (한) which is a wide character
    screen.process("한".as_bytes());

    let buffer = &screen.buffer;
    // First cell should contain the character and be marked as wide
    assert_eq!(buffer[0][0].c, '한');
    assert!(buffer[0][0].wide);
    assert!(!buffer[0][0].placeholder);

    // Second cell should be a placeholder
    assert_eq!(buffer[0][1].c, ' ');
    assert!(!buffer[0][1].wide);
    assert!(buffer[0][1].placeholder);

    // Cursor should have advanced by 2
    assert_eq!(screen.cursor_position(), (0, 2));
}

#[test]
fn test_wide_character_chinese() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write Chinese character (中) which is a wide character
    screen.process("中".as_bytes());

    let buffer = &screen.buffer;
    assert_eq!(buffer[0][0].c, '中');
    assert!(buffer[0][0].wide);
    assert!(buffer[0][1].placeholder);
    assert_eq!(screen.cursor_position(), (0, 2));
}

#[test]
fn test_wide_character_japanese() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write Japanese character (日) which is a wide character
    screen.process("日".as_bytes());

    let buffer = &screen.buffer;
    assert_eq!(buffer[0][0].c, '日');
    assert!(buffer[0][0].wide);
    assert!(buffer[0][1].placeholder);
    assert_eq!(screen.cursor_position(), (0, 2));
}

#[test]
fn test_mixed_ascii_and_wide() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write mixed ASCII and Korean: "A한B"
    screen.process("A한B".as_bytes());

    let buffer = &screen.buffer;
    // A at position 0
    assert_eq!(buffer[0][0].c, 'A');
    assert!(!buffer[0][0].wide);
    assert!(!buffer[0][0].placeholder);

    // 한 at positions 1-2
    assert_eq!(buffer[0][1].c, '한');
    assert!(buffer[0][1].wide);
    assert!(!buffer[0][1].placeholder);
    assert!(buffer[0][2].placeholder);

    // B at position 3
    assert_eq!(buffer[0][3].c, 'B');
    assert!(!buffer[0][3].wide);
    assert!(!buffer[0][3].placeholder);

    assert_eq!(screen.cursor_position(), (0, 4));
}

#[test]
fn test_wide_character_with_colors() {
    let mut screen = TerminalScreen::new(10, 3);

    // Set red foreground and write Korean character
    screen.process(b"\x1b[31m");
    screen.process("한".as_bytes());

    let buffer = &screen.buffer;
    // Check both cells have the same foreground color
    assert_eq!(buffer[0][0].fg, Some(AnsiColor::Indexed(1)));
    assert_eq!(buffer[0][1].fg, Some(AnsiColor::Indexed(1)));
    assert!(buffer[0][0].wide);
    assert!(buffer[0][1].placeholder);
}

#[test]
fn test_wide_character_wrapping() {
    let mut screen = TerminalScreen::new(5, 3);

    // Write "ABC한" - the wide character should fit on the first line
    // at positions 3-4 (exactly 2 columns available)
    screen.process("ABC한".as_bytes());

    let buffer = &screen.buffer;
    // First line: "ABC한" (한 occupies positions 3-4)
    assert_eq!(buffer[0][0].c, 'A');
    assert_eq!(buffer[0][1].c, 'B');
    assert_eq!(buffer[0][2].c, 'C');
    assert_eq!(buffer[0][3].c, '한');
    assert!(buffer[0][3].wide);
    assert!(buffer[0][4].placeholder);

    // Cursor should be at the end of the first line
    assert_eq!(screen.cursor_position(), (0, 5));

    // Now test actual wrapping: write "ABCD한" where 한 should wrap
    let mut screen2 = TerminalScreen::new(5, 3);
    screen2.process("ABCD한".as_bytes());

    let buffer2 = &screen2.buffer;
    // First line: "ABCD"
    assert_eq!(buffer2[0][0].c, 'A');
    assert_eq!(buffer2[0][1].c, 'B');
    assert_eq!(buffer2[0][2].c, 'C');
    assert_eq!(buffer2[0][3].c, 'D');

    // Wide character should wrap to next line (only 1 column left)
    assert_eq!(buffer2[1][0].c, '한');
    assert!(buffer2[1][0].wide);
    assert!(buffer2[1][1].placeholder);

    assert_eq!(screen2.cursor_position(), (1, 2));
}

#[test]
fn test_wide_character_at_edge() {
    let mut screen = TerminalScreen::new(5, 3);

    // Move to position 4 (last column)
    screen.process(b"\x1b[1;5H");
    assert_eq!(screen.cursor_position(), (0, 4));

    // Try to write wide character - should wrap to next line
    screen.process("한".as_bytes());

    let buffer = &screen.buffer;
    // Wide character should be on the next line
    assert_eq!(buffer[1][0].c, '한');
    assert!(buffer[1][0].wide);
    assert!(buffer[1][1].placeholder);

    assert_eq!(screen.cursor_position(), (1, 2));
}

#[test]
fn test_overwrite_wide_character() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write wide character
    screen.process("한".as_bytes());
    assert!(screen.buffer[0][0].wide);
    assert!(screen.buffer[0][1].placeholder);

    // Move cursor back to position 0
    screen.process(b"\x1b[1;1H");

    // Overwrite with regular character
    screen.process(b"X");

    let buffer = &screen.buffer;
    // X should be at position 0
    assert_eq!(buffer[0][0].c, 'X');
    assert!(!buffer[0][0].wide);

    // Placeholder should be cleared
    assert_eq!(buffer[0][1].c, ' ');
    assert!(!buffer[0][1].wide);
    assert!(!buffer[0][1].placeholder);
}

#[test]
fn test_overwrite_wide_character_placeholder() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write wide character
    screen.process("한".as_bytes());
    assert!(screen.buffer[0][0].wide);
    assert!(screen.buffer[0][1].placeholder);

    // Move cursor to placeholder position
    screen.process(b"\x1b[1;2H");

    // Overwrite placeholder with regular character
    screen.process(b"X");

    let buffer = &screen.buffer;
    // Wide character should be cleared
    assert_eq!(buffer[0][0].c, ' ');
    assert!(!buffer[0][0].wide);

    // X should be at position 1
    assert_eq!(buffer[0][1].c, 'X');
    assert!(!buffer[0][1].wide);
    assert!(!buffer[0][1].placeholder);
}

#[test]
fn test_multiple_wide_characters() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write multiple Korean characters: "한글"
    screen.process("한글".as_bytes());

    let buffer = &screen.buffer;
    // First character at positions 0-1
    assert_eq!(buffer[0][0].c, '한');
    assert!(buffer[0][0].wide);
    assert!(buffer[0][1].placeholder);

    // Second character at positions 2-3
    assert_eq!(buffer[0][2].c, '글');
    assert!(buffer[0][2].wide);
    assert!(buffer[0][3].placeholder);

    assert_eq!(screen.cursor_position(), (0, 4));
}

#[test]
fn test_wide_character_with_formatting() {
    let mut screen = TerminalScreen::new(10, 3);

    // Write bold, underlined Korean character
    screen.process(b"\x1b[1;4m");
    screen.process("한".as_bytes());

    let buffer = &screen.buffer;
    // Both cells should have the same formatting
    assert!(buffer[0][0].bold);
    assert!(buffer[0][0].underline);
    assert!(buffer[0][1].bold);
    assert!(buffer[0][1].underline);
    assert!(buffer[0][0].wide);
    assert!(buffer[0][1].placeholder);
}

#[test]
fn test_emoji_wide_character() {
    let mut screen = TerminalScreen::new(10, 3);

    // Some emojis are wide characters
    // Note: emoji width depends on unicode-width crate's classification
    screen.process("👋".as_bytes());

    // Check that cursor moved (exact behavior depends on emoji width)
    let pos = screen.cursor_position();
    assert!(pos.1 > 0);
}

#[test]
fn test_application_cursor_keys_mode_sequences() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test that mode changes are correctly applied via escape sequences
    // Normal mode (default)
    assert!(!screen.application_cursor_keys());

    // Enable application mode (like vim does)
    screen.process(b"\x1b[?1h");
    assert!(screen.application_cursor_keys());

    // Disable application mode (back to normal)
    screen.process(b"\x1b[?1l");
    assert!(!screen.application_cursor_keys());

    // Test multiple mode changes
    for _ in 0..3 {
        screen.process(b"\x1b[?1h");
        assert!(screen.application_cursor_keys());
        screen.process(b"\x1b[?1l");
        assert!(!screen.application_cursor_keys());
    }
}

#[test]
fn test_device_attributes_primary_da1() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send Primary DA request (CSI c)
    screen.process(b"\x1b[c");

    // Should have one pending response
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "\x1b[?1;2c");
}

#[test]
fn test_device_attributes_secondary_da2() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send Secondary DA request (CSI > c)
    screen.process(b"\x1b[>c");

    // Should have one pending response
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "\x1b[>0;0;0c");
}

#[test]
fn test_device_status_report_dsr() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send DSR request (CSI 5 n)
    screen.process(b"\x1b[5n");

    // Should have one pending response
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "\x1b[0n");
}

#[test]
fn test_cursor_position_report_cpr() {
    let mut screen = TerminalScreen::new(80, 24);

    // Move cursor to position (5, 10)
    screen.process(b"\x1b[6;11H");
    assert_eq!(screen.cursor_position(), (5, 10));

    // Send CPR request (CSI 6 n)
    screen.process(b"\x1b[6n");

    // Should have one pending response with 1-based indexing
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "\x1b[6;11R");
}

#[test]
fn test_cpr_at_origin() {
    let mut screen = TerminalScreen::new(80, 24);

    // Cursor should be at origin (0, 0)
    assert_eq!(screen.cursor_position(), (0, 0));

    // Send CPR request
    screen.process(b"\x1b[6n");

    // Response should be (1, 1) in 1-based indexing
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "\x1b[1;1R");
}

#[test]
fn test_multiple_device_attribute_requests() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send multiple requests
    screen.process(b"\x1b[c"); // Primary DA
    screen.process(b"\x1b[>c"); // Secondary DA
    screen.process(b"\x1b[5n"); // DSR
    screen.process(b"\x1b[6n"); // CPR

    // Should have four pending responses
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 4);
    assert_eq!(responses[0], "\x1b[?1;2c");
    assert_eq!(responses[1], "\x1b[>0;0;0c");
    assert_eq!(responses[2], "\x1b[0n");
    assert_eq!(responses[3], "\x1b[1;1R");
}

#[test]
fn test_take_pending_responses_clears_queue() {
    let mut screen = TerminalScreen::new(80, 24);

    // Send DA request
    screen.process(b"\x1b[c");

    // Take responses
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);

    // Queue should be empty now
    let responses2 = screen.take_pending_responses();
    assert_eq!(responses2.len(), 0);
}

#[test]
fn test_device_attributes_with_regular_content() {
    let mut screen = TerminalScreen::new(80, 24);

    // Write some text
    screen.process(b"Hello, World!");

    // Send DA request
    screen.process(b"\x1b[c");

    // Should have response and text should be preserved
    let responses = screen.take_pending_responses();
    assert_eq!(responses.len(), 1);

    // Check that text is still there
    let buffer = &screen.buffer;
    assert_eq!(buffer[0][0].c, 'H');
    assert_eq!(buffer[0][1].c, 'e');
}

#[test]
fn test_sgr_dim() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable dim
    screen.process(b"\x1b[2m");
    assert!(screen.dim);

    // Write text with dim
    screen.process(b"dim text");
    assert!(screen.buffer[0][0].dim);

    // Reset
    screen.process(b"\x1b[0m");
    assert!(!screen.dim);
}

#[test]
fn test_sgr_italic() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable italic
    screen.process(b"\x1b[3m");
    assert!(screen.italic);

    // Write text with italic
    screen.process(b"italic text");
    assert!(screen.buffer[0][0].italic);

    // Disable italic (SGR 23)
    screen.process(b"\x1b[23m");
    assert!(!screen.italic);
}

#[test]
fn test_sgr_strikethrough() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable strikethrough
    screen.process(b"\x1b[9m");
    assert!(screen.strikethrough);

    // Write text with strikethrough
    screen.process(b"strike");
    assert!(screen.buffer[0][0].strikethrough);

    // Disable strikethrough (SGR 29)
    screen.process(b"\x1b[29m");
    assert!(!screen.strikethrough);
}

#[test]
fn test_sgr_normal_intensity() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable bold
    screen.process(b"\x1b[1m");
    assert!(screen.bold);
    assert!(!screen.dim);

    // Enable dim (should not affect bold in this test)
    screen.process(b"\x1b[2m");
    assert!(screen.dim);

    // SGR 22 - Normal intensity (disables both bold and dim)
    screen.process(b"\x1b[22m");
    assert!(!screen.bold);
    assert!(!screen.dim);
}

#[test]
fn test_sgr_combined_attributes() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable multiple attributes
    screen.process(b"\x1b[1;3;4;9m");
    assert!(screen.bold);
    assert!(screen.italic);
    assert!(screen.underline);
    assert!(screen.strikethrough);

    // Write text with combined attributes
    screen.process(b"styled");
    let cell = &screen.buffer[0][0];
    assert!(cell.bold);
    assert!(cell.italic);
    assert!(cell.underline);
    assert!(cell.strikethrough);

    // Reset all
    screen.process(b"\x1b[0m");
    assert!(!screen.bold);
    assert!(!screen.italic);
    assert!(!screen.underline);
    assert!(!screen.strikethrough);
}

#[test]
fn test_sgr_dim_and_bold() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable bold
    screen.process(b"\x1b[1mBold");
    assert!(screen.bold);
    assert!(!screen.dim);

    // Enable dim (both can be active, though visually may conflict)
    screen.process(b"\x1b[2mDim");
    assert!(screen.bold);
    assert!(screen.dim);

    // SGR 22 should reset both
    screen.process(b"\x1b[22mNormal");
    assert!(!screen.bold);
    assert!(!screen.dim);
}

#[test]
fn test_sgr_reset_specific_attributes() {
    let mut screen = TerminalScreen::new(80, 24);

    // Enable all new attributes
    screen.process(b"\x1b[2;3;9m");
    assert!(screen.dim);
    assert!(screen.italic);
    assert!(screen.strikethrough);

    // Reset italic (SGR 23)
    screen.process(b"\x1b[23m");
    assert!(screen.dim);
    assert!(!screen.italic);
    assert!(screen.strikethrough);

    // Reset dim (SGR 22)
    screen.process(b"\x1b[22m");
    assert!(!screen.dim);
    assert!(!screen.italic);
    assert!(screen.strikethrough);

    // Reset strikethrough (SGR 29)
    screen.process(b"\x1b[29m");
    assert!(!screen.dim);
    assert!(!screen.italic);
    assert!(!screen.strikethrough);
}

#[test]
fn test_sgr_256_color_foreground() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test 256-color foreground (SGR 38;5;N)
    // Standard color (0-15)
    screen.process(b"\x1b[38;5;9mRed");
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(9)));
    let cell = &screen.buffer[0][0];
    assert_eq!(cell.c, 'R');
    assert_eq!(cell.fg, Some(AnsiColor::Palette256(9)));

    // 6x6x6 color cube (16-231)
    screen.process(b"\x1b[38;5;196mBright");
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(196)));

    // Grayscale (232-255)
    screen.process(b"\x1b[38;5;244mGray");
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(244)));

    // Reset to default
    screen.process(b"\x1b[39m");
    assert_eq!(screen.current_fg, None);
}

#[test]
fn test_sgr_256_color_background() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test 256-color background (SGR 48;5;N)
    // Standard color
    screen.process(b"\x1b[48;5;12mBlue");
    assert_eq!(screen.current_bg, Some(AnsiColor::Palette256(12)));
    let cell = &screen.buffer[0][0];
    assert_eq!(cell.c, 'B');
    assert_eq!(cell.bg, Some(AnsiColor::Palette256(12)));

    // 6x6x6 color cube
    screen.process(b"\x1b[48;5;46mGreen");
    assert_eq!(screen.current_bg, Some(AnsiColor::Palette256(46)));

    // Grayscale
    screen.process(b"\x1b[48;5;240mDark");
    assert_eq!(screen.current_bg, Some(AnsiColor::Palette256(240)));

    // Reset to default
    screen.process(b"\x1b[49m");
    assert_eq!(screen.current_bg, None);
}

#[test]
fn test_sgr_truecolor_foreground() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test TrueColor foreground (SGR 38;2;R;G;B)
    screen.process(b"\x1b[38;2;255;128;64mOrange");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(255, 128, 64)));
    let cell = &screen.buffer[0][0];
    assert_eq!(cell.c, 'O');
    assert_eq!(cell.fg, Some(AnsiColor::Rgb(255, 128, 64)));

    // Test pure colors
    screen.process(b"\x1b[38;2;255;0;0mRed");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(255, 0, 0)));

    screen.process(b"\x1b[38;2;0;255;0mGreen");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(0, 255, 0)));

    screen.process(b"\x1b[38;2;0;0;255mBlue");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(0, 0, 255)));

    // Test black and white
    screen.process(b"\x1b[38;2;0;0;0mBlack");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(0, 0, 0)));

    screen.process(b"\x1b[38;2;255;255;255mWhite");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(255, 255, 255)));

    // Reset to default
    screen.process(b"\x1b[39m");
    assert_eq!(screen.current_fg, None);
}

#[test]
fn test_sgr_truecolor_background() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test TrueColor background (SGR 48;2;R;G;B)
    screen.process(b"\x1b[48;2;100;200;150mCyan");
    assert_eq!(screen.current_bg, Some(AnsiColor::Rgb(100, 200, 150)));
    let cell = &screen.buffer[0][0];
    assert_eq!(cell.c, 'C');
    assert_eq!(cell.bg, Some(AnsiColor::Rgb(100, 200, 150)));

    // Test various colors
    screen.process(b"\x1b[48;2;64;128;192mBlue");
    assert_eq!(screen.current_bg, Some(AnsiColor::Rgb(64, 128, 192)));

    screen.process(b"\x1b[48;2;255;255;0mYellow");
    assert_eq!(screen.current_bg, Some(AnsiColor::Rgb(255, 255, 0)));

    // Reset to default
    screen.process(b"\x1b[49m");
    assert_eq!(screen.current_bg, None);
}

#[test]
fn test_sgr_mixed_color_modes() {
    let mut screen = TerminalScreen::new(80, 24);

    // Mix 16-color, 256-color, and TrueColor
    // 16-color foreground + 256-color background
    screen.process(b"\x1b[31;48;5;220mTest1");
    assert_eq!(screen.current_fg, Some(AnsiColor::Indexed(1)));
    assert_eq!(screen.current_bg, Some(AnsiColor::Palette256(220)));

    // TrueColor foreground + 16-color background
    screen.process(b"\x1b[38;2;128;64;200;44mTest2");
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(128, 64, 200)));
    assert_eq!(screen.current_bg, Some(AnsiColor::Indexed(4)));

    // 256-color foreground + TrueColor background
    screen.process(b"\x1b[38;5;100;48;2;50;100;150mTest3");
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(100)));
    assert_eq!(screen.current_bg, Some(AnsiColor::Rgb(50, 100, 150)));

    // Reset all
    screen.process(b"\x1b[0m");
    assert_eq!(screen.current_fg, None);
    assert_eq!(screen.current_bg, None);
}

#[test]
fn test_sgr_color_with_attributes() {
    let mut screen = TerminalScreen::new(80, 24);

    // Combine TrueColor with text attributes
    screen.process(b"\x1b[1;3;4;38;2;255;100;50mBold Italic Underline Orange");
    assert!(screen.bold);
    assert!(screen.italic);
    assert!(screen.underline);
    assert_eq!(screen.current_fg, Some(AnsiColor::Rgb(255, 100, 50)));

    let cell = &screen.buffer[0][0];
    assert_eq!(cell.c, 'B');
    assert!(cell.bold);
    assert!(cell.italic);
    assert!(cell.underline);
    assert_eq!(cell.fg, Some(AnsiColor::Rgb(255, 100, 50)));

    // Combine 256-color with attributes
    screen.process(b"\x1b[2;9;48;5;200mDim Strikethrough");
    assert!(screen.dim);
    assert!(screen.strikethrough);
    assert_eq!(screen.current_bg, Some(AnsiColor::Palette256(200)));
}

#[test]
fn test_ansi_color_to_iced_color() {
    // Test Indexed color conversion
    let indexed = AnsiColor::Indexed(1); // Red
    let color = indexed.to_color();
    assert_eq!(color, Color::from_rgb(0.8, 0.2, 0.2));

    // Test 256-color palette conversion (standard color)
    let palette_std = AnsiColor::Palette256(9); // Bright Red
    let color_std = palette_std.to_color();
    assert_eq!(color_std, Color::from_rgb(1.0, 0.3, 0.3));

    // Test 256-color palette conversion (color cube)
    let palette_cube = AnsiColor::Palette256(16); // First color in 6x6x6 cube
    let color_cube = palette_cube.to_color();
    assert_eq!(color_cube, Color::from_rgb(0.0, 0.0, 0.0));

    let palette_cube2 = AnsiColor::Palette256(231); // Last color in 6x6x6 cube
    let color_cube2 = palette_cube2.to_color();
    assert_eq!(color_cube2, Color::from_rgb(1.0, 1.0, 1.0));

    // Test 256-color palette conversion (grayscale)
    let palette_gray = AnsiColor::Palette256(232); // First grayscale
    let color_gray = palette_gray.to_color();
    let expected_gray = 8.0 / 255.0;
    assert_eq!(
        color_gray,
        Color::from_rgb(expected_gray, expected_gray, expected_gray)
    );

    let palette_gray_mid = AnsiColor::Palette256(244); // Mid grayscale
    let color_gray_mid = palette_gray_mid.to_color();
    let expected_gray_mid = (8.0 + 12.0 * 10.0) / 255.0;
    assert_eq!(
        color_gray_mid,
        Color::from_rgb(expected_gray_mid, expected_gray_mid, expected_gray_mid)
    );

    // Test RGB color conversion
    let rgb = AnsiColor::Rgb(255, 128, 64);
    let color_rgb = rgb.to_color();
    assert_eq!(color_rgb, Color::from_rgb(1.0, 128.0 / 255.0, 64.0 / 255.0));

    let rgb_black = AnsiColor::Rgb(0, 0, 0);
    let color_black = rgb_black.to_color();
    assert_eq!(color_black, Color::from_rgb(0.0, 0.0, 0.0));

    let rgb_white = AnsiColor::Rgb(255, 255, 255);
    let color_white = rgb_white.to_color();
    assert_eq!(color_white, Color::from_rgb(1.0, 1.0, 1.0));
}

#[test]
fn test_256_color_palette_ranges() {
    let mut screen = TerminalScreen::new(80, 24);

    // Test all standard colors (0-15)
    for i in 0..=15 {
        screen.process(format!("\x1b[38;5;{}m.", i).as_bytes());
        assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(i)));
    }

    // Test color cube boundaries (16-231)
    screen.process(b"\x1b[38;5;16m."); // First cube color
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(16)));

    screen.process(b"\x1b[38;5;231m."); // Last cube color
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(231)));

    // Test grayscale boundaries (232-255)
    screen.process(b"\x1b[38;5;232m."); // First grayscale
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(232)));

    screen.process(b"\x1b[38;5;255m."); // Last grayscale
    assert_eq!(screen.current_fg, Some(AnsiColor::Palette256(255)));
}

#[test]
fn test_color_cube_calculation() {
    // Test the 6x6x6 color cube calculation
    // Color 16 should be (0,0,0) in the cube = RGB(0,0,0)
    let color_16 = palette256_to_color(16);
    assert_eq!(color_16, Color::from_rgb(0.0, 0.0, 0.0));

    // Color 21 should be (0,0,5) in the cube = RGB(0,0,255)
    let color_21 = palette256_to_color(21);
    assert_eq!(color_21, Color::from_rgb(0.0, 0.0, 255.0 / 255.0));

    // Color 226 should be (5,5,0) in the cube = RGB(255,255,0) - yellow
    let color_226 = palette256_to_color(226);
    assert_eq!(
        color_226,
        Color::from_rgb(255.0 / 255.0, 255.0 / 255.0, 0.0)
    );

    // Color 231 should be (5,5,5) in the cube = RGB(255,255,255)
    let color_231 = palette256_to_color(231);
    assert_eq!(color_231, Color::from_rgb(1.0, 1.0, 1.0));
}
#[cfg(test)]
mod resize_tests {
    use super::*;

    #[test]
    fn test_resize_no_change() {
        let mut screen = TerminalScreen::new(80, 24);
        screen.process(b"Hello, World!");

        // Resize to same dimensions should be a no-op
        screen.resize(80, 24);

        assert_eq!(screen.buffer.len(), 24);
        assert_eq!(screen.buffer[0].len(), 80);
        assert_eq!(screen.buffer[0][0].c, 'H');
    }

    #[test]
    fn test_resize_columns_increase() {
        let mut screen = TerminalScreen::new(10, 5);
        screen.process(b"Hello");

        // Increase columns from 10 to 20
        screen.resize(20, 5);

        assert_eq!(screen.buffer.len(), 5);
        assert_eq!(screen.buffer[0].len(), 20);
        // Original content should be preserved
        assert_eq!(screen.buffer[0][0].c, 'H');
        assert_eq!(screen.buffer[0][1].c, 'e');
        assert_eq!(screen.buffer[0][2].c, 'l');
        assert_eq!(screen.buffer[0][3].c, 'l');
        assert_eq!(screen.buffer[0][4].c, 'o');
        // New cells should be empty
        assert_eq!(screen.buffer[0][10].c, ' ');
        assert_eq!(screen.buffer[0][19].c, ' ');
    }

    #[test]
    fn test_resize_columns_decrease() {
        let mut screen = TerminalScreen::new(20, 5);
        screen.process(b"Hello, World!");

        // Decrease columns from 20 to 10
        screen.resize(10, 5);

        assert_eq!(screen.buffer.len(), 5);
        assert_eq!(screen.buffer[0].len(), 10);
        // Content should be truncated
        assert_eq!(screen.buffer[0][0].c, 'H');
        assert_eq!(screen.buffer[0][1].c, 'e');
        assert_eq!(screen.buffer[0][9].c, 'r');
    }

    #[test]
    fn test_resize_rows_decrease() {
        let mut screen = TerminalScreen::new(10, 5);
        // Fill with text
        for i in 0..5 {
            screen.process(format!("Line {}\r\n", i).as_bytes());
        }

        let scrollback_before = screen.scrollback.len();

        // Decrease rows from 5 to 3
        screen.resize(10, 3);

        assert_eq!(screen.buffer.len(), 3);
        // Top 2 lines should be moved to scrollback
        assert_eq!(screen.scrollback.len(), scrollback_before + 2);
    }

    #[test]
    fn test_resize_cursor_adjustment() {
        let mut screen = TerminalScreen::new(80, 24);
        // Move cursor to position
        screen.process(b"\x1b[10;40H"); // Row 10, Col 40

        // Resize to smaller dimensions
        screen.resize(30, 15);

        let (row, col) = screen.cursor_position();
        // Cursor should be adjusted to fit within new bounds
        assert!(row < 15);
        assert!(col < 30);
    }

    #[test]
    fn test_resize_minimum_dimensions() {
        let mut screen = TerminalScreen::new(80, 24);

        // Try to resize to 0x0 (should be clamped to 1x1)
        screen.resize(0, 0);

        assert_eq!(screen.buffer.len(), 1);
        assert_eq!(screen.buffer[0].len(), 1);
    }
}

#[cfg(test)]
mod alternate_screen_tests {
    use super::*;

    #[test]
    fn test_alternate_screen_basic_switch() {
        let mut screen = TerminalScreen::new(80, 24);

        // Write some text on main screen
        screen.process(b"Main screen text");
        let main_buffer = screen.buffer.clone();

        // Switch to alternate screen (CSI ?47h)
        screen.process(b"\x1b[?47h");
        assert!(screen.use_alternate_screen);

        // Alternate screen should be clear
        assert_eq!(screen.buffer[0][0].c, ' ');

        // Write on alternate screen
        screen.process(b"Alternate screen text");

        // Switch back (CSI ?47l)
        screen.process(b"\x1b[?47l");
        assert!(!screen.use_alternate_screen);

        // Main screen text should be restored
        assert_eq!(screen.buffer[0][0].c, main_buffer[0][0].c);
    }

    #[test]
    fn test_alternate_screen_1047_clear() {
        let mut screen = TerminalScreen::new(80, 24);

        // Write on main screen
        screen.process(b"Main screen");

        // Switch with CSI ?1047h (should clear alternate screen)
        screen.process(b"\x1b[?1047h");
        assert!(screen.use_alternate_screen);
        assert_eq!(screen.buffer[0][0].c, ' ');

        // Write on alternate screen (cursor is preserved, so reset it first)
        screen.process(b"[1;1HAlt");
        assert_eq!(screen.buffer[0][0].c, 'A');

        // Switch back
        screen.process(b"\x1b[?1047l");
        assert!(!screen.use_alternate_screen);
        assert_eq!(screen.buffer[0][0].c, 'M'); // 'Main screen'
    }

    #[test]
    fn test_alternate_screen_1049_full_save() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set up main screen with cursor position and attributes
        screen.process(b"\x1b[5;10HMain \x1b[31mRed Text\x1b[0m");
        let (row, col) = screen.cursor_position();

        // Switch with CSI ?1049h (save cursor + clear)
        screen.process(b"\x1b[?1049h");
        assert!(screen.use_alternate_screen);

        // Cursor should be reset to 0,0 in alternate screen
        let (alt_row, alt_col) = screen.cursor_position();
        assert_eq!(alt_row, 0);
        assert_eq!(alt_col, 0);

        // Write on alternate screen
        screen.process(b"Alternate content");

        // Switch back - should restore cursor position
        screen.process(b"\x1b[?1049l");
        assert!(!screen.use_alternate_screen);

        // Original content and cursor position should be restored
        let (restored_row, restored_col) = screen.cursor_position();
        assert_eq!(restored_row, row);
        assert_eq!(restored_col, col);
    }

    #[test]
    fn test_alternate_screen_preserves_attributes() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set text attributes on main screen
        screen.process(b"\x1b[1;4;31mBold Red Underline\x1b[0m");
        screen.process(b"\x1b[1m"); // Set bold again

        // Switch to alternate screen
        screen.process(b"\x1b[?1049h");

        // Attributes should be reset in alternate screen
        assert!(!screen.bold);

        // Set different attributes in alternate screen
        screen.process(b"\x1b[34mBlue");
        assert!(screen.current_fg.is_some());

        // Switch back to main screen
        screen.process(b"\x1b[?1049l");

        // Original attributes should be restored (bold was set)
        assert!(screen.bold);
    }

    #[test]
    fn test_alternate_screen_preserves_scroll_region() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set scroll region on main screen
        screen.process(b"\x1b[5;20r"); // Lines 5-20
        assert_eq!(screen.scroll_region, Some((4, 19))); // 0-indexed

        // Switch to alternate screen
        screen.process(b"\x1b[?1049h");

        // Scroll region should be cleared in alternate screen
        assert_eq!(screen.scroll_region, None);

        // Set different scroll region in alternate screen
        screen.process(b"\x1b[1;10r");
        assert_eq!(screen.scroll_region, Some((0, 9)));

        // Switch back
        screen.process(b"\x1b[?1049l");

        // Original scroll region should be restored
        assert_eq!(screen.scroll_region, Some((4, 19)));
    }

    #[test]
    fn test_alternate_screen_preserves_scrollback() {
        let mut screen = TerminalScreen::new(80, 3);

        // Generate scrollback on main screen
        for i in 0..10 {
            screen.process(format!("Line {}\n", i).as_bytes());
        }

        let scrollback_len = screen.scrollback.len();
        assert!(scrollback_len > 0, "Should have scrollback");

        // Switch to alternate screen
        screen.process(b"\x1b[?1049h");

        // Alternate screen should have empty scrollback
        assert_eq!(screen.scrollback.len(), 0);

        // Generate scrollback in alternate screen
        for i in 0..5 {
            screen.process(format!("Alt Line {}\n", i).as_bytes());
        }

        // Switch back
        screen.process(b"\x1b[?1049l");

        // Original scrollback should be restored
        assert_eq!(screen.scrollback.len(), scrollback_len);
    }

    #[test]
    fn test_alternate_screen_vim_like_behavior() {
        let mut screen = TerminalScreen::new(80, 24);

        // Simulate vim-like usage
        // 1. User has text on main screen
        screen.process(b"$ ls\nfile1.txt\nfile2.txt\n$ ");
        let main_content = screen.buffer.clone();

        // 2. Launch vim (CSI ?1049h)
        screen.process(b"\x1b[?1049h");
        assert!(screen.use_alternate_screen);

        // 3. Vim content
        screen.process(b"~\n~\n~\nfile.txt [New File]");

        // 4. Exit vim (CSI ?1049l)
        screen.process(b"\x1b[?1049l");
        assert!(!screen.use_alternate_screen);

        // 5. User should see original shell prompt
        assert_eq!(screen.buffer[3][2].c, main_content[3][2].c);
    }

    #[test]
    fn test_alternate_screen_less_like_behavior() {
        let mut screen = TerminalScreen::new(80, 24);

        // Main screen with command
        screen.process(b"$ cat bigfile.txt | less\n");

        // less enters alternate screen with CSI ?1047h
        screen.process(b"\x1b[?1047h");

        // Display file content
        screen.process(b"Line 1 of file\nLine 2 of file\nLine 3 of file");

        // User quits less (CSI ?1047l)
        screen.process(b"\x1b[?1047l");

        // Should return to shell prompt
        assert_eq!(screen.buffer[0][0].c, '$');
    }

    #[test]
    fn test_alternate_screen_multiple_switches() {
        let mut screen = TerminalScreen::new(80, 24);

        screen.process(b"Main1");

        // First switch
        screen.process(b"\x1b[?1049h");
        screen.process(b"Alt1");
        screen.process(b"\x1b[?1049l");
        assert_eq!(screen.buffer[0][0].c, 'M');

        // Second switch
        screen.process(b"\x1b[?1049h");
        screen.process(b"Alt2");
        screen.process(b"\x1b[?1049l");
        assert_eq!(screen.buffer[0][0].c, 'M');

        // State should be consistent after multiple switches
        assert!(!screen.use_alternate_screen);
    }

    #[test]
    fn test_alternate_screen_htop_like_behavior() {
        let mut screen = TerminalScreen::new(80, 24);

        // Shell prompt
        screen.process(b"$ htop\n");

        // htop uses CSI ?1049h
        screen.process(b"\x1b[?1049h");
        assert!(screen.use_alternate_screen);

        // htop draws interface with colors and positioning
        screen.process(b"\x1b[1;1H\x1b[7mCPU [|||||||||||||||| 100%]\x1b[0m");
        screen.process(b"\x1b[2;1HMem [|||||            50%]");

        // Exit htop
        screen.process(b"\x1b[?1049l");
        assert!(!screen.use_alternate_screen);

        // Back to shell
        assert_eq!(screen.buffer[0][0].c, '$');
    }

    #[test]
    fn test_alternate_screen_preserves_decsc_state() {
        let mut screen = TerminalScreen::new(80, 24);

        // Save cursor with DECSC on main screen
        screen.process(b"\x1b[10;20H"); // Move cursor
        screen.process(b"\x1b7"); // DECSC - save cursor
        screen.process(b"\x1b[1;1H"); // Move somewhere else

        // Switch to alternate screen
        screen.process(b"\x1b[?1049h");

        // DECSC state should be cleared in alternate screen
        screen.process(b"\x1b8"); // DECRC - restore cursor
        let (_row, _col) = screen.cursor_position();
        // Should be at origin or unchanged, not at saved position

        // Switch back
        screen.process(b"\x1b[?1049l");

        // Original DECSC state should be restored
        screen.process(b"\x1b8"); // DECRC
        let (restored_row, restored_col) = screen.cursor_position();
        assert_eq!(restored_row, 9); // 10th row (0-indexed)
        assert_eq!(restored_col, 19); // 20th column (0-indexed)
    }

    // Tests for new terminal sequences

    #[test]
    fn test_ansi_cursor_save_restore() {
        let mut screen = TerminalScreen::new(80, 24);

        // Move cursor to a specific position
        screen.process(b"\x1b[10;20H");
        let (row, col) = screen.cursor_position();
        assert_eq!(row, 9); // 10th row (0-indexed)
        assert_eq!(col, 19); // 20th column (0-indexed)

        // Save cursor with ANSI.SYS style (CSI s)
        screen.process(b"\x1b[s");

        // Move cursor somewhere else
        screen.process(b"\x1b[1;1H");
        let (row, col) = screen.cursor_position();
        assert_eq!(row, 0);
        assert_eq!(col, 0);

        // Restore cursor with ANSI.SYS style (CSI u)
        screen.process(b"\x1b[u");
        let (row, col) = screen.cursor_position();
        assert_eq!(row, 9);
        assert_eq!(col, 19);
    }

    #[test]
    fn test_cursor_blink_enable_disable() {
        let mut screen = TerminalScreen::new(80, 24);

        // Default state - blink enabled
        assert!(screen.cursor_blink_enabled());

        // Disable cursor blink (CSI ?12l)
        screen.process(b"\x1b[?12l");
        assert!(!screen.cursor_blink_enabled());

        // Enable cursor blink (CSI ?12h)
        screen.process(b"\x1b[?12h");
        assert!(screen.cursor_blink_enabled());
    }

    #[test]
    fn test_osc_4_set_palette_color() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set color palette entry 1 (red) to a custom color
        screen.process(b"\x1b]4;1;rgb:ff/00/00\x07");

        // Verify the color was set
        let color = screen.get_palette_color(1);
        assert_eq!(color, Some((255, 0, 0)));
    }

    #[test]
    fn test_osc_4_query_palette_color() {
        let mut screen = TerminalScreen::new(80, 24);

        // Query color palette entry 0 (black)
        screen.process(b"\x1b]4;0;?\x07");

        // Should have a pending response
        let responses = screen.take_pending_responses();
        assert_eq!(responses.len(), 1);
        assert!(responses[0].starts_with("\x1b]4;0;rgb:"));
    }

    #[test]
    fn test_osc_10_set_default_foreground() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set default foreground color to white
        screen.process(b"\x1b]10;rgb:ff/ff/ff\x07");

        // Verify the color was set
        assert_eq!(screen.default_fg_color(), Some((255, 255, 255)));
    }

    #[test]
    fn test_osc_10_query_default_foreground() {
        let mut screen = TerminalScreen::new(80, 24);

        // Query default foreground color
        screen.process(b"\x1b]10;?\x07");

        // Should have a pending response with default gray color
        let responses = screen.take_pending_responses();
        assert_eq!(responses.len(), 1);
        assert!(responses[0].starts_with("\x1b]10;rgb:"));
        assert!(responses[0].contains("cc/cc/cc")); // Default gray (204, 204, 204)
    }

    #[test]
    fn test_osc_11_set_default_background() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set default background color to dark blue
        screen.process(b"\x1b]11;rgb:00/00/80\x07");

        // Verify the color was set
        assert_eq!(screen.default_bg_color(), Some((0, 0, 128)));
    }

    #[test]
    fn test_osc_11_query_default_background() {
        let mut screen = TerminalScreen::new(80, 24);

        // Query default background color
        screen.process(b"\x1b]11;?\x07");

        // Should have a pending response with default black color
        let responses = screen.take_pending_responses();
        assert_eq!(responses.len(), 1);
        assert!(responses[0].starts_with("\x1b]11;rgb:"));
        assert!(responses[0].contains("00/00/00")); // Default black
    }

    #[test]
    fn test_osc_52_clipboard_set() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set clipboard content (base64 encoded "Hello")
        screen.process(b"\x1b]52;c;SGVsbG8=\x07");

        // Verify clipboard request was stored
        let clipboard = screen.take_clipboard_request();
        assert_eq!(clipboard, Some("SGVsbG8=".to_string()));
    }

    #[test]
    fn test_osc_52_clipboard_query_ignored() {
        let mut screen = TerminalScreen::new(80, 24);

        // Query clipboard (should be ignored, not stored)
        screen.process(b"\x1b]52;c;?\x07");

        // No clipboard request should be stored for queries
        let clipboard = screen.take_clipboard_request();
        assert_eq!(clipboard, None);
    }

    #[test]
    fn test_color_palette_initialization() {
        let screen = TerminalScreen::new(80, 24);

        // Test standard 16 colors
        assert_eq!(screen.get_palette_color(0), Some((0, 0, 0))); // Black
        assert_eq!(screen.get_palette_color(1), Some((204, 51, 51))); // Red
        assert_eq!(screen.get_palette_color(7), Some((204, 204, 204))); // White
        assert_eq!(screen.get_palette_color(15), Some((255, 255, 255))); // Bright White

        // Test 6x6x6 color cube (index 16 should be 0,0,0)
        assert_eq!(screen.get_palette_color(16), Some((0, 0, 0)));

        // Test grayscale (index 232 should be first grayscale)
        assert_eq!(screen.get_palette_color(232), Some((8, 8, 8)));
    }

    #[test]
    fn test_parse_rgb_color_2_digit() {
        let screen = TerminalScreen::new(80, 24);

        // Test 2-digit hex format
        let color = screen.parse_rgb_color("rgb:ff/00/80");
        assert_eq!(color, Some((255, 0, 128)));
    }

    #[test]
    fn test_parse_rgb_color_4_digit() {
        let screen = TerminalScreen::new(80, 24);

        // Test 4-digit hex format (should use first 2 digits)
        let color = screen.parse_rgb_color("rgb:ff00/0080/8000");
        assert_eq!(color, Some((255, 0, 128)));
    }

    #[test]
    fn test_parse_rgb_color_invalid() {
        let screen = TerminalScreen::new(80, 24);

        // Test invalid formats
        assert_eq!(screen.parse_rgb_color("rgb:ff/00"), None); // Too few components
        assert_eq!(screen.parse_rgb_color("ff/00/80"), None); // Missing prefix
        assert_eq!(screen.parse_rgb_color("rgb:gg/00/00"), None); // Invalid hex
    }

    #[test]
    fn test_multiple_color_operations() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set multiple palette colors
        screen.process(b"\x1b]4;1;rgb:ff/00/00\x07"); // Red
        screen.process(b"\x1b]4;2;rgb:00/ff/00\x07"); // Green
        screen.process(b"\x1b]4;3;rgb:00/00/ff\x07"); // Blue

        // Verify all were set correctly
        assert_eq!(screen.get_palette_color(1), Some((255, 0, 0)));
        assert_eq!(screen.get_palette_color(2), Some((0, 255, 0)));
        assert_eq!(screen.get_palette_color(3), Some((0, 0, 255)));

        // Set default colors
        screen.process(b"\x1b]10;rgb:aa/bb/cc\x07"); // Foreground
        screen.process(b"\x1b]11;rgb:11/22/33\x07"); // Background

        assert_eq!(screen.default_fg_color(), Some((170, 187, 204)));
        assert_eq!(screen.default_bg_color(), Some((17, 34, 51)));
    }

    #[test]
    fn test_cursor_operations_combined() {
        let mut screen = TerminalScreen::new(80, 24);

        // Test combining cursor save/restore with blink control
        screen.process(b"\x1b[10;20H"); // Move cursor
        screen.process(b"\x1b[s"); // Save cursor (ANSI.SYS)
        screen.process(b"\x1b[?12l"); // Disable blink

        assert!(!screen.cursor_blink_enabled());

        screen.process(b"\x1b[1;1H"); // Move cursor
        screen.process(b"\x1b[u"); // Restore cursor

        let (row, col) = screen.cursor_position();
        assert_eq!(row, 9);
        assert_eq!(col, 19);
        assert!(!screen.cursor_blink_enabled()); // Blink state preserved
    }

    // New tests for improved alternate screen functionality

    #[test]
    fn test_1049h_saves_cursor_and_resets() {
        let mut screen = TerminalScreen::new(80, 24);

        // Position cursor on main screen
        screen.process(b"[15;30H");
        let (orig_row, orig_col) = screen.cursor_position();
        assert_eq!(orig_row, 14); // 0-indexed
        assert_eq!(orig_col, 29);

        // Enter alternate screen with ?1049h (should save cursor and reset to 0,0)
        screen.process(b"[?1049h");
        assert!(screen.is_alternate_screen());

        // Cursor should be at 0,0 in alternate screen
        let (alt_row, alt_col) = screen.cursor_position();
        assert_eq!(alt_row, 0);
        assert_eq!(alt_col, 0);

        // Exit alternate screen
        screen.process(b"[?1049l");
        assert!(!screen.is_alternate_screen());

        // Cursor should be restored to original position
        let (restored_row, restored_col) = screen.cursor_position();
        assert_eq!(restored_row, orig_row);
        assert_eq!(restored_col, orig_col);
    }

    #[test]
    fn test_1047h_clears_but_no_cursor_save() {
        let mut screen = TerminalScreen::new(80, 24);

        // Position cursor on main screen
        screen.process(b"[10;20H");
        let (orig_row, orig_col) = screen.cursor_position();

        // Enter alternate screen with ?1047h (clear but cursor position preserved)
        screen.process(b"[?1047h");
        assert!(screen.is_alternate_screen());

        // Cursor position should NOT be reset to 0,0
        let (alt_row, alt_col) = screen.cursor_position();
        assert_eq!(alt_row, orig_row);
        assert_eq!(alt_col, orig_col);

        // Screen should be cleared
        assert_eq!(screen.buffer[0][0].c, ' ');
    }

    #[test]
    fn test_47h_no_clear_no_cursor_save() {
        let mut screen = TerminalScreen::new(80, 24);

        // Position cursor on main screen
        screen.process(b"[5;10H");
        let (orig_row, orig_col) = screen.cursor_position();

        // Enter alternate screen with ?47h (basic mode)
        screen.process(b"[?47h");
        assert!(screen.is_alternate_screen());

        // Cursor position should be preserved
        let (alt_row, alt_col) = screen.cursor_position();
        assert_eq!(alt_row, orig_row);
        assert_eq!(alt_col, orig_col);
    }

    #[test]
    fn test_public_enter_leave_methods() {
        let mut screen = TerminalScreen::new(80, 24);

        // Write on main screen
        screen.process(b"Main screen content");

        // Use public API to enter alternate screen
        screen.enter_alternate_screen(true, true);
        assert!(screen.is_alternate_screen());
        
        // Cursor should be at 0,0
        let (row, col) = screen.cursor_position();
        assert_eq!(row, 0);
        assert_eq!(col, 0);

        // Screen should be cleared
        assert_eq!(screen.buffer[0][0].c, ' ');

        // Write on alternate screen
        screen.process(b"Alternate content");

        // Leave alternate screen
        screen.leave_alternate_screen(true);
        assert!(!screen.is_alternate_screen());

        // Main screen content should be restored
        assert_eq!(screen.buffer[0][0].c, 'M'); // 'Main'
    }

    #[test]
    fn test_current_buffer_returns_correct_buffer() {
        let mut screen = TerminalScreen::new(80, 24);

        // Write on main screen
        screen.process(b"Main");
        let main_buffer = screen.current_buffer();
        assert_eq!(main_buffer[0][0].c, 'M');

        // Enter alternate screen
        screen.enter_alternate_screen(true, true);
        
        // Write on alternate screen
        screen.process(b"Alt");
        let alt_buffer = screen.current_buffer();
        assert_eq!(alt_buffer[0][0].c, 'A');
        
        // Main buffer should not be affected
        screen.leave_alternate_screen(true);
        let restored_buffer = screen.current_buffer();
        assert_eq!(restored_buffer[0][0].c, 'M');
    }

    #[test]
    fn test_alternate_screen_state_fields() {
        let mut screen = TerminalScreen::new(80, 24);

        // Set various attributes
        screen.process(b"[1;31mBold Red[0m");
        screen.process(b"[5;15r"); // Set scroll region
        
        // Save cursor with DECSC
        screen.process(b"[10;20H7");

        // Enter alternate screen
        screen.enter_alternate_screen(true, true);

        // Attributes should be reset in alternate screen
        assert!(!screen.bold);
        assert!(screen.current_fg.is_none());
        assert_eq!(screen.scroll_region, None);

        // Set different state in alternate screen
        screen.process(b"[34mBlue");
        screen.process(b"[1;10r");

        // Leave alternate screen
        screen.leave_alternate_screen(true);

        // Original scroll region should be restored
        assert_eq!(screen.scroll_region, Some((4, 14))); // 0-indexed
    }

    #[test]
    fn test_scrollback_disabled_in_alternate() {
        let mut screen = TerminalScreen::new(80, 3);

        // Generate scrollback on main screen
        for i in 0..5 {
            screen.process(format!("Main {}
", i).as_bytes());
        }
        let main_scrollback_len = screen.scrollback_size();
        assert!(main_scrollback_len > 0);

        // Enter alternate screen
        screen.enter_alternate_screen(true, true);

        // Scrollback should be empty in alternate screen
        assert_eq!(screen.scrollback_size(), 0);

        // Generate lines that would create scrollback
        for i in 0..5 {
            screen.process(format!("Alt {}
", i).as_bytes());
        }

        // Leave alternate screen
        screen.leave_alternate_screen(true);

        // Original scrollback should be restored
        assert_eq!(screen.scrollback_size(), main_scrollback_len);
    }

    #[test]
    fn test_vim_complete_workflow() {
        let mut screen = TerminalScreen::new(80, 24);

        // Shell session with scrollback
        for i in 0..30 {
            screen.process(format!("Command {}
", i).as_bytes());
        }
        
        let scrollback_before = screen.scrollback_size();
        let (cursor_row, cursor_col) = screen.cursor_position();

        // Vim launches with ?1049h
        screen.process(b"[?1049h");
        
        // Verify alternate screen state
        assert!(screen.is_alternate_screen());
        assert_eq!(screen.scrollback_size(), 0);
        assert_eq!(screen.cursor_position(), (0, 0));

        // Vim draws interface
        screen.process(b"\x1b[1;1H~\r\n~\r\n~\r\n\"file.txt\" [New File]");

        // User edits
        screen.process(b"[1;1HHello, World!");

        // Vim exits with ?1049l
        screen.process(b"[?1049l");

        // Verify restoration
        assert!(!screen.is_alternate_screen());
        assert_eq!(screen.scrollback_size(), scrollback_before);
        assert_eq!(screen.cursor_position(), (cursor_row, cursor_col));
        
        // Vim content should not be visible
        assert_ne!(screen.buffer[0][0].c, '~');
    }
}

#[cfg(test)]
mod compression_tests {
    use super::*;

    #[test]
    fn test_compress_empty_line() {
        let line: Vec<Cell> = vec![];
        let compressed = CompressedLine::compress(&line);

        assert_eq!(compressed.original_length, 0);
        assert_eq!(compressed.segments.len(), 0);
        assert_eq!(compressed.uncompressed_size, 0);
        assert_eq!(compressed.compressed_size, 0);

        let decompressed = compressed.decompress();
        assert_eq!(decompressed.len(), 0);
    }

    #[test]
    fn test_compress_single_cell() {
        let line = vec![Cell::default()];
        let compressed = CompressedLine::compress(&line);

        assert_eq!(compressed.original_length, 1);
        assert_eq!(compressed.segments.len(), 1);
        assert_eq!(compressed.segments[0].count, 1);

        let decompressed = compressed.decompress();
        assert_eq!(decompressed.len(), 1);
        assert_eq!(decompressed[0].c, ' ');
    }

    #[test]
    fn test_compress_uniform_line() {
        // All spaces - should compress to 1 segment
        let line = vec![Cell::default(); 80];
        let compressed = CompressedLine::compress(&line);

        assert_eq!(compressed.original_length, 80);
        assert_eq!(compressed.segments.len(), 1);
        assert_eq!(compressed.segments[0].count, 80);
        assert_eq!(compressed.segments[0].cell.c, ' ');

        // Verify compression ratio
        let ratio = compressed.compression_ratio();
        assert!(ratio < 0.1); // Should compress very well

        let decompressed = compressed.decompress();
        assert_eq!(decompressed.len(), 80);
        for cell in decompressed {
            assert_eq!(cell.c, ' ');
        }
    }

    #[test]
    fn test_compress_alternating_cells() {
        // Worst case: alternating characters
        let mut line = Vec::new();
        for i in 0..80 {
            let mut cell = Cell::default();
            cell.c = if i % 2 == 0 { 'A' } else { 'B' };
            line.push(cell);
        }

        let compressed = CompressedLine::compress(&line);

        assert_eq!(compressed.original_length, 80);
        assert_eq!(compressed.segments.len(), 80); // No compression possible

        // Compression ratio should be close to or greater than 1.0 (no benefit)
        let ratio = compressed.compression_ratio();
        assert!(ratio >= 0.9);

        let decompressed = compressed.decompress();
        assert_eq!(decompressed.len(), 80);
        for (i, cell) in decompressed.iter().enumerate() {
            let expected = if i % 2 == 0 { 'A' } else { 'B' };
            assert_eq!(cell.c, expected);
        }
    }

    #[test]
    fn test_compress_realistic_terminal_line() {
        // Realistic case: prompt + text + spaces
        let mut line = Vec::new();

        // "$ ls -la" followed by spaces
        let text = "$ ls -la";
        for ch in text.chars() {
            let mut cell = Cell::default();
            cell.c = ch;
            cell.bold = true;
            line.push(cell);
        }

        // Fill rest with spaces
        for _ in text.len()..80 {
            line.push(Cell::default());
        }

        let compressed = CompressedLine::compress(&line);

        assert_eq!(compressed.original_length, 80);
        // Should compress to: text.len() segments for text + 1 segment for spaces
        assert!(compressed.segments.len() <= text.len() + 1);

        // Should have good compression due to trailing spaces
        let ratio = compressed.compression_ratio();
        assert!(ratio < 0.5);

        let decompressed = compressed.decompress();
        assert_eq!(decompressed.len(), 80);
        assert_eq!(decompressed[0].c, '$');
        assert_eq!(decompressed[1].c, ' ');
        assert_eq!(decompressed[2].c, 'l');
        assert!(decompressed[79].c == ' ');
    }

    #[test]
    fn test_compress_line_with_colors() {
        let mut line = Vec::new();

        // Red 'A' repeated 20 times
        for _ in 0..20 {
            let mut cell = Cell::default();
            cell.c = 'A';
            cell.fg = Some(AnsiColor::Indexed(1)); // Red
            line.push(cell);
        }

        // Green 'B' repeated 20 times
        for _ in 0..20 {
            let mut cell = Cell::default();
            cell.c = 'B';
            cell.fg = Some(AnsiColor::Indexed(2)); // Green
            line.push(cell);
        }

        // Spaces
        for _ in 0..40 {
            line.push(Cell::default());
        }

        let compressed = CompressedLine::compress(&line);

        assert_eq!(compressed.original_length, 80);
        assert_eq!(compressed.segments.len(), 3); // 3 runs
        assert_eq!(compressed.segments[0].count, 20);
        assert_eq!(compressed.segments[1].count, 20);
        assert_eq!(compressed.segments[2].count, 40);

        let decompressed = compressed.decompress();
        assert_eq!(decompressed.len(), 80);
        assert_eq!(decompressed[0].c, 'A');
        assert_eq!(decompressed[0].fg, Some(AnsiColor::Indexed(1)));
        assert_eq!(decompressed[20].c, 'B');
        assert_eq!(decompressed[20].fg, Some(AnsiColor::Indexed(2)));
        assert_eq!(decompressed[40].c, ' ');
    }

    #[test]
    fn test_compression_stats_tracking() {
        let mut stats = CompressionStats::new();

        // Compress several lines
        let line1 = vec![Cell::default(); 80]; // All spaces
        let compressed1 = CompressedLine::compress(&line1);
        stats.update(&compressed1);

        let mut line2 = Vec::new();
        for i in 0..80 {
            let mut cell = Cell::default();
            cell.c = if i % 2 == 0 { 'A' } else { 'B' };
            line2.push(cell);
        }
        let compressed2 = CompressedLine::compress(&line2);
        stats.update(&compressed2);

        assert_eq!(stats.total_lines, 2);
        assert!(stats.total_uncompressed > 0);
        assert!(stats.total_compressed > 0);
        assert!(stats.best_ratio < stats.worst_ratio);
        assert!(stats.avg_ratio > 0.0);
        assert!(stats.space_saved_percent() >= 0.0);
    }

    #[test]
    fn test_scrollback_compression_on_scroll() {
        let mut screen = TerminalScreen::new(80, 24);

        // Fill screen with text
        for i in 0..30 {
            screen.process(format!("Line {}\n", i).as_bytes());
        }

        // Check that scrollback is compressed
        let scrollback_size = screen.scrollback_size();
        assert!(scrollback_size > 0);

        // Get compression stats
        let stats = screen.compression_stats();
        assert_eq!(stats.total_lines, scrollback_size);
        assert!(stats.total_compressed > 0);
        assert!(stats.total_uncompressed > 0);

        // For lines with text, compression should save space
        // (due to trailing spaces)
        assert!(stats.space_saved() > 0);
    }

    #[test]
    fn test_scrollback_compression_roundtrip() {
        let mut screen = TerminalScreen::new(80, 24);

        // Add lines that will go to scrollback
        for i in 0..50 {
            screen.process(format!("Test line {}\n", i).as_bytes());
        }

        // Get all lines (should decompress scrollback)
        let all_lines = screen.get_all_lines();

        // Should have scrollback + visible lines
        let expected_total = screen.scrollback_size() + screen.rows;
        assert_eq!(all_lines.len(), expected_total);

        // Check that first line is readable
        let first_line_text: String = all_lines[0]
            .iter()
            .filter(|c| !c.placeholder)
            .map(|c| c.c)
            .collect::<String>()
            .trim()
            .to_string();

        assert!(first_line_text.contains("Test line"));
    }

    #[test]
    fn test_cells_equal() {
        let cell1 = Cell::default();
        let mut cell2 = Cell::default();

        // Same cells should be equal
        assert!(cells_equal(&cell1, &cell2));

        // Different character
        cell2.c = 'A';
        assert!(!cells_equal(&cell1, &cell2));
        cell2.c = ' ';

        // Different color
        cell2.fg = Some(AnsiColor::Indexed(1));
        assert!(!cells_equal(&cell1, &cell2));
        cell2.fg = None;

        // Different bold
        cell2.bold = true;
        assert!(!cells_equal(&cell1, &cell2));
        cell2.bold = false;

        // Different underline
        cell2.underline = true;
        assert!(!cells_equal(&cell1, &cell2));
    }

    #[test]
    fn test_compression_with_wide_characters() {
        let mut line = Vec::new();

        // Add wide characters (e.g., CJK)
        for _ in 0..10 {
            let mut cell = Cell::default();
            cell.c = '中';
            cell.wide = true;
            line.push(cell);

            // Placeholder for second half
            let mut placeholder = Cell::default();
            placeholder.placeholder = true;
            line.push(placeholder);
        }

        // Fill with spaces
        for _ in 0..60 {
            line.push(Cell::default());
        }

        let compressed = CompressedLine::compress(&line);

        assert_eq!(compressed.original_length, 80);
        // Should compress well: runs of wide chars, placeholders, and spaces
        assert!(compressed.segments.len() < 80);

        let decompressed = compressed.decompress();
        assert_eq!(decompressed.len(), 80);
        assert_eq!(decompressed[0].c, '中');
        assert!(decompressed[0].wide);
        assert!(decompressed[1].placeholder);
    }

    #[test]
    fn test_compression_stats_space_saved() {
        let mut stats = CompressionStats::new();

        // All spaces line - excellent compression
        let line = vec![Cell::default(); 100];
        let compressed = CompressedLine::compress(&line);
        stats.update(&compressed);

        assert!(stats.space_saved() > 0);
        assert!(stats.space_saved_percent() > 50.0); // Should save >50%
    }

    #[test]
    fn test_resize_with_compression() {
        let mut screen = TerminalScreen::new(80, 24);

        // Fill screen
        for i in 0..30 {
            screen.process(format!("Line {}\n", i).as_bytes());
        }

        let scrollback_before = screen.scrollback_size();
        assert!(scrollback_before > 0);

        // Resize smaller (should compress more lines to scrollback)
        screen.resize(80, 20);

        let scrollback_after = screen.scrollback_size();
        assert!(scrollback_after > scrollback_before);

        // Stats should be updated
        let stats = screen.compression_stats();
        assert_eq!(stats.total_lines, scrollback_after);
    }
}

#[cfg(test)]
mod dirty_tracker_tests {
    use super::*;

    #[test]
    fn test_dirty_tracker_new() {
        let tracker = DirtyTracker::new();
        assert!(tracker.needs_full_redraw());
        assert!(tracker.needs_redraw());
        assert!(!tracker.is_cursor_only());
    }

    #[test]
    fn test_mark_line() {
        let mut tracker = DirtyTracker::new();
        tracker.clear();

        assert!(!tracker.needs_redraw());

        tracker.mark_line(5);
        assert!(tracker.needs_redraw());
        assert!(tracker.is_line_dirty(5));
        assert!(!tracker.is_line_dirty(4));
        assert!(!tracker.needs_full_redraw());
        assert!(!tracker.is_cursor_only());
    }

    #[test]
    fn test_mark_range() {
        let mut tracker = DirtyTracker::new();
        tracker.clear();

        tracker.mark_range(10, 15);
        assert!(tracker.needs_redraw());

        for line in 10..=15 {
            assert!(tracker.is_line_dirty(line));
        }
        assert!(!tracker.is_line_dirty(9));
        assert!(!tracker.is_line_dirty(16));
    }

    #[test]
    fn test_mark_all() {
        let mut tracker = DirtyTracker::new();
        tracker.clear();

        tracker.mark_line(5);
        tracker.mark_all();

        assert!(tracker.needs_full_redraw());
        assert!(tracker.needs_redraw());
        assert_eq!(tracker.dirty_count(), usize::MAX);
        assert!(tracker.dirty_lines().is_empty());
    }

    #[test]
    fn test_mark_cursor() {
        let mut tracker = DirtyTracker::new();
        tracker.clear();

        tracker.mark_cursor();
        assert!(tracker.is_cursor_only());
        assert!(tracker.needs_redraw());
        assert!(!tracker.needs_full_redraw());
    }

    #[test]
    fn test_cursor_only_cleared_by_line_mark() {
        let mut tracker = DirtyTracker::new();
        tracker.clear();

        tracker.mark_cursor();
        assert!(tracker.is_cursor_only());

        tracker.mark_line(10);
        assert!(!tracker.is_cursor_only());
        assert!(tracker.needs_redraw());
    }

    #[test]
    fn test_clear() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_line(5);
        tracker.mark_line(10);

        tracker.clear();
        assert!(!tracker.needs_redraw());
        assert!(!tracker.needs_full_redraw());
        assert!(!tracker.is_cursor_only());
        assert_eq!(tracker.dirty_count(), 0);
    }

    #[test]
    fn test_dirty_count() {
        let mut tracker = DirtyTracker::new();
        tracker.clear();

        assert_eq!(tracker.dirty_count(), 0);

        tracker.mark_line(1);
        tracker.mark_line(2);
        tracker.mark_line(3);
        assert_eq!(tracker.dirty_count(), 3);
    }

    #[test]
    fn test_screen_write_marks_dirty() {
        let mut screen = TerminalScreen::new(80, 24);
        screen.clear_dirty();

        assert!(!screen.dirty_tracker().needs_redraw());

        screen.process(b"Hello");
        assert!(screen.dirty_tracker().needs_redraw());
        assert!(screen.dirty_tracker().is_line_dirty(0));
    }

    #[test]
    fn test_screen_scroll_marks_dirty() {
        let mut screen = TerminalScreen::new(80, 24);
        screen.clear_dirty();

        // Fill screen to cause scroll
        for _ in 0..25 {
            screen.process(b"Line\n");
        }

        assert!(screen.dirty_tracker().needs_redraw());
        let dirty_count = screen.dirty_tracker().dirty_count();
        assert!(dirty_count > 0);
    }

    #[test]
    fn test_screen_resize_marks_all_dirty() {
        let mut screen = TerminalScreen::new(80, 24);
        screen.clear_dirty();

        screen.resize(100, 30);
        assert!(screen.dirty_tracker().needs_full_redraw());
    }
}
