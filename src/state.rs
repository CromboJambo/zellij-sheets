//! Core state management module for the spreadsheet viewer
//!
//! This module provides the `SheetsState` struct which manages all aspects of
//! spreadsheet data, display configuration, and user interaction state.
//!
//! ## Data Model
//!
//! The state stores:
//! - Headers and rows from the loaded spreadsheet
//! - Scroll position and selection state
//! - User preferences (view mode, colors, etc.)
//! - File metadata and modification tracking
//!
//! ## State Management
//!
//! The state is designed to be:
//! - Serializable for persistence across sessions
//! - Thread-safe for concurrent access
//! - Efficient for large datasets

use crate::config::SheetsConfig;
use crate::data_loader::{load_data, LoadedData};
use crate::layout::LayoutCache;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Data loading error: {0}")]
    DataLoadError(#[from] crate::data_loader::DataLoaderError),

    #[error("State error: {0}")]
    StateError(String),
}

pub type Result<T> = std::result::Result<T, StateError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    List,
    Compact,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub message: String,
    /// Epoch seconds at creation time, used for expiry checks.
    pub timestamp: SystemTime,
    pub level: StatusLevel,
    /// How long this message should be displayed, in seconds.
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Number,
    Boolean,
    Empty,
    String,
}

/// Serializable snapshot of `SheetsState`, excluding runtime-only fields
/// (`Arc<SheetsConfig>`, `StatusMessage`s, and `SystemTime`s) that cannot
/// round-trip through serde without custom impls.
#[derive(Clone, Serialize, Deserialize)]
struct SheetsStateSnapshot {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    scroll_row: usize,
    selected_row: usize,
    selected_col: usize,
    col_offset: usize,
    max_scroll_row: usize,
    max_col_offset: usize,
    file_name: String,
    width: usize,
    height: usize,
    view_mode: ViewMode,
    sort_column: Option<String>,
    sort_direction: SortDirection,
    filter_expr: Option<String>,
    search_query: Option<String>,
    search_active: bool,
    search_direction: SearchDirection,
    file_path: Option<PathBuf>,
    last_error: Option<String>,
}

#[derive(Clone)]
pub struct SheetsState {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    scroll_row: usize,
    selected_row: usize,
    selected_col: usize,
    col_offset: usize,
    max_scroll_row: usize,
    max_col_offset: usize,
    file_name: String,
    width: usize,
    height: usize,
    config: Arc<SheetsConfig>,
    view_mode: ViewMode,
    sort_column: Option<String>,
    sort_direction: SortDirection,
    filter_expr: Option<String>,
    search_query: Option<String>,
    search_active: bool,
    search_direction: SearchDirection,
    file_path: Option<PathBuf>,
    file_mod_time: Option<SystemTime>,
    last_error: Option<String>,
    status_messages: Vec<StatusMessage>,
    pub layout_cache: LayoutCache,
}

impl Default for SheetsState {
    fn default() -> Self {
        Self::new(Arc::new(SheetsConfig::default()))
    }
}

impl SheetsState {
    pub fn new(config: Arc<SheetsConfig>) -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            scroll_row: 0,
            selected_row: 0,
            selected_col: 0,
            col_offset: 0,
            max_scroll_row: 0,
            max_col_offset: 0,
            file_name: String::new(),
            width: 80,
            height: 24,
            config,
            view_mode: ViewMode::Grid,
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            filter_expr: None,
            search_query: None,
            search_active: false,
            search_direction: SearchDirection::Forward,
            file_path: None,
            file_mod_time: None,
            last_error: None,
            status_messages: Vec::new(),
            layout_cache: LayoutCache::default(),
        }
    }

    pub fn init(&mut self, data: LoadedData) -> Result<()> {
        self.headers = data.headers;
        self.rows = data.rows;
        self.selected_row = 0;
        self.selected_col = 0;
        self.scroll_row = 0;
        self.col_offset = 0;
        self.layout_cache = LayoutCache::prepare(&self.headers, &self.rows);
        self.sync_bounds();
        Ok(())
    }

    pub fn load_file(&mut self, path: PathBuf) -> Result<()> {
        let data = load_data(&path)?;
        self.file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();
        self.file_mod_time = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        self.file_path = Some(path.clone());
        self.init(data)?;
        self.add_status_message(StatusMessage {
            message: format!("Loaded {}", path.display()),
            timestamp: SystemTime::now(),
            level: StatusLevel::Success,
            duration_secs: 5,
        });
        Ok(())
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width.max(20);
        self.height = height.max(8);
        self.sync_bounds();
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_row > 0 {
            self.scroll_row -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_row < self.max_scroll_row {
            self.scroll_row += 1;
        }
    }

    pub fn scroll_left(&mut self) {
        if self.col_offset > 0 {
            self.col_offset -= 1;
            self.selected_col = self.selected_col.max(self.col_offset);
            self.selected_col = self.selected_col.min(self.last_visible_col());
        }
    }

    pub fn scroll_right(&mut self) {
        if self.col_offset < self.max_col_offset {
            self.col_offset += 1;
            self.selected_col = self.selected_col.max(self.col_offset);
            self.selected_col = self.selected_col.min(self.last_visible_col());
        }
    }

    pub fn page_up(&mut self) {
        let page_size = self.config.behavior.page_size.max(1);
        self.scroll_row = self.scroll_row.saturating_sub(page_size);
        self.selected_row = self.selected_row.saturating_sub(page_size);
        self.adjust_scroll_for_selection();
    }

    pub fn page_down(&mut self) {
        let page_size = self.config.behavior.page_size.max(1);
        self.scroll_row = (self.scroll_row + page_size).min(self.max_scroll_row);
        self.selected_row = (self.selected_row + page_size).min(self.last_row_index());
        self.adjust_scroll_for_selection();
    }

    pub fn half_page_up(&mut self) {
        let page_size = (self.visible_rows() / 2).max(1);
        self.scroll_row = self.scroll_row.saturating_sub(page_size);
        self.selected_row = self.selected_row.saturating_sub(page_size);
        self.adjust_scroll_for_selection();
    }

    pub fn half_page_down(&mut self) {
        let page_size = (self.visible_rows() / 2).max(1);
        self.scroll_row = (self.scroll_row + page_size).min(self.max_scroll_row);
        self.selected_row = (self.selected_row + page_size).min(self.last_row_index());
        self.adjust_scroll_for_selection();
    }

    pub fn go_to_top(&mut self) {
        self.scroll_row = 0;
        self.selected_row = 0;
        self.adjust_scroll_for_selection();
    }

    pub fn go_to_bottom(&mut self) {
        self.scroll_row = self.max_scroll_row;
        self.selected_row = self.last_row_index();
        self.adjust_scroll_for_selection();
    }

    pub fn go_to_first_col(&mut self) {
        self.selected_col = 0;
        self.adjust_scroll_for_selection();
    }

    pub fn go_to_last_col(&mut self) {
        self.selected_col = self.last_col_index();
        self.adjust_scroll_for_selection();
    }

    pub fn go_to_top_visible(&mut self) {
        self.selected_row = self.scroll_row.min(self.last_row_index());
        self.adjust_scroll_for_selection();
    }

    pub fn go_to_middle_visible(&mut self) {
        let (start, end) = self.row_range();
        let visible_len = end.saturating_sub(start);
        if visible_len == 0 {
            self.selected_row = 0;
        } else {
            self.selected_row = start + ((visible_len - 1) / 2);
        }
        self.adjust_scroll_for_selection();
    }

    pub fn go_to_bottom_visible(&mut self) {
        let (_, end) = self.row_range();
        self.selected_row = end.saturating_sub(1);
        self.adjust_scroll_for_selection();
    }

    pub fn select_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
            self.adjust_scroll_for_selection();
        }
    }

    pub fn select_down(&mut self) {
        if self.selected_row < self.last_row_index() {
            self.selected_row += 1;
            self.adjust_scroll_for_selection();
        }
    }

    pub fn select_left(&mut self) {
        if self.selected_col > 0 {
            self.selected_col -= 1;
            self.adjust_scroll_for_selection();
        }
    }

    pub fn select_right(&mut self) {
        if self.selected_col < self.last_col_index() {
            self.selected_col += 1;
            self.adjust_scroll_for_selection();
        }
    }

    pub fn quit(&mut self) {
        self.add_status_message(StatusMessage {
            message: "Exiting".to_string(),
            timestamp: SystemTime::now(),
            level: StatusLevel::Info,
            duration_secs: 1,
        });
    }

    // -------------------------------------------------------------------------
    // Plain (infallible) accessors
    // -------------------------------------------------------------------------

    pub fn scroll_row(&self) -> usize {
        self.scroll_row
    }

    pub fn selected_row(&self) -> usize {
        self.selected_row
    }

    pub fn selected_col(&self) -> usize {
        self.selected_col
    }

    pub fn col_offset(&self) -> usize {
        self.col_offset
    }

    pub fn max_col_offset(&self) -> usize {
        self.max_col_offset
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn col_count(&self) -> usize {
        self.headers.len()
    }

    pub fn headers(&self) -> Option<&Vec<String>> {
        (!self.headers.is_empty()).then_some(&self.headers)
    }

    pub fn file_name(&self) -> &str {
        if self.file_name.is_empty() {
            "No file loaded"
        } else {
            &self.file_name
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn config(&self) -> &SheetsConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: SheetsConfig) {
        self.config = Arc::new(config);
    }

    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    pub fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }

    pub fn file_mod_time(&self) -> Option<SystemTime> {
        self.file_mod_time
    }

    pub fn set_file_mod_time(&mut self, time: Option<SystemTime>) {
        self.file_mod_time = time;
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn set_last_error(&mut self, error: Option<String>) {
        self.last_error = error;
    }

    pub fn clear_last_error(&mut self) {
        self.last_error = None;
    }

    /// Whether to show row numbers. Reads from `DisplayConfig`.
    pub fn show_row_numbers(&self) -> bool {
        self.config.display.show_row_numbers
    }

    /// Whether to show column numbers. Reads from `DisplayConfig`.
    pub fn show_column_numbers(&self) -> bool {
        self.config.display.show_column_numbers
    }

    /// Whether to show data type indicators. Reads from `DisplayConfig`.
    pub fn show_data_types(&self) -> bool {
        self.config.display.show_data_types
    }

    pub fn filter_expr(&self) -> Option<&str> {
        self.filter_expr.as_deref()
    }

    pub fn set_filter_expr(&mut self, expr: Option<String>) {
        self.filter_expr = expr;
    }

    pub fn sort_column(&self) -> Option<&str> {
        self.sort_column.as_deref()
    }

    pub fn sort_direction(&self) -> &SortDirection {
        &self.sort_direction
    }

    pub fn set_sort(&mut self, column: Option<String>, direction: SortDirection) {
        self.sort_column = column;
        self.sort_direction = direction;
    }

    // -------------------------------------------------------------------------
    // Display helpers
    // -------------------------------------------------------------------------

    pub fn visible_rows(&self) -> usize {
        self.height.saturating_sub(5).max(1)
    }

    pub fn visible_cols(&self) -> usize {
        self.visible_cols_from_offset(self.col_offset)
    }

    pub fn visible_cols_from_offset(&self, offset: usize) -> usize {
        if self.col_count() == 0 || offset >= self.col_count() {
            return 0;
        }

        let layouts = crate::layout::LayoutEngine::new().resolve(&self.layout_cache, self.width);
        let mut used_width = 0;
        let mut visible_cols = 0;

        for layout in layouts.iter().skip(offset) {
            let separator_width = usize::from(visible_cols > 0) * 3;
            let next_width = used_width + separator_width + layout.resolved_width;
            if next_width > self.width {
                break;
            }
            used_width = next_width;
            visible_cols += 1;
        }

        visible_cols.max(1)
    }

    pub fn row_range(&self) -> (usize, usize) {
        let start = self.scroll_row;
        let end = (start + self.visible_rows()).min(self.row_count());
        (start, end)
    }

    pub fn get_row_range(&self) -> (usize, usize) {
        self.row_range()
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<String> {
        self.rows.get(row)?.get(col).cloned()
    }

    pub fn get_row(&self, row: usize) -> Option<Vec<String>> {
        self.rows.get(row).cloned()
    }

    pub fn get_data_type(&self, col: usize) -> Option<DataType> {
        if col >= self.col_count() {
            return None;
        }

        self.rows
            .iter()
            .filter_map(|row| row.get(col))
            .find(|value| !value.trim().is_empty())
            .map(|value| infer_data_type(value))
            .or(Some(DataType::Empty))
    }

    pub fn at_top(&self) -> bool {
        self.scroll_row == 0
    }

    pub fn at_bottom(&self) -> bool {
        self.scroll_row >= self.max_scroll_row
    }

    // -------------------------------------------------------------------------
    // Status messages
    // -------------------------------------------------------------------------

    pub fn add_status_message(&mut self, message: StatusMessage) {
        self.status_messages.push(message);
        // Expire messages using each message's own duration, not a shared one.
        self.status_messages.retain(|msg| {
            msg.timestamp
                .elapsed()
                .map(|elapsed| elapsed.as_secs() < msg.duration_secs)
                .unwrap_or(true)
        });
    }

    pub fn status_messages(&self) -> &[StatusMessage] {
        &self.status_messages
    }

    pub fn clear_status_messages(&mut self) {
        self.status_messages.clear();
    }

    // -------------------------------------------------------------------------
    // View mode
    // -------------------------------------------------------------------------

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    pub fn get_view_mode(&self) -> ViewMode {
        self.view_mode.clone()
    }

    // -------------------------------------------------------------------------
    // Search
    // -------------------------------------------------------------------------

    pub fn set_search_query(&mut self, query: Option<String>) {
        self.search_query = query;
    }

    pub fn get_search_query(&self) -> Option<&str> {
        self.search_query.as_deref()
    }

    pub fn is_search_active(&self) -> bool {
        self.search_active
    }

    pub fn search_direction(&self) -> SearchDirection {
        self.search_direction
    }

    pub fn begin_search(&mut self, direction: SearchDirection) {
        self.search_active = true;
        self.search_direction = direction;
        self.search_query = None;
    }

    pub fn search_append(&mut self, ch: char) {
        if !self.search_active {
            return;
        }

        self.search_query.get_or_insert_with(String::new).push(ch);
    }

    pub fn search_backspace(&mut self) {
        if !self.search_active {
            return;
        }

        if let Some(query) = &mut self.search_query {
            query.pop();
        }
    }

    pub fn search_commit(&mut self) -> bool {
        self.search_active = false;

        if self.search_query.as_deref().unwrap_or_default().is_empty() {
            self.search_query = None;
            return false;
        }

        match self.search_direction {
            SearchDirection::Forward => self.search_next(),
            SearchDirection::Backward => self.search_prev(),
        }
    }

    pub fn search_cancel(&mut self) {
        self.search_active = false;
        self.search_query = None;
    }

    pub fn search_next(&mut self) -> bool {
        self.search_direction = SearchDirection::Forward;
        self.find_and_select_match(SearchDirection::Forward)
    }

    pub fn search_prev(&mut self) -> bool {
        self.search_direction = SearchDirection::Backward;
        self.find_and_select_match(SearchDirection::Backward)
    }

    // -------------------------------------------------------------------------
    // Serialization
    // -------------------------------------------------------------------------

    /// Serialize the current state to a JSON snapshot string.
    ///
    /// Runtime-only fields (`config`, `status_messages`, `file_mod_time`) are
    /// excluded; they cannot round-trip through serde without custom impls.
    pub fn to_snapshot_json(&self) -> Result<String> {
        let snapshot = SheetsStateSnapshot {
            headers: self.headers.clone(),
            rows: self.rows.clone(),
            scroll_row: self.scroll_row,
            selected_row: self.selected_row,
            selected_col: self.selected_col,
            col_offset: self.col_offset,
            max_scroll_row: self.max_scroll_row,
            max_col_offset: self.max_col_offset,
            file_name: self.file_name.clone(),
            width: self.width,
            height: self.height,
            view_mode: self.view_mode.clone(),
            sort_column: self.sort_column.clone(),
            sort_direction: self.sort_direction.clone(),
            filter_expr: self.filter_expr.clone(),
            search_query: self.search_query.clone(),
            search_active: self.search_active,
            search_direction: self.search_direction,
            file_path: self.file_path.clone(),
            last_error: self.last_error.clone(),
        };
        serde_json::to_string(&snapshot)
            .map_err(|e| StateError::StateError(format!("serialization error: {e}")))
    }

    /// Restore state from a JSON snapshot string produced by [`to_snapshot_json`].
    ///
    /// The `config` is taken from the caller; `status_messages` and
    /// `file_mod_time` are reset to empty/`None`.
    pub fn from_snapshot_json(json: &str, config: Arc<SheetsConfig>) -> Result<Self> {
        let snapshot: SheetsStateSnapshot = serde_json::from_str(json)
            .map_err(|e| StateError::StateError(format!("deserialization error: {e}")))?;

        let layout_cache = LayoutCache::prepare(&snapshot.headers, &snapshot.rows);
        Ok(Self {
            headers: snapshot.headers,
            rows: snapshot.rows,
            scroll_row: snapshot.scroll_row,
            selected_row: snapshot.selected_row,
            selected_col: snapshot.selected_col,
            col_offset: snapshot.col_offset,
            max_scroll_row: snapshot.max_scroll_row,
            max_col_offset: snapshot.max_col_offset,
            file_name: snapshot.file_name,
            width: snapshot.width,
            height: snapshot.height,
            config,
            view_mode: snapshot.view_mode,
            sort_column: snapshot.sort_column,
            sort_direction: snapshot.sort_direction,
            filter_expr: snapshot.filter_expr,
            search_query: snapshot.search_query,
            search_active: snapshot.search_active,
            search_direction: snapshot.search_direction,
            file_path: snapshot.file_path,
            file_mod_time: None,
            last_error: snapshot.last_error,
            status_messages: Vec::new(),
            layout_cache,
        })
    }

    // -------------------------------------------------------------------------
    // I/O-backed accessor (legitimately fallible)
    // -------------------------------------------------------------------------

    pub fn is_file_modified(&self) -> Result<bool> {
        let Some(path) = self.file_path.as_ref() else {
            return Ok(false);
        };
        let Some(last_mod_time) = self.file_mod_time else {
            return Ok(false);
        };
        let current_mod_time = std::fs::metadata(path).and_then(|m| m.modified())?;
        Ok(current_mod_time > last_mod_time)
    }

    // -------------------------------------------------------------------------
    // Private helpers
    // -------------------------------------------------------------------------

    fn sync_bounds(&mut self) {
        self.max_scroll_row = self.row_count().saturating_sub(self.visible_rows());
        self.max_col_offset = self
            .col_count()
            .saturating_sub(self.visible_cols_from_offset(0));
        self.scroll_row = self.scroll_row.min(self.max_scroll_row);
        self.col_offset = self.col_offset.min(self.max_col_offset);
        self.selected_row = self.selected_row.min(self.last_row_index());
        self.selected_col = self.selected_col.min(self.last_col_index());
        self.adjust_scroll_for_selection();
    }

    fn last_row_index(&self) -> usize {
        self.row_count().saturating_sub(1)
    }

    fn last_col_index(&self) -> usize {
        self.col_count().saturating_sub(1)
    }

    fn last_visible_col(&self) -> usize {
        self.col_offset
            .saturating_add(self.visible_cols().saturating_sub(1))
            .min(self.last_col_index())
    }

    fn adjust_scroll_for_selection(&mut self) {
        if self.selected_row < self.scroll_row {
            self.scroll_row = self.selected_row;
        } else if self.selected_row >= self.scroll_row + self.visible_rows() {
            self.scroll_row = self
                .selected_row
                .saturating_sub(self.visible_rows().saturating_sub(1));
        }

        if self.selected_col < self.col_offset {
            self.col_offset = self.selected_col;
        } else {
            while self.selected_col > self.last_visible_col()
                && self.col_offset < self.max_col_offset
            {
                self.col_offset += 1;
            }
        }

        self.scroll_row = self.scroll_row.min(self.max_scroll_row);
        self.col_offset = self.col_offset.min(self.max_col_offset);
    }

    fn find_and_select_match(&mut self, direction: SearchDirection) -> bool {
        let Some(query) = self.search_query.as_deref() else {
            return false;
        };
        if query.is_empty() || self.row_count() == 0 || self.col_count() == 0 {
            return false;
        }

        let row_count = self.row_count();
        let col_count = self.col_count();
        let total_cells = row_count * col_count;
        if total_cells == 0 {
            return false;
        }

        let start_index = self.selected_row * col_count + self.selected_col;
        for step in 1..=total_cells {
            let index = match direction {
                SearchDirection::Forward => (start_index + step) % total_cells,
                SearchDirection::Backward => {
                    (start_index + total_cells - (step % total_cells)) % total_cells
                }
            };
            let row = index / col_count;
            let col = index % col_count;

            if self
                .rows
                .get(row)
                .and_then(|values| values.get(col))
                .is_some_and(|value| cell_matches_query(value, query))
            {
                self.selected_row = row;
                self.selected_col = col;
                self.adjust_scroll_for_selection();
                return true;
            }
        }

        self.add_status_message(StatusMessage {
            message: format!("Pattern not found: {query}"),
            timestamp: SystemTime::now(),
            level: StatusLevel::Warning,
            duration_secs: 3,
        });
        false
    }
}

// -----------------------------------------------------------------------------
// Free functions
// -----------------------------------------------------------------------------

pub fn cell_matches_query(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(&query.to_lowercase())
}

fn infer_data_type(value: &str) -> DataType {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return DataType::Empty;
    }
    if trimmed.parse::<f64>().is_ok() {
        return DataType::Number;
    }
    if matches!(trimmed.to_lowercase().as_str(), "true" | "false") {
        return DataType::Boolean;
    }
    DataType::String
}
