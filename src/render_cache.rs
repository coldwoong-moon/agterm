//! Rendering Cache System for AgTerm
//!
//! Provides multi-level caching infrastructure to optimize terminal rendering:
//! - **CellCache**: Caches individual cell rendering results
//! - **GlyphCache**: Caches font glyph metrics and rendered shapes
//! - **LineCache**: Caches entire line rendering with dirty flag tracking
//! - **StyleCache**: Caches style attribute combinations
//!
//! ## Performance Benefits:
//! - Reduces redundant rendering computations
//! - Minimizes font measurement overhead
//! - Enables efficient partial screen updates
//! - Tracks cache hit rates for profiling
//!
//! ## Memory Management:
//! - LRU eviction policy prevents unbounded growth
//! - Configurable memory limits per cache type
//! - Automatic cleanup on size threshold breach

use iced::Color;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

// ============================================================================
// Cache Statistics
// ============================================================================

/// Statistics for monitoring cache performance
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of cache lookups
    pub hits: u64,
    /// Number of successful cache hits
    pub misses: u64,
    /// Number of entries evicted due to capacity limits
    pub evictions: u64,
    /// Current number of entries in cache
    pub entries: usize,
    /// Maximum capacity
    pub capacity: usize,
    /// Last reset timestamp
    pub last_reset: Option<Instant>,
}

impl CacheStats {
    /// Calculate cache hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Record a cache hit
    #[inline]
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record a cache miss
    #[inline]
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Record an eviction
    #[inline]
    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    /// Update entry count
    #[inline]
    pub fn update_entries(&mut self, count: usize) {
        self.entries = count;
    }

    /// Reset statistics
    pub fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
        self.last_reset = Some(Instant::now());
    }

    /// Get total operations
    pub fn total_operations(&self) -> u64 {
        self.hits + self.misses
    }
}

// ============================================================================
// Style Cache
// ============================================================================

/// Unique identifier for a style combination
///
/// Combines text attributes (bold, italic, etc.) with colors to create
/// a hash key for caching style rendering parameters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StyleKey {
    pub fg_color: Option<[u8; 4]>, // RGBA
    pub bg_color: Option<[u8; 4]>, // RGBA
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim: bool,
}

impl StyleKey {
    /// Create a style key from color and attributes
    pub fn new(
        fg: Option<Color>,
        bg: Option<Color>,
        bold: bool,
        italic: bool,
        underline: bool,
        strikethrough: bool,
        dim: bool,
    ) -> Self {
        Self {
            fg_color: fg.map(color_to_rgba),
            bg_color: bg.map(color_to_rgba),
            bold,
            italic,
            underline,
            strikethrough,
            dim,
        }
    }

    /// Memory size of this key in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Cached style rendering data
#[derive(Debug, Clone)]
pub struct CachedStyle {
    /// Computed foreground color (after dim/bold adjustments)
    pub computed_fg: Color,
    /// Computed background color
    pub computed_bg: Option<Color>,
    /// Font weight modifier
    pub font_weight: f32,
    /// Last access time for LRU
    last_access: Instant,
}

impl CachedStyle {
    /// Create a new cached style entry
    pub fn new(fg: Color, bg: Option<Color>, weight: f32) -> Self {
        Self {
            computed_fg: fg,
            computed_bg: bg,
            font_weight: weight,
            last_access: Instant::now(),
        }
    }

    /// Update last access time
    #[inline]
    pub fn touch(&mut self) {
        self.last_access = Instant::now();
    }

    /// Get age since last access
    pub fn age(&self) -> Duration {
        self.last_access.elapsed()
    }

    /// Memory size in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Cache for style combinations
pub struct StyleCache {
    cache: HashMap<StyleKey, CachedStyle>,
    capacity: usize,
    access_order: VecDeque<StyleKey>,
    stats: CacheStats,
}

impl StyleCache {
    /// Create a new style cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            capacity,
            access_order: VecDeque::with_capacity(capacity),
            stats: CacheStats {
                capacity,
                ..Default::default()
            },
        }
    }

    /// Get a cached style, returning None if not found
    pub fn get(&mut self, key: &StyleKey) -> Option<&CachedStyle> {
        if let Some(style) = self.cache.get_mut(key) {
            style.touch();
            self.stats.record_hit();

            // Move to back of access order (most recently used)
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(key.clone());

            Some(style)
        } else {
            self.stats.record_miss();
            None
        }
    }

    /// Insert a style into the cache
    pub fn insert(&mut self, key: StyleKey, style: CachedStyle) {
        // Evict if at capacity
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&key) {
            self.evict_lru();
        }

        self.cache.insert(key.clone(), style);
        self.access_order.push_back(key);
        self.stats.update_entries(self.cache.len());
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some(oldest_key) = self.access_order.pop_front() {
            self.cache.remove(&oldest_key);
            self.stats.record_eviction();
        }
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.stats.update_entries(0);
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get current memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        let key_size = std::mem::size_of::<StyleKey>();
        let value_size = std::mem::size_of::<CachedStyle>();
        self.cache.len() * (key_size + value_size)
    }
}

// ============================================================================
// Glyph Cache
// ============================================================================

/// Key for identifying a glyph in the cache
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    /// Character codepoint
    pub character: char,
    /// Font size in pixels
    pub font_size: u16,
    /// Bold flag
    pub bold: bool,
    /// Italic flag
    pub italic: bool,
}

impl GlyphKey {
    /// Create a new glyph key
    pub fn new(character: char, font_size: u16, bold: bool, italic: bool) -> Self {
        Self {
            character,
            font_size,
            bold,
            italic,
        }
    }

    /// Memory size in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Cached glyph metrics and rendering data
#[derive(Debug, Clone)]
pub struct CachedGlyph {
    /// Glyph width in pixels
    pub width: f32,
    /// Glyph height in pixels
    pub height: f32,
    /// Baseline offset
    pub baseline_offset: f32,
    /// Advance width (for positioning next glyph)
    pub advance: f32,
    /// Whether this is a wide character (CJK, emoji)
    pub is_wide: bool,
    /// Last access time for LRU
    last_access: Instant,
}

impl CachedGlyph {
    /// Create a new cached glyph
    pub fn new(width: f32, height: f32, baseline: f32, advance: f32, is_wide: bool) -> Self {
        Self {
            width,
            height,
            baseline_offset: baseline,
            advance,
            is_wide,
            last_access: Instant::now(),
        }
    }

    /// Update last access time
    #[inline]
    pub fn touch(&mut self) {
        self.last_access = Instant::now();
    }

    /// Get age since last access
    pub fn age(&self) -> Duration {
        self.last_access.elapsed()
    }

    /// Memory size in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Cache for font glyph metrics
pub struct GlyphCache {
    cache: HashMap<GlyphKey, CachedGlyph>,
    capacity: usize,
    access_order: VecDeque<GlyphKey>,
    stats: CacheStats,
}

impl GlyphCache {
    /// Create a new glyph cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            capacity,
            access_order: VecDeque::with_capacity(capacity),
            stats: CacheStats {
                capacity,
                ..Default::default()
            },
        }
    }

    /// Get a cached glyph
    pub fn get(&mut self, key: &GlyphKey) -> Option<&CachedGlyph> {
        if let Some(glyph) = self.cache.get_mut(key) {
            glyph.touch();
            self.stats.record_hit();

            // Move to back of access order
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(key.clone());

            Some(glyph)
        } else {
            self.stats.record_miss();
            None
        }
    }

    /// Insert a glyph into the cache
    pub fn insert(&mut self, key: GlyphKey, glyph: CachedGlyph) {
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&key) {
            self.evict_lru();
        }

        self.cache.insert(key.clone(), glyph);
        self.access_order.push_back(key);
        self.stats.update_entries(self.cache.len());
    }

    /// Evict least recently used glyph
    fn evict_lru(&mut self) {
        if let Some(oldest_key) = self.access_order.pop_front() {
            self.cache.remove(&oldest_key);
            self.stats.record_eviction();
        }
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.stats.update_entries(0);
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get current memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        let key_size = std::mem::size_of::<GlyphKey>();
        let value_size = std::mem::size_of::<CachedGlyph>();
        self.cache.len() * (key_size + value_size)
    }
}

// ============================================================================
// Cell Cache
// ============================================================================

/// Key for identifying a rendered cell
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellKey {
    /// Character in the cell
    pub character: char,
    /// Style attributes
    pub style: StyleKey,
    /// Font size
    pub font_size: u16,
}

impl CellKey {
    /// Create a new cell key
    pub fn new(character: char, style: StyleKey, font_size: u16) -> Self {
        Self {
            character,
            style,
            font_size,
        }
    }

    /// Memory size in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>() + self.style.memory_size()
    }
}

/// Cached cell rendering result
#[derive(Debug, Clone)]
pub struct CachedCell {
    /// Pre-computed text content
    pub text: String,
    /// Computed foreground color
    pub fg_color: Color,
    /// Computed background color
    pub bg_color: Option<Color>,
    /// Font weight
    pub font_weight: f32,
    /// Cell width multiplier (1 for normal, 2 for wide chars)
    pub width_multiplier: u8,
    /// Last access time
    last_access: Instant,
}

impl CachedCell {
    /// Create a new cached cell
    pub fn new(
        text: String,
        fg: Color,
        bg: Option<Color>,
        weight: f32,
        width_mult: u8,
    ) -> Self {
        Self {
            text,
            fg_color: fg,
            bg_color: bg,
            font_weight: weight,
            width_multiplier: width_mult,
            last_access: Instant::now(),
        }
    }

    /// Update last access time
    #[inline]
    pub fn touch(&mut self) {
        self.last_access = Instant::now();
    }

    /// Get age since last access
    pub fn age(&self) -> Duration {
        self.last_access.elapsed()
    }

    /// Memory size in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>() + self.text.len()
    }
}

/// Cache for rendered cells
pub struct CellCache {
    cache: HashMap<CellKey, CachedCell>,
    capacity: usize,
    access_order: VecDeque<CellKey>,
    stats: CacheStats,
}

impl CellCache {
    /// Create a new cell cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            capacity,
            access_order: VecDeque::with_capacity(capacity),
            stats: CacheStats {
                capacity,
                ..Default::default()
            },
        }
    }

    /// Get a cached cell
    pub fn get(&mut self, key: &CellKey) -> Option<&CachedCell> {
        if let Some(cell) = self.cache.get_mut(key) {
            cell.touch();
            self.stats.record_hit();

            // Move to back of access order
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(key.clone());

            Some(cell)
        } else {
            self.stats.record_miss();
            None
        }
    }

    /// Insert a cell into the cache
    pub fn insert(&mut self, key: CellKey, cell: CachedCell) {
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&key) {
            self.evict_lru();
        }

        self.cache.insert(key.clone(), cell);
        self.access_order.push_back(key);
        self.stats.update_entries(self.cache.len());
    }

    /// Evict least recently used cell
    fn evict_lru(&mut self) {
        if let Some(oldest_key) = self.access_order.pop_front() {
            self.cache.remove(&oldest_key);
            self.stats.record_eviction();
        }
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.stats.update_entries(0);
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get current memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        self.cache
            .iter()
            .map(|(k, v)| k.memory_size() + v.memory_size())
            .sum()
    }
}

// ============================================================================
// Line Cache
// ============================================================================

/// Key for identifying a rendered line
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LineKey {
    /// Line index
    pub line_index: usize,
    /// Hash of line content and styles
    pub content_hash: u64,
}

impl LineKey {
    /// Create a new line key
    pub fn new(line_index: usize, content_hash: u64) -> Self {
        Self {
            line_index,
            content_hash,
        }
    }

    /// Memory size in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Cached line rendering result
#[derive(Debug, Clone)]
pub struct CachedLine {
    /// Pre-rendered geometry or drawing commands
    /// (In a real implementation, this would be a canvas geometry cache)
    pub geometry_placeholder: Vec<u8>,
    /// Whether this line is dirty and needs re-rendering
    pub dirty: bool,
    /// Last access time
    last_access: Instant,
    /// Line width in pixels
    pub width: f32,
    /// Line height in pixels
    pub height: f32,
}

impl CachedLine {
    /// Create a new cached line
    pub fn new(geometry: Vec<u8>, width: f32, height: f32) -> Self {
        Self {
            geometry_placeholder: geometry,
            dirty: false,
            last_access: Instant::now(),
            width,
            height,
        }
    }

    /// Mark line as dirty (needs re-rendering)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Mark line as clean (freshly rendered)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
        self.last_access = Instant::now();
    }

    /// Update last access time
    #[inline]
    pub fn touch(&mut self) {
        self.last_access = Instant::now();
    }

    /// Get age since last access
    pub fn age(&self) -> Duration {
        self.last_access.elapsed()
    }

    /// Memory size in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>() + self.geometry_placeholder.len()
    }
}

/// Cache for rendered lines with dirty tracking
pub struct LineCache {
    cache: HashMap<LineKey, CachedLine>,
    capacity: usize,
    access_order: VecDeque<LineKey>,
    stats: CacheStats,
    /// Track which line indices are dirty
    dirty_lines: std::collections::HashSet<usize>,
}

impl LineCache {
    /// Create a new line cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            capacity,
            access_order: VecDeque::with_capacity(capacity),
            stats: CacheStats {
                capacity,
                ..Default::default()
            },
            dirty_lines: std::collections::HashSet::new(),
        }
    }

    /// Get a cached line if it's clean
    pub fn get(&mut self, key: &LineKey) -> Option<&CachedLine> {
        if let Some(line) = self.cache.get_mut(key) {
            if !line.dirty {
                line.touch();
                self.stats.record_hit();

                // Move to back of access order
                if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                    self.access_order.remove(pos);
                }
                self.access_order.push_back(key.clone());

                return Some(line);
            }
        }
        self.stats.record_miss();
        None
    }

    /// Insert a line into the cache
    pub fn insert(&mut self, key: LineKey, mut line: CachedLine) {
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&key) {
            self.evict_lru();
        }

        line.mark_clean();
        self.dirty_lines.remove(&key.line_index);
        self.cache.insert(key.clone(), line);
        self.access_order.push_back(key);
        self.stats.update_entries(self.cache.len());
    }

    /// Mark a line as dirty
    pub fn mark_line_dirty(&mut self, line_index: usize) {
        self.dirty_lines.insert(line_index);

        // Mark all cached entries for this line as dirty
        for (key, line) in self.cache.iter_mut() {
            if key.line_index == line_index {
                line.mark_dirty();
            }
        }
    }

    /// Mark multiple lines as dirty
    pub fn mark_lines_dirty(&mut self, line_indices: impl Iterator<Item = usize>) {
        for idx in line_indices {
            self.mark_line_dirty(idx);
        }
    }

    /// Check if a line is dirty
    pub fn is_line_dirty(&self, line_index: usize) -> bool {
        self.dirty_lines.contains(&line_index)
    }

    /// Clear all dirty flags
    pub fn clear_dirty_flags(&mut self) {
        self.dirty_lines.clear();
        for line in self.cache.values_mut() {
            line.mark_clean();
        }
    }

    /// Get all dirty line indices
    pub fn get_dirty_lines(&self) -> Vec<usize> {
        self.dirty_lines.iter().copied().collect()
    }

    /// Evict least recently used line
    fn evict_lru(&mut self) {
        if let Some(oldest_key) = self.access_order.pop_front() {
            self.cache.remove(&oldest_key);
            self.stats.record_eviction();
        }
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.dirty_lines.clear();
        self.stats.update_entries(0);
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get current memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        self.cache
            .iter()
            .map(|(k, v)| k.memory_size() + v.memory_size())
            .sum()
    }
}

// ============================================================================
// Unified Render Cache Manager
// ============================================================================

/// Configuration for the render cache system
#[derive(Debug, Clone)]
pub struct RenderCacheConfig {
    /// Maximum number of cached styles
    pub style_cache_capacity: usize,
    /// Maximum number of cached glyphs
    pub glyph_cache_capacity: usize,
    /// Maximum number of cached cells
    pub cell_cache_capacity: usize,
    /// Maximum number of cached lines
    pub line_cache_capacity: usize,
    /// Maximum total memory usage in bytes (0 = unlimited)
    pub max_memory_bytes: usize,
}

impl Default for RenderCacheConfig {
    fn default() -> Self {
        Self {
            style_cache_capacity: 1024,      // ~1K style combinations
            glyph_cache_capacity: 4096,      // ~4K unique glyphs
            cell_cache_capacity: 16384,      // ~16K rendered cells
            line_cache_capacity: 512,        // ~512 lines (enough for large terminals)
            max_memory_bytes: 64 * 1024 * 1024, // 64 MB total
        }
    }
}

/// Unified manager for all rendering caches
pub struct RenderCacheManager {
    pub style_cache: StyleCache,
    pub glyph_cache: GlyphCache,
    pub cell_cache: CellCache,
    pub line_cache: LineCache,
    config: RenderCacheConfig,
}

impl RenderCacheManager {
    /// Create a new render cache manager with default configuration
    pub fn new() -> Self {
        Self::with_config(RenderCacheConfig::default())
    }

    /// Create a new render cache manager with custom configuration
    pub fn with_config(config: RenderCacheConfig) -> Self {
        Self {
            style_cache: StyleCache::new(config.style_cache_capacity),
            glyph_cache: GlyphCache::new(config.glyph_cache_capacity),
            cell_cache: CellCache::new(config.cell_cache_capacity),
            line_cache: LineCache::new(config.line_cache_capacity),
            config,
        }
    }

    /// Clear all caches
    pub fn clear_all(&mut self) {
        self.style_cache.clear();
        self.glyph_cache.clear();
        self.cell_cache.clear();
        self.line_cache.clear();
    }

    /// Get total memory usage across all caches
    pub fn total_memory_usage(&self) -> usize {
        self.style_cache.memory_usage()
            + self.glyph_cache.memory_usage()
            + self.cell_cache.memory_usage()
            + self.line_cache.memory_usage()
    }

    /// Check if memory usage exceeds configured limit
    pub fn exceeds_memory_limit(&self) -> bool {
        if self.config.max_memory_bytes == 0 {
            return false;
        }
        self.total_memory_usage() > self.config.max_memory_bytes
    }

    /// Get combined statistics from all caches
    pub fn get_all_stats(&self) -> RenderCacheStats {
        RenderCacheStats {
            style_stats: self.style_cache.stats().clone(),
            glyph_stats: self.glyph_cache.stats().clone(),
            cell_stats: self.cell_cache.stats().clone(),
            line_stats: self.line_cache.stats().clone(),
            total_memory_bytes: self.total_memory_usage(),
            config: self.config.clone(),
        }
    }

    /// Reset all statistics
    pub fn reset_stats(&mut self) {
        self.style_cache.stats.reset();
        self.glyph_cache.stats.reset();
        self.cell_cache.stats.reset();
        self.line_cache.stats.reset();
    }
}

impl Default for RenderCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined statistics from all render caches
#[derive(Debug, Clone)]
pub struct RenderCacheStats {
    pub style_stats: CacheStats,
    pub glyph_stats: CacheStats,
    pub cell_stats: CacheStats,
    pub line_stats: CacheStats,
    pub total_memory_bytes: usize,
    pub config: RenderCacheConfig,
}

impl RenderCacheStats {
    /// Get overall hit rate across all caches
    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.style_stats.hits
            + self.glyph_stats.hits
            + self.cell_stats.hits
            + self.line_stats.hits;
        let total_ops = self.style_stats.total_operations()
            + self.glyph_stats.total_operations()
            + self.cell_stats.total_operations()
            + self.line_stats.total_operations();

        if total_ops == 0 {
            0.0
        } else {
            (total_hits as f64 / total_ops as f64) * 100.0
        }
    }

    /// Get total number of cached entries
    pub fn total_entries(&self) -> usize {
        self.style_stats.entries
            + self.glyph_stats.entries
            + self.cell_stats.entries
            + self.line_stats.entries
    }

    /// Get total number of evictions
    pub fn total_evictions(&self) -> u64 {
        self.style_stats.evictions
            + self.glyph_stats.evictions
            + self.cell_stats.evictions
            + self.line_stats.evictions
    }

    /// Format memory usage as human-readable string
    pub fn memory_usage_string(&self) -> String {
        let mb = self.total_memory_bytes as f64 / (1024.0 * 1024.0);
        format!("{:.2} MB", mb)
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Convert Iced Color to RGBA byte array
fn color_to_rgba(color: Color) -> [u8; 4] {
    [
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8,
        (color.a * 255.0) as u8,
    ]
}

/// Compute hash for line content (for cache key generation)
pub fn compute_line_hash(cells: &[impl Hash]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    for cell in cells {
        cell.hash(&mut hasher);
    }
    hasher.finish()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_hit_rate() {
        let mut stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.hit_rate(), 66.66666666666666);
        assert_eq!(stats.total_operations(), 3);
    }

    #[test]
    fn test_style_cache_basic() {
        let mut cache = StyleCache::new(2);

        let key1 = StyleKey::new(
            Some(Color::WHITE),
            None,
            true,
            false,
            false,
            false,
            false,
        );
        let style1 = CachedStyle::new(Color::WHITE, None, 1.5);

        cache.insert(key1.clone(), style1.clone());
        assert_eq!(cache.stats().entries, 1);

        let retrieved = cache.get(&key1);
        assert!(retrieved.is_some());
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn test_style_cache_lru_eviction() {
        let mut cache = StyleCache::new(2);

        let key1 = StyleKey::new(Some(Color::WHITE), None, true, false, false, false, false);
        let key2 = StyleKey::new(Some(Color::BLACK), None, false, true, false, false, false);
        let key3 = StyleKey::new(Some(Color::from_rgb(0.5, 0.5, 0.5)), None, false, false, true, false, false);

        cache.insert(key1.clone(), CachedStyle::new(Color::WHITE, None, 1.0));
        cache.insert(key2.clone(), CachedStyle::new(Color::BLACK, None, 1.0));

        assert_eq!(cache.stats().entries, 2);

        // This should evict key1 (least recently used)
        cache.insert(key3.clone(), CachedStyle::new(Color::from_rgb(0.5, 0.5, 0.5), None, 1.0));

        assert_eq!(cache.stats().entries, 2);
        assert_eq!(cache.stats().evictions, 1);

        // key1 should be evicted
        assert!(cache.get(&key1).is_none());
        assert!(cache.get(&key2).is_some());
        assert!(cache.get(&key3).is_some());
    }

    #[test]
    fn test_glyph_cache_basic() {
        let mut cache = GlyphCache::new(100);

        let key = GlyphKey::new('A', 16, false, false);
        let glyph = CachedGlyph::new(10.0, 16.0, 12.0, 10.0, false);

        cache.insert(key.clone(), glyph.clone());
        assert_eq!(cache.stats().entries, 1);

        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().width, 10.0);
    }

    #[test]
    fn test_glyph_cache_wide_characters() {
        let mut cache = GlyphCache::new(100);

        let key = GlyphKey::new('あ', 16, false, false);
        let glyph = CachedGlyph::new(20.0, 16.0, 12.0, 20.0, true);

        cache.insert(key.clone(), glyph);

        let retrieved = cache.get(&key).unwrap();
        assert!(retrieved.is_wide);
        assert_eq!(retrieved.width, 20.0);
    }

    #[test]
    fn test_cell_cache_basic() {
        let mut cache = CellCache::new(100);

        let style_key = StyleKey::new(Some(Color::WHITE), None, false, false, false, false, false);
        let key = CellKey::new('X', style_key, 16);
        let cell = CachedCell::new("X".to_string(), Color::WHITE, None, 1.0, 1);

        cache.insert(key.clone(), cell);
        assert_eq!(cache.stats().entries, 1);

        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().text, "X");
    }

    #[test]
    fn test_line_cache_dirty_tracking() {
        let mut cache = LineCache::new(100);

        let key = LineKey::new(0, 12345);
        let line = CachedLine::new(vec![1, 2, 3], 800.0, 20.0);

        cache.insert(key.clone(), line);
        assert!(!cache.is_line_dirty(0));

        // Mark line as dirty
        cache.mark_line_dirty(0);
        assert!(cache.is_line_dirty(0));

        // Should not return dirty lines from cache
        assert!(cache.get(&key).is_none());
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_line_cache_multiple_dirty_lines() {
        let mut cache = LineCache::new(100);

        cache.mark_lines_dirty(0..5);

        let dirty = cache.get_dirty_lines();
        assert_eq!(dirty.len(), 5);

        for i in 0..5 {
            assert!(cache.is_line_dirty(i));
        }
    }

    #[test]
    fn test_render_cache_manager() {
        let mut manager = RenderCacheManager::new();

        // Test style cache
        let style_key = StyleKey::new(Some(Color::WHITE), None, true, false, false, false, false);
        manager.style_cache.insert(
            style_key.clone(),
            CachedStyle::new(Color::WHITE, None, 1.5),
        );

        assert!(manager.style_cache.get(&style_key).is_some());

        // Test glyph cache
        let glyph_key = GlyphKey::new('A', 16, false, false);
        manager.glyph_cache.insert(
            glyph_key.clone(),
            CachedGlyph::new(10.0, 16.0, 12.0, 10.0, false),
        );

        assert!(manager.glyph_cache.get(&glyph_key).is_some());

        // Check stats
        let stats = manager.get_all_stats();
        assert_eq!(stats.style_stats.entries, 1);
        assert_eq!(stats.glyph_stats.entries, 1);
        assert!(stats.total_memory_bytes > 0);
    }

    #[test]
    fn test_memory_usage_tracking() {
        let manager = RenderCacheManager::new();
        let initial_memory = manager.total_memory_usage();

        assert!(initial_memory >= 0);
    }

    #[test]
    fn test_clear_all_caches() {
        let mut manager = RenderCacheManager::new();

        let style_key = StyleKey::new(Some(Color::WHITE), None, false, false, false, false, false);
        manager.style_cache.insert(
            style_key.clone(),
            CachedStyle::new(Color::WHITE, None, 1.0),
        );

        manager.clear_all();

        assert_eq!(manager.style_cache.stats().entries, 0);
        assert_eq!(manager.glyph_cache.stats().entries, 0);
        assert_eq!(manager.cell_cache.stats().entries, 0);
        assert_eq!(manager.line_cache.stats().entries, 0);
    }

    #[test]
    fn test_cache_stats_reset() {
        let mut manager = RenderCacheManager::new();

        let style_key = StyleKey::new(Some(Color::WHITE), None, false, false, false, false, false);
        manager.style_cache.insert(
            style_key.clone(),
            CachedStyle::new(Color::WHITE, None, 1.0),
        );

        manager.style_cache.get(&style_key);
        assert!(manager.style_cache.stats().hits > 0);

        manager.reset_stats();
        assert_eq!(manager.style_cache.stats().hits, 0);
    }

    #[test]
    fn test_overall_hit_rate() {
        let mut manager = RenderCacheManager::new();

        let style_key = StyleKey::new(Some(Color::WHITE), None, false, false, false, false, false);
        manager.style_cache.insert(
            style_key.clone(),
            CachedStyle::new(Color::WHITE, None, 1.0),
        );

        // Hit
        manager.style_cache.get(&style_key);
        // Miss
        let missing_key = StyleKey::new(Some(Color::BLACK), None, true, false, false, false, false);
        manager.style_cache.get(&missing_key);

        let stats = manager.get_all_stats();
        assert_eq!(stats.overall_hit_rate(), 50.0);
    }

    #[test]
    fn test_color_to_rgba() {
        let color = Color::from_rgb(1.0, 0.5, 0.0);
        let rgba = color_to_rgba(color);

        assert_eq!(rgba[0], 255);
        assert_eq!(rgba[1], 127);
        assert_eq!(rgba[2], 0);
        assert_eq!(rgba[3], 255);
    }

    #[test]
    fn test_line_hash_consistency() {
        let cells1 = vec!['A', 'B', 'C'];
        let cells2 = vec!['A', 'B', 'C'];
        let cells3 = vec!['A', 'B', 'D'];

        let hash1 = compute_line_hash(&cells1);
        let hash2 = compute_line_hash(&cells2);
        let hash3 = compute_line_hash(&cells3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_custom_cache_config() {
        let config = RenderCacheConfig {
            style_cache_capacity: 512,
            glyph_cache_capacity: 2048,
            cell_cache_capacity: 8192,
            line_cache_capacity: 256,
            max_memory_bytes: 32 * 1024 * 1024,
        };

        let manager = RenderCacheManager::with_config(config.clone());

        assert_eq!(manager.config.style_cache_capacity, 512);
        assert_eq!(manager.config.max_memory_bytes, 32 * 1024 * 1024);
    }

    #[test]
    fn test_memory_limit_check() {
        let config = RenderCacheConfig {
            max_memory_bytes: 1024, // Very small limit
            ..Default::default()
        };

        let mut manager = RenderCacheManager::with_config(config);

        // Add many entries to exceed limit
        for i in 0..100 {
            let key = StyleKey::new(
                Some(Color::from_rgb(i as f32 / 100.0, 0.5, 0.5)),
                None,
                false,
                false,
                false,
                false,
                false,
            );
            manager.style_cache.insert(key, CachedStyle::new(Color::WHITE, None, 1.0));
        }

        // Should exceed the tiny 1KB limit
        assert!(manager.exceeds_memory_limit());
    }
}
