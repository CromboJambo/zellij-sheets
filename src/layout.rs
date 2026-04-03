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

/// Width of the " | " separator between columns.
const SEPARATOR_WIDTH: usize = 3;
/// Absolute floor — columns never shrink below this.
const DEFAULT_MIN_COL_WIDTH: usize = 4;
/// Absolute ceiling — no single column hogs the viewport.
const DEFAULT_MAX_COL_WIDTH: usize = 40;

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

        for (col, header) in headers.iter().enumerate() {
            let header_width = UnicodeWidthStr::width(header.as_str());

            let mut max_content_width: usize = 0;
            let mut min_content_width: usize = 0;

            for row in rows {
                if let Some(cell) = row.get(col) {
                    let cell_w = UnicodeWidthStr::width(cell.as_str());
                    max_content_width = max_content_width.max(cell_w);

                    let min_w = cell
                        .split_whitespace()
                        .map(UnicodeWidthStr::width)
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

/// Resolved layout for a single column — the output of the layout phase.
#[derive(Debug, Clone)]
pub struct ColumnLayout {
    /// Zero-based column index.
    pub index: usize,
    /// Final resolved display width in terminal columns.
    pub resolved_width: usize,
    /// True when the column was shrunk below its ideal width.
    pub truncated: bool,
}

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
    pub fn resolve(&self, cache: &LayoutCache, terminal_width: usize) -> Vec<ColumnLayout> {
        let col_count = cache.col_count();
        if col_count == 0 {
            return Vec::new();
        }

        let separator_budget = SEPARATOR_WIDTH * col_count.saturating_sub(1);
        let available = terminal_width.saturating_sub(separator_budget);

        let mut widths: Vec<usize> = cache
            .columns
            .iter()
            .map(|m| {
                let ideal = m.header_width.max(m.max_content_width);
                ideal.clamp(self.min_col_width, self.max_col_width)
            })
            .collect();

        if widths.iter().sum::<usize>() > available {
            self.shrink(&mut widths, available, cache);
        }

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
    fn shrink(&self, widths: &mut [usize], available: usize, cache: &LayoutCache) {
        for use_soft_floor in [true, false] {
            loop {
                let total: usize = widths.iter().sum();
                if total <= available {
                    return;
                }

                let floor = |i: usize| -> usize {
                    if use_soft_floor {
                        cache.columns[i].min_content_width.max(self.min_col_width)
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
                    break;
                }

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

/// Render a cell value into exactly `width` terminal columns.
pub fn fit_cell(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let display_width = UnicodeWidthStr::width(value);
    if display_width <= width {
        let pad = width - display_width;
        let mut s = value.to_string();
        s.push_str(&" ".repeat(pad));
        return s;
    }

    if width == 1 {
        return "~".to_string();
    }

    let target = width - 1;
    let mut out = String::new();
    let mut used = 0;

    for ch in value.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + ch_width > target {
            break;
        }
        out.push(ch);
        used += ch_width;
    }

    out.push('~');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_headers() -> Vec<String> {
        vec!["Name".into(), "Description".into(), "Count".into()]
    }

    fn make_rows() -> Vec<Vec<String>> {
        vec![
            vec!["apple".into(), "small red fruit".into(), "10".into()],
            vec!["banana".into(), "long yellow fruit".into(), "200".into()],
        ]
    }

    #[test]
    fn fit_cell_pads_short_value() {
        assert_eq!(fit_cell("abc", 5), "abc  ");
    }

    #[test]
    fn fit_cell_exact_width() {
        assert_eq!(fit_cell("abcd", 4), "abcd");
    }

    #[test]
    fn fit_cell_truncates_long_value() {
        assert_eq!(fit_cell("abcdef", 4), "abc~");
    }

    #[test]
    fn fit_cell_handles_wide_chars() {
        assert_eq!(fit_cell("表計算", 5), "表計~");
    }

    #[test]
    fn fit_cell_zero_width() {
        assert_eq!(fit_cell("abc", 0), "");
    }

    #[test]
    fn fit_cell_empty_value() {
        assert_eq!(fit_cell("", 3), "   ");
    }

    #[test]
    fn cache_prepare_measures_headers() {
        let cache = LayoutCache::prepare(&make_headers(), &[]);
        assert_eq!(cache.columns[0].header_width, 4);
        assert_eq!(cache.columns[1].header_width, 11);
    }

    #[test]
    fn cache_prepare_measures_content() {
        let cache = LayoutCache::prepare(&make_headers(), &make_rows());
        assert_eq!(cache.columns[0].max_content_width, 6);
        assert_eq!(cache.columns[1].max_content_width, 17);
    }

    #[test]
    fn cache_prepare_min_content_width() {
        let cache = LayoutCache::prepare(&make_headers(), &make_rows());
        assert_eq!(cache.columns[1].min_content_width, 6);
    }

    #[test]
    fn cache_is_empty_on_default() {
        assert!(LayoutCache::default().is_empty());
    }

    #[test]
    fn engine_resolve_empty_cache() {
        let engine = LayoutEngine::new();
        assert!(engine.resolve(&LayoutCache::default(), 80).is_empty());
    }

    #[test]
    fn engine_resolve_fits_comfortably() {
        let cache = LayoutCache::prepare(&make_headers(), &make_rows());
        let engine = LayoutEngine::new();
        let layouts = engine.resolve(&cache, 80);

        assert_eq!(layouts.len(), 3);
        assert!(layouts.iter().all(|layout| !layout.truncated));
    }

    #[test]
    fn engine_resolve_shrinks_on_narrow_terminal() {
        let cache = LayoutCache::prepare(&make_headers(), &make_rows());
        let engine = LayoutEngine::new();
        let layouts = engine.resolve(&cache, 20);

        assert_eq!(layouts.len(), 3);
        assert!(layouts.iter().any(|layout| layout.truncated));
    }

    #[test]
    fn engine_never_shrinks_below_min() {
        let cache = LayoutCache::prepare(&make_headers(), &make_rows());
        let engine = LayoutEngine::with_bounds(4, 40);
        let layouts = engine.resolve(&cache, 5);

        assert!(layouts.iter().all(|layout| layout.resolved_width >= 4));
    }

    #[test]
    fn engine_caps_at_max_col_width() {
        let headers = vec!["Description".to_string()];
        let rows = vec![vec!["x".repeat(120)]];
        let cache = LayoutCache::prepare(&headers, &rows);
        let engine = LayoutEngine::with_bounds(4, 20);
        let layouts = engine.resolve(&cache, 80);

        assert_eq!(layouts[0].resolved_width, 20);
        assert!(!layouts[0].truncated);
    }

    #[test]
    fn engine_resolve_indices_are_correct() {
        let cache = LayoutCache::prepare(&make_headers(), &make_rows());
        let engine = LayoutEngine::new();
        let layouts = engine.resolve(&cache, 80);

        assert_eq!(layouts[0].index, 0);
        assert_eq!(layouts[1].index, 1);
        assert_eq!(layouts[2].index, 2);
    }
}
