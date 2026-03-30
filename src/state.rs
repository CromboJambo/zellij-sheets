use crate::config::SheetsConfig;
use crate::data_loader::{load_data, LoadedData};
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

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub message: String,
    pub timestamp: SystemTime,
    pub level: StatusLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Number,
    Boolean,
    Empty,
    String,
}

#[derive(Clone)]
pub struct SheetsState {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    scroll_row: usize,
    selected_row: usize,
    max_scroll_row: usize,
    file_name: String,
    width: usize,
    height: usize,
    config: Arc<SheetsConfig>,
    view_mode: ViewMode,
    sort_column: Option<String>,
    sort_direction: SortDirection,
    filter_expr: Option<String>,
    search_query: Option<String>,
    file_path: Option<PathBuf>,
    file_mod_time: Option<SystemTime>,
    last_error: Option<String>,
    status_messages: Vec<StatusMessage>,
    show_row_numbers: bool,
    show_column_numbers: bool,
    show_grid_lines: bool,
    show_data_types: bool,
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
            max_scroll_row: 0,
            file_name: String::new(),
            width: 80,
            height: 24,
            config,
            view_mode: ViewMode::Grid,
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            filter_expr: None,
            search_query: None,
            file_path: None,
            file_mod_time: None,
            last_error: None,
            status_messages: Vec::new(),
            show_row_numbers: false,
            show_column_numbers: true,
            show_grid_lines: true,
            show_data_types: false,
        }
    }

    pub fn init(&mut self, data: LoadedData) -> Result<()> {
        self.headers = data.headers;
        self.rows = data.rows;
        self.selected_row = 0;
        self.scroll_row = 0;
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
        self.add_status_message(
            StatusMessage {
                message: format!("Loaded {}", path.display()),
                timestamp: SystemTime::now(),
                level: StatusLevel::Success,
            },
            5,
        );
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

    pub fn scroll_left(&mut self) {}

    pub fn scroll_right(&mut self) {}

    pub fn page_up(&mut self) {
        let page_size = self.config.behavior.page_size.max(1);
        self.scroll_row = self.scroll_row.saturating_sub(page_size);
        self.selected_row = self.selected_row.saturating_sub(page_size);
    }

    pub fn page_down(&mut self) {
        let page_size = self.config.behavior.page_size.max(1);
        self.scroll_row = (self.scroll_row + page_size).min(self.max_scroll_row);
        self.selected_row = (self.selected_row + page_size).min(self.last_row_index());
    }

    pub fn go_to_top(&mut self) {
        self.scroll_row = 0;
        self.selected_row = 0;
    }

    pub fn go_to_bottom(&mut self) {
        self.scroll_row = self.max_scroll_row;
        self.selected_row = self.last_row_index();
    }

    pub fn select_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
            if self.selected_row < self.scroll_row {
                self.scroll_row = self.selected_row;
            }
        }
    }

    pub fn select_down(&mut self) {
        if self.selected_row < self.last_row_index() {
            self.selected_row += 1;
            if self.selected_row >= self.scroll_row + self.visible_rows() {
                self.scroll_row = self
                    .selected_row
                    .saturating_sub(self.visible_rows().saturating_sub(1));
            }
        }
    }

    pub fn quit(&mut self) {
        self.add_status_message(
            StatusMessage {
                message: "Exiting".to_string(),
                timestamp: SystemTime::now(),
                level: StatusLevel::Info,
            },
            1,
        );
    }

    pub fn scroll_row(&self) -> usize {
        self.scroll_row
    }

    pub fn selected_row(&self) -> usize {
        self.selected_row
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

    pub fn visible_rows(&self) -> usize {
        self.height.saturating_sub(5).max(1)
    }

    pub fn row_range(&self) -> (usize, usize) {
        let start = self.scroll_row;
        let end = (start + self.visible_rows()).min(self.row_count());
        (start, end)
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

    pub fn add_status_message(&mut self, message: StatusMessage, duration_secs: u64) {
        self.status_messages.push(message);
        self.status_messages.retain(|message| {
            message
                .timestamp
                .elapsed()
                .map(|elapsed| elapsed.as_secs() < duration_secs)
                .unwrap_or(true)
        });
    }

    pub fn get_status_messages(&self) -> Result<Vec<StatusMessage>> {
        Ok(self.status_messages.clone())
    }

    pub fn clear_status_messages(&mut self) {
        self.status_messages.clear();
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    pub fn get_view_mode(&self) -> Result<ViewMode> {
        Ok(self.view_mode.clone())
    }

    pub fn set_search_query(&mut self, query: Option<String>) {
        self.search_query = query;
    }

    pub fn get_search_query(&self) -> Result<Option<String>> {
        Ok(self.search_query.clone())
    }

    pub fn set_filter_expr(&mut self, expr: Option<String>) {
        self.filter_expr = expr;
    }

    pub fn get_filter_expr(&self) -> Result<Option<String>> {
        Ok(self.filter_expr.clone())
    }

    pub fn set_sort(&mut self, column: Option<String>, direction: SortDirection) {
        self.sort_column = column;
        self.sort_direction = direction;
    }

    pub fn get_sort_column(&self) -> Result<Option<String>> {
        Ok(self.sort_column.clone())
    }

    pub fn get_sort_direction(&self) -> Result<SortDirection> {
        Ok(self.sort_direction.clone())
    }

    pub fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }

    pub fn get_file_path(&self) -> Result<Option<PathBuf>> {
        Ok(self.file_path.clone())
    }

    pub fn set_file_mod_time(&mut self, time: Option<SystemTime>) {
        self.file_mod_time = time;
    }

    pub fn get_file_mod_time(&self) -> Result<Option<SystemTime>> {
        Ok(self.file_mod_time)
    }

    pub fn get_column_names(&self) -> Result<Vec<String>> {
        Ok(self.headers.clone())
    }

    pub fn get_row_count(&self) -> Result<usize> {
        Ok(self.row_count())
    }

    pub fn get_column_count(&self) -> Result<usize> {
        Ok(self.col_count())
    }

    pub fn get_selected_row(&self) -> Result<usize> {
        Ok(self.selected_row)
    }

    pub fn get_row_range(&self) -> Result<(usize, usize)> {
        Ok(self.row_range())
    }

    pub fn get_width(&self) -> Result<usize> {
        Ok(self.width)
    }

    pub fn get_height(&self) -> Result<usize> {
        Ok(self.height)
    }

    pub fn get_file_name(&self) -> Result<String> {
        Ok(self.file_name().to_string())
    }

    pub fn get_config(&self) -> Result<SheetsConfig> {
        Ok((*self.config).clone())
    }

    pub fn set_config(&mut self, config: SheetsConfig) {
        self.config = Arc::new(config);
    }

    pub fn get_last_error(&self) -> Result<Option<String>> {
        Ok(self.last_error.clone())
    }

    pub fn set_last_error(&mut self, error: Option<String>) {
        self.last_error = error;
    }

    pub fn clear_last_error(&mut self) {
        self.last_error = None;
    }

    pub fn set_show_row_numbers(&mut self, show: bool) {
        self.show_row_numbers = show;
    }

    pub fn get_show_row_numbers(&self) -> Result<bool> {
        Ok(self.show_row_numbers)
    }

    pub fn set_show_column_numbers(&mut self, show: bool) {
        self.show_column_numbers = show;
    }

    pub fn get_show_column_numbers(&self) -> Result<bool> {
        Ok(self.show_column_numbers)
    }

    pub fn set_show_grid_lines(&mut self, show: bool) {
        self.show_grid_lines = show;
    }

    pub fn get_show_grid_lines(&self) -> Result<bool> {
        Ok(self.show_grid_lines)
    }

    pub fn set_show_data_types(&mut self, show: bool) {
        self.show_data_types = show;
    }

    pub fn get_show_data_types(&self) -> Result<bool> {
        Ok(self.show_data_types)
    }

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

    fn sync_bounds(&mut self) {
        self.max_scroll_row = self.row_count().saturating_sub(self.visible_rows());
        self.scroll_row = self.scroll_row.min(self.max_scroll_row);
        self.selected_row = self.selected_row.min(self.last_row_index());
    }

    fn last_row_index(&self) -> usize {
        self.row_count().saturating_sub(1)
    }
}

pub fn serialize_state(_state: &SheetsState) -> Result<String> {
    Err(StateError::StateError(
        "state serialization is not implemented".to_string(),
    ))
}

pub fn deserialize_state(_json: &str) -> Result<SheetsState> {
    Err(StateError::StateError(
        "state deserialization is not implemented".to_string(),
    ))
}

pub fn save_state(_state: &SheetsState, _path: &PathBuf) -> Result<()> {
    Err(StateError::StateError(
        "saving state is not implemented".to_string(),
    ))
}

pub fn load_state(_path: &PathBuf) -> Result<SheetsState> {
    Err(StateError::StateError(
        "loading state is not implemented".to_string(),
    ))
}

fn infer_data_type(value: &str) -> DataType {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return DataType::Empty;
    }
    if trimmed.eq_ignore_ascii_case("true") || trimmed.eq_ignore_ascii_case("false") {
        return DataType::Boolean;
    }
    if trimmed.parse::<i64>().is_ok() || trimmed.parse::<f64>().is_ok() {
        return DataType::Number;
    }
    DataType::String
}
