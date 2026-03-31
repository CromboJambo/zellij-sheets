// Zellij Sheets - State Management Module
// Handles application state, data management, and navigation

use crate::config::SheetsConfig;
use crate::data_loader::{load_data, LoadedData};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use thiserror::Error;

/// Error types for state management
#[derive(Debug, Error)]
pub enum StateError {
   #[error("IO error: {0}")]
   IoError(#[from] std::io::Error),

   #[error("Data loading error: {0}")]
   DataLoadError(#[from] crate::data_loader::DataLoaderError),

   #[error("Polars error: {0}")]
   PolarsError(#[from] PolarsError),

   #[error("State error: {0}")]
   StateError(String),
}

/// Result type for state operations
pub type Result<T> = std::result::Result<T, StateError>;

/// Main application state structure
pub struct SheetsState {
   /// Configuration
   config: Arc<SheetsConfig>,

    /// Data frame with spreadsheet data
    data_frame: Arc<RwLock<DataFrame>>,

    /// Column names
    column_names: Arc<RwLock<Vec<String>>>,

    /// Row count
    row_count: Arc<RwLock<usize>>,

    /// Selected row index
    selected_row: Arc<RwLock<usize>>,

    /// Scroll offset (number of rows to skip)
    scroll_offset: Arc<RwLock<usize>>,

    /// Current file path
    file_path: Arc<RwLock<Option<PathBuf>>>,

    /// File modification time
    file_mod_time: Arc<RwLock<Option<SystemTime>>>,

    /// View mode (list, grid, compact)
    view_mode: Arc<RwLock<ViewMode>>,

    /// Sort column and direction
    sort_column: Arc<RwLock<Option<String>>>,
    sort_direction: Arc<RwLock<SortDirection>>,

    /// Filter expression
    filter_expr: Arc<RwLock<Option<String>>>,

    /// Search query
    search_query: Arc<RwLock<Option<String>>>,

    /// Last error message
    last_error: Arc<RwLock<Option<String>>>,

    /// Status messages
    status_messages: Arc<RwLock<Vec<StatusMessage>>>,

    /// View settings
    show_row_numbers: Arc<RwLock<bool>>,
    show_column_numbers: Arc<RwLock<bool>>,
    show_grid_lines: Arc<RwLock<bool>>,
    show_data_types: Arc<RwLock<bool>>,
}

/// View mode enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViewMode {
    /// Full grid view
    Grid,
    /// Compact list view
    List,
    /// Compact grid view
    Compact,
    /// Raw data view
    Raw,
}

/// Sort direction enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Status message structure
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub message: String,
    pub timestamp: SystemTime,
    pub level: StatusLevel,
}

/// Status level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Default configuration for SheetsState
impl Default for SheetsState {
    fn default() -> Self {
        Self::new(Arc::new(SheetsConfig::default()))
    }
}

impl SheetsState {
    /// Create a new state instance
    pub fn new(config: Arc<SheetsConfig>) -> Self {
        Self {
            config,
            data_frame: Arc::new(RwLock::new(DataFrame::default())),
            column_names: Arc::new(RwLock::new(vec![])),
            row_count: Arc::new(RwLock::new(0)),
            selected_row: Arc::new(RwLock::new(0)),
            scroll_offset: Arc::new(RwLock::new(0)),
            file_path: Arc::new(RwLock::new(None)),
            file_mod_time: Arc::new(RwLock::new(None)),
            view_mode: Arc::new(RwLock::new(ViewMode::Grid)),
            sort_column: Arc::new(RwLock::new(None)),
            sort_direction: Arc::new(RwLock::new(SortDirection::Ascending)),
            filter_expr: Arc::new(RwLock::new(None)),
            search_query: Arc::new(RwLock::new(None)),
            last_error: Arc::new(RwLock::new(None)),
            status_messages: Arc::new(RwLock::new(vec![])),
            show_row_numbers: Arc::new(RwLock::new(false)),
            show_column_numbers: Arc::new(RwLock::new(false)),
            show_grid_lines: Arc::new(RwLock::new(true)),
            show_data_types: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize state with data
    pub fn init(&self, data: DataFrame, headers: Vec<String>) -> Result<()> {
        let mut df = data.clone();
        let mut headers = headers;

        // Apply any filters
        if let Some(expr) = self.get_filter_expr()? {
            df = df.filter(&expr)?;
        }

        // Apply search filter
        if let Some(query) = self.get_search_query()? {
            df = self.apply_search_filter(df, &query)?;
        }

        // Apply sorting
        if let Some(col) = self.get_sort_column()? {
            df = self.apply_sort(df, &col, self.get_sort_direction()?)?;
        }

        // Update state
        let mut column_names = self.column_names.write().unwrap();
        let mut row_count = self.row_count.write().unwrap();
        let mut data_frame = self.data_frame.write().unwrap();

        *column_names = headers;
        *row_count = df.height();
        *data_frame = df;

        Ok(())
    }

    /// Load data from file
    pub fn load_file(&self, path: PathBuf) -> Result<()> {
        let data = load_data(&path)?;
        let headers = data
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        self.init(data, headers)?;

        let mut file_path = self.file_path.write().unwrap();
        let mut file_mod_time = self.file_mod_time.write().unwrap();

        *file_path = Some(path.clone());
        *file_mod_time = std::fs::metadata(&path).map(|m| m.modified()).ok();

        self.add_status_message(
            StatusMessage {
                message: format!("Loaded: {}", path.display()),
                timestamp: SystemTime::now(),
                level: StatusLevel::Success,
            },
            5,
        );

        Ok(())
    }

    /// Apply search filter to data frame
    fn apply_search_filter(&self, df: DataFrame, query: &str) -> Result<DataFrame> {
        let lower_query = query.to_lowercase();

        let columns = df.get_column_names();
        let mut filter_exprs = Vec::new();

        for col_name in columns {
            if let Ok(col) = df.column(col_name) {
                match col.dtype() {
                    DataType::String => {
                        if let Ok(strings) = col.str() {
                            let mask = strings
                                .iter()
                                .map(|s| s.to_lowercase().contains(&lower_query))
                                .collect::<Vec<bool>>();

                            if let Ok(expr) =
                                BooleanChunked::from_iter(mask).into_series().into_frame()
                            {
                                filter_exprs.push(expr);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(first) = filter_exprs.first() {
            let mut filter = first.clone();
            for expr in filter_exprs.iter().skip(1) {
                filter = filter & expr.clone();
            }
            Ok(df.filter(&filter)?)
        } else {
            Ok(df)
        }
    }

    /// Apply sorting to data frame
    fn apply_sort(
        &self,
        df: DataFrame,
        column: &str,
        direction: SortDirection,
    ) -> Result<DataFrame> {
        match direction {
            SortDirection::Ascending => df.sort([column], SortMultipleOptions::default()),
            SortDirection::Descending => {
                let df = df.sort([column], SortMultipleOptions::default());
                df.sort([column], SortMultipleOptions::default().with_reverse(true))
            }
        }
    }

    /// Navigate to previous row
    pub fn select_up(&self) {
        let mut selected_row = self.selected_row.write().unwrap();
        let scroll_offset = self.scroll_offset.read().unwrap();

        if *selected_row > 0 {
            *selected_row -= 1;
        }

        // Auto-scroll if selected row is above viewport
        if *selected_row < *scroll_offset {
            *scroll_offset = *selected_row;
        }
    }

    /// Navigate to next row
    pub fn select_down(&self) {
        let mut selected_row = self.selected_row.write().unwrap();
        let scroll_offset = self.scroll_offset.read().unwrap();
        let row_count = self.row_count.read().unwrap();

        if *selected_row < *row_count - 1 {
            *selected_row += 1;
        }

        // Auto-scroll if selected row is below viewport
        if *selected_row >= *scroll_offset + 20 {
            *scroll_offset = *selected_row.saturating_sub(19);
        }
    }

    /// Navigate to first row
    pub fn go_to_top(&self) {
        let mut selected_row = self.selected_row.write().unwrap();
        let mut scroll_offset = self.scroll_offset.write().unwrap();

        *selected_row = 0;
        *scroll_offset = 0;
    }

    /// Navigate to last row
    pub fn go_to_bottom(&self) {
        let mut selected_row = self.selected_row.write().unwrap();
        let mut scroll_offset = self.scroll_offset.write().unwrap();
        let row_count = self.row_count.read().unwrap();

        *selected_row = row_count.saturating_sub(1);
        *scroll_offset = row_count.saturating_sub(20);
    }

    /// Page up
    pub fn page_up(&self) {
        let mut scroll_offset = self.scroll_offset.write().unwrap();
        let row_count = self.row_count.read().unwrap();

        *scroll_offset = scroll_offset.saturating_sub(10);
    }

    /// Page down
    pub fn page_down(&self) {
        let mut scroll_offset = self.scroll_offset.write().unwrap();
        let row_count = self.row_count.read().unwrap();

        *scroll_offset = scroll_offset.saturating_add(10);
        if *scroll_offset >= row_count {
            *scroll_offset = row_count.saturating_sub(1);
        }
    }

    /// Set search query
    pub fn set_search_query(&self, query: Option<String>) {
        let mut search_query = self.search_query.write().unwrap();
        let query_clone = query.clone();
        *search_query = query;

        if query_clone.is_some() {
            self.add_status_message(
                StatusMessage {
                    message: format!("Searching: {}", query_clone.as_ref().unwrap()),
                    timestamp: SystemTime::now(),
                    level: StatusLevel::Info,
                },
                3,
            );
        }
    }

    /// Get search query
    pub fn get_search_query(&self) -> Result<Option<String>> {
        Ok(self.search_query.read().unwrap().clone())
    }

    /// Set filter expression
    pub fn set_filter_expr(&self, expr: Option<String>) {
        let mut filter_expr = self.filter_expr.write().unwrap();
        let expr_clone = expr.clone();
        *filter_expr = expr;

        if expr_clone.is_some() {
            self.add_status_message(
                StatusMessage {
                    message: format!("Filter: {}", expr_clone.as_ref().unwrap()),
                    timestamp: SystemTime::now(),
                    level: StatusLevel::Info,
                },
                3,
            );
        }
    }

    /// Get filter expression
    pub fn get_filter_expr(&self) -> Result<Option<String>> {
        Ok(self.filter_expr.read().unwrap().clone())
    }

    /// Set sort column and direction
    pub fn set_sort(&self, column: Option<String>, direction: SortDirection) {
            pub fn set_sort(&self, column: Option<String>, direction: SortDirection) {
                let mut sort_column = self.sort_column.write().unwrap();
                let mut sort_direction = self.sort_direction.write().unwrap();
                let column_clone = column.clone();
                let direction_clone = direction.clone();

                *sort_column = column;
                *sort_direction = direction;

                if column_clone.is_some() {
                    let dir_str = match direction_clone {
                        SortDirection::Ascending => "ascending",
                        SortDirection::Descending => "descending",
                    };
                    self.add_status_message(
                        StatusMessage {
                            message: format!("Sort: {} ({})", column_clone.as_ref().unwrap(), dir_str),
                            timestamp: SystemTime::now(),
                            level: StatusLevel::Info,
                        },
                        3,
                    );
               }

    /// Get sort column
    pub fn get_sort_column(&self) -> Result<Option<String>> {
        Ok(self.sort_column.read().unwrap().clone())
    }

    /// Get sort direction
    pub fn get_sort_direction(&self) -> Result<SortDirection> {
        Ok(self.sort_direction.read().unwrap().clone())
    }

    /// Quit application
    pub fn quit(&self) {
        self.add_status_message(
            StatusMessage {
                message: "Goodbye!".to_string(),
                timestamp: SystemTime::now(),
                level: StatusLevel::Info,
            },
            5,
        );
    }

    /// Add status message
    pub fn add_status_message(&self, message: StatusMessage, duration: u64) {
        let mut messages = self.status_messages.write().unwrap();

        messages.push(message);

        // Remove old messages
        messages.retain(|msg| {
            if msg.timestamp.elapsed().unwrap().as_secs() < duration {
                true
            } else {
                false
            }
        });
    }

    /// Get status messages
    pub fn get_status_messages(&self) -> Result<Vec<StatusMessage>> {
        Ok(self.status_messages.read().unwrap().clone())
    }

    /// Clear status messages
    pub fn clear_status_messages(&self) {
        self.status_messages.write().unwrap().clear();
    }

    /// Set view mode


    /// Get view mode
    pub fn get_view_mode(&self) -> Result<ViewMode> {
        Ok(self.view_mode.read().unwrap().clone())
    }

    /// Set file path
    pub fn set_file_path(&self, path: PathBuf) {
        self.file_path.write().unwrap().replace(path);
    }

    /// Get file path
    pub fn get_file_path(&self) -> Result<Option<PathBuf>> {
        Ok(self.file_path.read().unwrap().clone())
    }

    /// Set file modification time
    pub fn set_file_mod_time(&self, time: Option<SystemTime>) {
        self.file_mod_time.write().unwrap().replace(time);
    }

    /// Get file modification time
    pub fn get_file_mod_time(&self) -> Result<Option<SystemTime>> {
        Ok(self.file_mod_time.read().unwrap().clone())
    }

    /// Get data frame
    pub fn get_data_frame(&self) -> Result<DataFrame> {
        Ok(self.data_frame.read().unwrap().clone())
    }

    /// Get column names
    pub fn get_column_names(&self) -> Result<Vec<String>> {
        Ok(self.column_names.read().unwrap().clone())
    }

    /// Get row count
    pub fn get_row_count(&self) -> Result<usize> {
        Ok(self.row_count.read().unwrap().clone())
    }

    /// Get column count
    pub fn get_column_count(&self) -> Result<usize> {
        Ok(self.column_names.read().unwrap().len())
    }

    /// Get selected row
    pub fn get_selected_row(&self) -> Result<usize> {
        Ok(self.selected_row.read().unwrap().clone())
    }

    /// Get column names
    pub fn get_column_names(&self) -> Result<Vec<String>> {
        Ok(self.column_names.read().unwrap().clone())
    }

    /// Get row range for rendering
    pub fn get_row_range(&self) -> Result<(usize, usize)> {
        let scroll_offset = self.scroll_offset.read().unwrap();
        let row_count = self.row_count.read().unwrap();
        let selected_row = self.selected_row.read().unwrap();

        let start_row = scroll_offset;
        let end_row = (scroll_offset + 20).min(row_count);

        Ok((start_row, end_row))
    }

    /// Get width
    pub fn get_width(&self) -> Result<usize> {
        Ok(80) // Default width
    }

    /// Get height
    pub fn get_height(&self) -> Result<usize> {
        Ok(24) // Default height
    }
}

/// Serialize state to JSON
pub fn serialize_state(state: &SheetsState) -> Result<String> {
    serde_json::to_string_pretty(state)
        .map_err(|e| StateError::StateError(format!("Serialization error: {}", e)))
}

/// Deserialize state from JSON
pub fn deserialize_state(json: &str) -> Result<SheetsState> {
    serde_json::from_str(json)
        .map_err(|e| StateError::StateError(format!("Deserialization error: {}", e)))
}

/// Save state to file
pub fn save_state(state: &SheetsState, path: &PathBuf) -> Result<()> {
    let json = serialize_state(state)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load state from file
pub fn load_state(path: &PathBuf) -> Result<SheetsState> {
    let json = std::fs::read_to_string(path)?;
    deserialize_state(&json)
}
