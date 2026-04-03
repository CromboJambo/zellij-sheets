//! Layout engine for zellij-sheets.
//!
//! Implements a two-phase Pretext-inspired layout model:
//!
//! **Prepare phase** (`LayoutCache::prepare`) — run once on data load.
//! Walks every cell and measures its display width using `unicode-width`,
//! which handles CJK wide chars, emoji, and zero-width combiners correctly.
//! Results are cached; no DOM, no re-measurement.
//!
//! **Layout phase** (`LayoutEngine::resolve`) — run on every render.
//! Pure arithmetic against the cache. Given the current terminal width,
//! negotiates column widths in two passes and returns a `Vec<ColumnLayout>`.
//! Fast enough to run synchronously on every keypress or resize event.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// ── constants ──────────────────────────────────────────────────────────────

/// Width of the " | " separator between columns.
const SEPARATOR_WIDTH: usize = 3;
/// Absolute floor — columns never shrink below this.
const DEFAULT_MIN_COL_WIDTH: usize = 4;
/// Absolute ceiling — no single column hogs the viewport.
const DEFAULT_MAX_COL_WIDTH: usize = 40;

// ── ColumnMeasure ──────────────────────────────────────────────────────────

/// Measured properties of a single column.
/// Built once during the prepare phase and stored in `LayoutCache`.
#[derive(Debug, Clone)]
pub struct ColumnMeasure {
    /// Display width of the column header.
    pub header_width: usize,
    /// Widest cell value seen in this column.
    pub max_content_width: usize,
    /// Widest unbreakable token (longest single word).
    /// Used as a soft floor during shrinking — we try not to break words.
    pub min_content_width: usize,
}

// ── LayoutCache ────────────────────────────────────────────────────────────

/// Per-column measurements cached after a data load.
/// Invalidated (replaced) whenever `SheetsState::init` is called.
#[derive(Debug, Clone, Default)]
pub struct LayoutCache {
    pub columns: Vec<ColumnMeasure>,
}

impl LayoutCache {
    /// Prepare phase: measure every cell in every column and cache the results.
    ///
    /// O(rows × cols) on load; O(1) per render afterward.
    pub fn prepare(headers: &[String], rows: &[Vec<String>]) -> Self {
        let col_count = headers.len();
        let mut columns = Vec::with_capacity(col_count);

        for col in 0..col_count {
            let header_width = UnicodeWidthStr::width(headers[col].as_str());

            let mut max_content_width: usize = 0;
            let mut min_content_width: usize = 0;

            for row in rows {
                if let Some(cell) = row.get(col) {
                    let cell_w = UnicodeWidthStr::width(cell.as_str());
                    max_content_width = max_content_width.max(cell_w);

                    // Longest unbreakable token in this cell.
                    let min_w = cell
                        .split_whitespace()
                        .map(|word| UnicodeWidthStr::width(word))
                        .max()
                        .unwrap_or(0);
                    min_content_width = min_content_width.max(min_w);
                }
            }

            columns.push(ColumnMeasure {
                header_width,
                max_content_width,
                min_content_width,
            });
        }

        Self { columns }
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    pub fn col_count(&self) -> usize {
        self.columns.len()
    }
}

// ── ColumnLayout ───────────────────────────────────────────────────────────

/// Resolved layout for a single column — the output of the layout phase.
#[derive(Debug, Clone)]
pub struct ColumnLayout {
    /// Zero-based column index.
    pub index: usize,
    /// Final resolved display width in terminal columns.
    pub resolved_width: usize,
    /// True when the column was shrunk below its ideal width.
    /// Lets the renderer decide whether to show a truncation indicator.
    pub truncated: bool,
}

// ── LayoutEngine ───────────────────────────────────────────────────────────

/// Stateless layout engine. Instantiate once; call `resolve` on every render.
pub struct LayoutEngine {
    pub min_col_width: usize,
    pub max_col_width: usize,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            min_col_width: DEFAULT_MIN_COL_WIDTH,
            max_col_width: DEFAULT_MAX_COL_WIDTH,
        }
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bounds(min_col_width: usize, max_col_width: usize) -> Self {
        Self {
            min_col_width,
            max_col_width,
        }
    }

    /// Layout phase: resolve column widths for the given terminal width.
    ///
    /// **Pass 1** — assign ideal widths: `max(header, content)` clamped to
    /// `[min_col_width, max_col_width]`.
    ///
    /// **Pass 2** — if the total exceeds available width, iteratively shed
    /// one cell at a time from the currently-widest shrinkable column,
    /// respecting `min_content_width` as a soft floor before falling back
    /// to `min_col_width`.
    pub fn resolve(&self, cache: &LayoutCache, terminal_width: usize) -> Vec<ColumnLayout> {
        let col_count = cache.col_count();
        if col_count == 0 {
            return Vec::new();
        }

        let separator_budget = SEPARATOR_WIDTH * col_count.saturating_sub(1);
        let available = terminal_width.saturating_sub(separator_budget);

        // Pass 1: ideal widths.
        let mut widths: Vec<usize> = cache
            .columns
            .iter()
            .map(|m| {
                let ideal = m.header_width.max(m.max_content_width);
                ideal.clamp(self.min_col_width, self.max_col_width)
            })
            .collect();

        // Pass 2: shrink if needed.
        if widths.iter().sum::<usize>() > available {
            self.shrink(&mut widths, available, cache);
        }

        // Produce layouts, flagging columns that were shrunk.
        widths
            .iter()
            .enumerate()
            .map(|(i, &w)| {
                let m = &cache.columns[i];
                let ideal = m
                    .header_width
                    .max(m.max_content_width)
                    .clamp(self.min_col_width, self.max_col_width);
                ColumnLayout {
                    index: i,
                    resolved_width: w,
                    truncated: w < ideal,
                }
            })
            .collect()
    }

    /// Iteratively shed width from the widest shrinkable column.
    ///
    /// Soft floor: don't shrink a column below its `min_content_width` until
    /// all other shrinkable columns have also hit their soft floors.
    /// Hard floor: `self.min_col_width`.
    fn shrink(&self, widths: &mut Vec<usize>, available: usize, cache: &LayoutCache) {
        // Two rounds: first respect soft floors, then hard floors.
        for use_soft_floor in [true, false] {
            loop {
                let total: usize = widths.iter().sum();
                if total <= available {
                    return;
                }

                let floor = |i: usize| -> usize {
                    if use_soft_floor {
                        cache.columns[i]
                            .min_content_width
                            .max(self.min_col_width)
                    } else {
                        self.min_col_width
                    }
                };

                let shrinkable: Vec<usize> = widths
                    .iter()
                    .enumerate()
                    .filter(|&(i, &w)| w > floor(i))
                    .map(|(i, _)| i)
                    .collect();

                if shrinkable.is_empty() {
                    break; // move to next round or give up
                }

                // Shed from the widest shrinkable column.
                let widest = shrinkable
                    .iter()
                    .copied()
                    .max_by_key(|&i| widths[i])
                    .unwrap();

                let excess = total - available;
                let room = widths[widest] - floor(widest);
                widths[widest] -= room.min(excess);
            }
        }
    }
}

// ── fit_cell ───────────────────────────────────────────────────────────────

/// Render a cell value into exactly `width` terminal columns.
///
/// - If the value is narrower: right-pad with spaces.
/// - If the value is wider: truncate to `width-1` columns and append `~`.
///
/// Handles multi-byte and wide characters correctly via `unicode-width`.
pub fn fit_cell(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let display_width = UnicodeWidthStr::width(value);

    if display_width <= width {
        let pad = width - display_width;
        let mut s = value.to_string();
        for _ in 0..pad {
            s.push(' ');
        }
        s
    } else {
        // Truncate: fill up to width-1, then add `~`.
        let target = width.saturating_sub(1);
        let mut result = String::new();
        let mut used = 0usize;

        for ch in value.chars() {
            let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
            if used + ch_w > target {
                break;
            }
            result.push(ch);
            used += ch_w;
        }

        // If a wide char left a gap before `target`, fill with spaces.
        while used < target {
            result.push(' ');
            used += 1;
        }

        result.push('~');
        result
    }
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_headers(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    fn make_rows(data: &[&[&str]]) -> Vec<Vec<String>> {
        data.iter()
            .map(|row| row.iter().map(|s| s.to_string()).collect())
            .collect()
    }

    // ── fit_cell ────────────────────────────────────────────────────────

    #[test]
    fn fit_cell_pads_short_value() {
        let result = fit_cell("hi", 6);
        assert_eq!(result, "hi    ");
        assert_eq!(UnicodeWidthStr::width(result.as_str()), 6);
    }

    #[test]
    fn fit_cell_exact_width() {
        let result = fit_cell("hello", 5);
        assert_eq!(result, "hello");
    }

    #[test]
    fn fit_cell_truncates_long_value() {
        let result = fit_cell("hello world", 7);
        assert_eq!(result, "hello ~");
        assert_eq!(UnicodeWidthStr::width(result.as_str()), 7);
    }

    #[test]
    fn fit_cell_handles_wide_chars() {
        // Each CJK char is width 2.
        let result = fit_cell("你好世界", 6);
        // "你好世" = 6 cols — fits exactly, no truncation needed? No — width is 8 > 6.
        // target = 5, "你好" = 4 cols, "世" would make it 6 > 5, so stop at "你好", pad to 5, then ~
        assert_eq!(UnicodeWidthStr::width(result.as_str()), 6);
        assert!(result.ends_with('~'));
    }

    #[test]
    fn fit_cell_zero_width() {
        assert_eq!(fit_cell("hello", 0), "");
    }

    #[test]
    fn fit_cell_empty_value() {
        let result = fit_cell("", 4);
        assert_eq!(result, "    ");
    }

    // ── LayoutCache ─────────────────────────────────────────────────────

    #[test]
    fn cache_prepare_measures_headers() {
        let headers = make_headers(&["Name", "Age", "City"]);
        let rows = make_rows(&[&["Alice", "30", "Minneapolis"]]);
        let cache = LayoutCache::prepare(&headers, &rows);

        assert_eq!(cache.columns[0].header_width, 4); // "Name"
        assert_eq!(cache.columns[1].header_width, 3); // "Age"
        assert_eq!(cache.columns[2].header_width, 4); // "City"
    }

    #[test]
    fn cache_prepare_measures_content() {
        let headers = make_headers(&["Col"]);
        let rows = make_rows(&[&["short"], &["a much longer value"], &["mid"]]);
        let cache = LayoutCache::prepare(&headers, &rows);

        assert_eq!(cache.columns[0].max_content_width, 19); // "a much longer value"
    }

    #[test]
    fn cache_prepare_min_content_width() {
        let headers = make_headers(&["Col"]);
        // "a much longer value" — longest word is "longer" = 6
        let rows = make_rows(&[&["a much longer value"]]);
        let cache = LayoutCache::prepare(&headers, &rows);

        assert_eq!(cache.columns[0].min_content_width, 6);
    }

    #[test]
    fn cache_is_empty_on_default() {
        let cache = LayoutCache::default();
        assert!(cache.is_empty());
    }

    // ── LayoutEngine ────────────────────────────────────────────────────

    #[test]
    fn engine_resolve_empty_cache() {
        let engine = LayoutEngine::new();
        let cache = LayoutCache::default();
        let layouts = engine.resolve(&cache, 80);
        assert!(layouts.is_empty());
    }

    #[test]
    fn engine_resolve_fits_comfortably() {
        let headers = make_headers(&["Name", "Age"]);
        let rows = make_rows(&[&["Alice", "30"]]);
        let cache = LayoutCache::prepare(&headers, &rows);
        let engine = LayoutEngine::new();

        // Wide terminal — no shrinking needed.
        let layouts = engine.resolve(&cache, 120);
        assert_eq!(layouts.len(), 2);
        assert!(!layouts[0].truncated);
        assert!(!layouts[1].truncated);
    }

    #[test]
    fn engine_resolve_shrinks_on_narrow_terminal() {
        let headers = make_headers(&["Description", "Notes"]);
        let rows = make_rows(&[&[
            "This is a very long description that exceeds the terminal width",
            "Also quite long notes here",
        ]]);
        let cache = LayoutCache::prepare(&headers, &rows);
        let engine = LayoutEngine::new();

        let layouts = engine.resolve(&cache, 40);
        let total: usize = layouts.iter().map(|l| l.resolved_width).sum::<usize>()
            + SEPARATOR_WIDTH * (layouts.len().saturating_sub(1));
        assert!(total <= 40, "total={total} should fit in 40 cols");
    }

    #[test]
    fn engine_never_shrinks_below_min() {
        let headers = make_headers(&["A", "B", "C", "D", "E"]);
        let rows = make_rows(&[&["val1", "val2", "val3", "val4", "val5"]]);
        let cache = LayoutCache::prepare(&headers, &rows);
        let engine = LayoutEngine::new();

        // Absurdly narrow terminal.
        let layouts = engine.resolve(&cache, 5);
        for layout in &layouts {
            assert!(
                layout.resolved_width >= DEFAULT_MIN_COL_WIDTH,
                "col {} width {} is below minimum",
                layout.index,
                layout.resolved_width
            );
        }
    }

    #[test]
    fn engine_caps_at_max_col_width() {
        let headers = make_headers(&["Col"]);
        let long_value = "x".repeat(200);
        let rows = make_rows(&[&[long_value.as_str()]]);
        let cache = LayoutCache::prepare(&headers, &rows);
        let engine = LayoutEngine::new();

        let layouts = engine.resolve(&cache, 200);
        assert!(layouts[0].resolved_width <= DEFAULT_MAX_COL_WIDTH);
        assert!(layouts[0].truncated);
    }

    #[test]
    fn engine_resolve_indices_are_correct() {
        let headers = make_headers(&["A", "B", "C"]);
        let rows: Vec<Vec<String>> = vec![];
        let cache = LayoutCache::prepare(&headers, &rows);
        let engine = LayoutEngine::new();

        let layouts = engine.resolve(&cache, 80);
        for (i, layout) in layouts.iter().enumerate() {
            assert_eq!(layout.index, i);
        }
    }
}
