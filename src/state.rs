// Zellij Sheets - State Management Module
// Handles application state for the spreadsheet viewer

use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use zellij_term::prelude::*;

/// Application state for the spreadsheet viewer
#[derive(Clone)]
pub struct SheetsState {
    /// Loaded data frame
    data: Option<DataFrame>,

    /// Column names (headers)
    headers: Option<Vec<String>>,

    /// Current scroll position (row)
    scroll_row: usize,

    /// Currently selected row
    selected_row: usize,

    /// Maximum scroll position
    max_scroll_row: usize,

    /// File name being viewed
    file_name: String,

    /// Window dimensions
    width: usize,
    height: usize,

    /// Configuration
    config: Arc<SheetsConfig>,
}

impl SheetsState {
    /// Create a new empty state
    pub fn new(config: Arc<SheetsConfig>) -> Self {
        Self {
            data: None,
            headers: None,
            scroll_row: 0,
            selected_row: 0,
            max_scroll_row: 0,
            file_name: String::new(),
            width: 0,
            height: 0,
            config,
        }
    }

    /// Initialize state with data
    pub fn init(&mut self, data: DataFrame, headers: Vec<String>) -> ZellijResult<()> {
        self.data = Some(data);
        self.headers = Some(headers);
        self.file_name = String::new(); // File name will be set by the caller

        // Calculate max scroll position
        self.max_scroll_row = self.row_count().saturating_sub(self.visible_rows());

        Ok(())
    }

    /// Resize window
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;

        // Recalculate max scroll position
        self.max_scroll_row = self.row_count().saturating_sub(self.visible_rows());

        // Clamp scroll position
        self.scroll_row = self.scroll_row.min(self.max_scroll_row);

        // Clamp selected row
        if self.selected_row >= self.row_count() {
            self.selected_row = self.row_count().saturating_sub(1);
        }
    }

    /// Scroll up one row
    pub fn scroll_up(&mut self) {
        if self.scroll_row > 0 {
            self.scroll_row = self.scroll_row.saturating_sub(1);
        }
    }

    /// Scroll down one row
    pub fn scroll_down(&mut self) {
        if self.scroll_row < self.max_scroll_row {
            self.scroll_row = self.scroll_row.min(self.max_scroll_row);
            if self.scroll_row + self.visible_rows() < self.row_count() {
                self.scroll_row += 1;
            }
        }
    }

    /// Scroll left one column
    pub fn scroll_left(&mut self) {
        // Implement horizontal scrolling if needed
    }

    /// Scroll right one column
    pub fn scroll_right(&mut self) {
        // Implement horizontal scrolling if needed
    }

    /// Page up
    pub fn page_up(&mut self) {
        let page_size = self.config.behavior.page_size;
        self.scroll_row = self.scroll_row.saturating_sub(page_size.saturating_sub(1));
    }

    /// Page down
    pub fn page_down(&mut self) {
        self.scroll_row = self.scroll_row.min(self.max_scroll_row);
        if self.scroll_row + self.visible_rows() < self.row_count() {
            self.scroll_row += self.config.behavior.page_size.saturating_sub(1);
        }
    }

    /// Go to top
    pub fn go_to_top(&mut self) {
        self.scroll_row = 0;
        self.selected_row = 0;
    }

    /// Go to bottom
    pub fn go_to_bottom(&mut self) {
        self.scroll_row = self.max_scroll_row;
        self.selected_row = self.row_count().saturating_sub(1);
    }

    /// Select row up
    pub fn select_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row = self.selected_row.saturating_sub(1);

            // Keep selection visible
            if self.selected_row < self.scroll_row {
                self.scroll_row = self.selected_row;
            }
        }
    }

    /// Select row down
    pub fn select_down(&mut self) {
        if self.selected_row < self.row_count().saturating_sub(1) {
            self.selected_row += 1;

            // Keep selection visible
            if self.selected_row >= self.scroll_row + self.visible_rows() {
                self.scroll_row = self
                    .selected_row
                    .saturating_sub(self.visible_rows())
                    .saturating_sub(1);
            }
        }
    }

    /// Select first column
    pub fn select_first_column(&mut self) {
        // Implement column selection if needed
    }

    /// Select last column
    pub fn select_last_column(&mut self) {
        // Implement column selection if needed
    }

    /// Quit application
    pub fn quit(&mut self) {
        // Clean up resources if needed
    }

    /// Get current scroll row
    pub fn scroll_row(&self) -> usize {
        self.scroll_row
    }

    /// Get selected row
    pub fn selected_row(&self) -> usize {
        self.selected_row
    }

    /// Get maximum scroll row
    pub fn max_scroll_row(&self) -> usize {
        self.max_scroll_row
    }

    /// Get row count
    pub fn row_count(&self) -> usize {
        self.data.as_ref().map_or(0, |d| d.height() as usize)
    }

    /// Get column count
    pub fn col_count(&self) -> usize {
        self.data.as_ref().map_or(0, |d| d.width() as usize)
    }

    /// Get headers
    pub fn headers(&self) -> Option<&Vec<String>> {
        self.headers.as_ref()
    }

    /// Get file name
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Get visible rows
    pub fn visible_rows(&self) -> usize {
        (self.height as usize).saturating_sub(4) // Subtract header and footer
    }

    /// Get cell at position
    pub fn get_cell(&self, row: usize, col: usize) -> Option<String> {
        self.data.as_ref().and_then(|d| {
            if col >= self.col_count() {
                return None;
            }

            if let Ok(series) = d.column(self.headers.as_ref()?.get(col)?.clone()) {
                series
                    .iter()
                    .nth(row as usize)
                    .map(|v| v.to_string())
                    .into()
            } else {
                None
            }
        })
    }

    /// Get data type at position
    pub fn get_data_type(&self, col: usize) -> Option<DataType> {
        self.data.as_ref().and_then(|d| {
            if col >= self.col_count() {
                return None;
            }

            d.dtypes().get(col).copied().map(|dt| dt.into())
        })
    }

    /// Check if scroll is at top
    pub fn at_top(&self) -> bool {
        self.scroll_row == 0
    }

    /// Check if scroll is at bottom
    pub fn at_bottom(&self) -> bool {
        self.scroll_row >= self.max_scroll_row
    }

    /// Get row range for display
    pub fn row_range(&self) -> (usize, usize) {
        let start = self.scroll_row;
        let end = (start + self.visible_rows()).min(self.row_count());
        (start, end)
    }
}

/// Data type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Int64,
    UInt64,
    Float32,
    Float64,
    String,
    Boolean,
    Date,
    Time,
    Datetime,
    Null,
    Unknown,
}

/// Convert Polars DataType to our DataType enum
impl From<polars::prelude::DataType> for DataType {
    fn from(dtype: polars::prelude::DataType) -> Self {
        match dtype {
            polars::prelude::DataType::Int64 => DataType::Int64,
            polars::prelude::DataType::UInt64 => DataType::UInt64,
            polars::prelude::DataType::Float32 => DataType::Float32,
            polars::prelude::DataType::Float64 => DataType::Float64,
            polars::prelude::DataType::String => DataType::String,
            polars::prelude::DataType::Boolean => DataType::Boolean,
            polars::prelude::DataType::Date => DataType::Date,
            polars::prelude::DataType::Time => DataType::Time,
            polars::prelude::DataType::Datetime(_, _) => DataType::Datetime,
            polars::prelude::DataType::Null => DataType::Null,
            _ => DataType::Unknown,
        }
    }
}

/// Handle Zellij plugin events
impl ZellijPlugin for SheetsState {
    fn handle_event(&mut self, event: ZellijEvent) -> ZellijResult<Option<ZellijEvent>> {
        match event {
            ZellijEvent::Init => {
                self.init(ZellijEvent::Draw);
                Ok(Some(ZellijEvent::Draw))
            }
            ZellijEvent::Resize { width, height } => {
                self.resize(width, height);
                Ok(Some(ZellijEvent::Draw))
            }
            ZellijEvent::Input { key, mod_mask } => {
                handle_input(key, mod_mask, self);
                Ok(Some(ZellijEvent::Draw))
            }
            ZellijEvent::Draw => {
                render_ui(self.clone());
                Ok(None)
            }
            ZellijEvent::Quit => {
                self.quit();
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

/// Handle keyboard input
fn handle_input(key: Key, mod_mask: Modifiers, state: &mut SheetsState) {
    match (mod_mask, key) {
        (Modifiers::NONE, Key::ArrowUp) => state.select_up(),
        (Modifiers::NONE, Key::ArrowDown) => state.select_down(),
        (Modifiers::NONE, Key::Home) => state.go_to_top(),
        (Modifiers::NONE, Key::End) => state.go_to_bottom(),
        (Modifiers::NONE, Key::PageUp) => state.page_up(),
        (Modifiers::NONE, Key::PageDown) => state.page_down(),
        (Modifiers::CONTROL, Key::C) => state.quit(),
        _ => {}
    }
}

/// Render UI
fn render_ui(state: SheetsState) {
    let mut screen = Screen::default();

    // Draw header
    draw_header(&mut screen, &state);

    // Draw grid
    draw_grid(&mut screen, &state);

    // Draw footer
    draw_footer(&mut screen, &state);
}

/// Draw header
fn draw_header(screen: &mut Screen, state: &SheetsState) {
    let header_style = Style::default()
        .fg(Color::Rgb(0, 120, 215))
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    let title = format!("📊 Zellij Sheets - {}", state.file_name());
    screen.draw_string(0, 0, &title, header_style);

    let info = format!("{} rows × {} columns", state.row_count(), state.col_count());
    screen.draw_string(0, 1, &info, Style::default().fg(Color::White));

    // Draw column headers
    if let Some(headers) = state.headers() {
        let header_x = 0;
        let header_y = 2;

        for (col, header) in headers.iter().enumerate() {
            let header_style = Style::default()
                .fg(Color::Cyan)
                .bg(Color::Rgb(0, 0, 0))
                .add_modifier(Modifier::BOLD);

            screen.draw_string(header_x, header_y, header, header_style);
            header_x += header.len() + 1;
        }
    }
}

/// Draw grid
fn draw_grid(screen: &mut Screen, state: &SheetsState) {
    let grid_style = Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0));
    let selected_style = Style::default()
        .fg(Color::White)
        .bg(Color::Rgb(0, 100, 200));

    let (start_row, end_row) = state.row_range();

    for row in start_row..end_row {
        let y = 3 + (row - start_row) as usize;

        if let Some(row_data) = state.get_row(row) {
            for (col, cell) in row_data.iter().enumerate() {
                let x = col as usize;
                let style = if row == state.selected_row() {
                    selected_style
                } else {
                    grid_style
                };

                screen.draw_string(x, y, cell, style);
            }
        }
    }
}

/// Draw footer
fn draw_footer(screen: &mut Screen, state: &SheetsState) {
    let footer_style = Style::default()
        .fg(Color::Rgb(0, 120, 215))
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    let footer = format!(
        "Scroll: ↑↓←→ | Page: PGUP/PGDN | Home: HOME | End: END | Quit: CTRL+C | Row: {}",
        state.selected_row() + 1
    );

    screen.draw_string(0, state.height() - 1, &footer, footer_style);
}

/// Handle Zellij plugin events
impl ZellijPlugin for SheetsState {
    fn handle_event(&mut self, event: ZellijEvent) -> ZellijResult<Option<ZellijEvent>> {
        match event {
            ZellijEvent::Init => {
                self.init(ZellijEvent::Draw);
                Ok(Some(ZellijEvent::Draw))
            }
            ZellijEvent::Resize { width, height } => {
                self.resize(width, height);
                Ok(Some(ZellijEvent::Draw))
            }
            ZellijEvent::Input { key, mod_mask } => {
                handle_input(key, mod_mask, self);
                Ok(Some(ZellijEvent::Draw))
            }
            ZellijEvent::Draw => {
                render_ui(self.clone());
                Ok(None)
            }
            ZellijEvent::Quit => {
                self.quit();
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

/// Handle keyboard input
fn handle_input(key: Key, mod_mask: Modifiers, state: &mut SheetsState) {
    match (mod_mask, key) {
        (Modifiers::NONE, Key::ArrowUp) => state.select_up(),
        (Modifiers::NONE, Key::ArrowDown) => state.select_down(),
        (Modifiers::NONE, Key::Home) => state.go_to_top(),
        (Modifiers::NONE, Key::End) => state.go_to_bottom(),
        (Modifiers::NONE, Key::PageUp) => state.page_up(),
        (Modifiers::NONE, Key::PageDown) => state.page_down(),
        (Modifiers::CONTROL, Key::C) => state.quit(),
        _ => {}
    }
}

/// Render UI
fn render_ui(state: SheetsState) {
    let mut screen = Screen::default();

    // Draw header
    draw_header(&mut screen, &state);

    // Draw grid
    draw_grid(&mut screen, &state);

    // Draw footer
    draw_footer(&mut screen, &state);
}

/// Draw header
fn draw_header(screen: &mut Screen, state: &SheetsState) {
    let header_style = Style::default()
        .fg(Color::Rgb(0, 120, 215))
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    let title = format!("📊 Zellij Sheets - {}", state.file_name());
    screen.draw_string(0, 0, &title, header_style);

    let info = format!("{} rows × {} columns", state.row_count(), state.col_count());
    screen.draw_string(0, 1, &info, Style::default().fg(Color::White));

    // Draw column headers
    if let Some(headers) = state.headers() {
        let header_x = 0;
        let header_y = 2;

        for (col, header) in headers.iter().enumerate() {
            let header_style = Style::default()
                .fg(Color::Cyan)
                .bg(Color::Rgb(0, 0, 0))
                .add_modifier(Modifier::BOLD);

            screen.draw_string(header_x, header_y, header, header_style);
            header_x += header.len() + 1;
        }
    }
}

/// Draw grid
fn draw_grid(screen: &mut Screen, state: &SheetsState) {
    let grid_style = Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0));
    let selected_style = Style::default()
        .fg(Color::White)
        .bg(Color::Rgb(0, 100, 200));

    let (start_row, end_row) = state.row_range();

    for row in start_row..end_row {
        let y = 3 + (row - start_row) as usize;

        if let Some(row_data) = state.get_row(row) {
            for (col, cell) in row_data.iter().enumerate() {
                let x = col as usize;
                let style = if row == state.selected_row() {
                    selected_style
                } else {
                    grid_style
                };

                screen.draw_string(x, y, cell, style);
            }
        }
    }
}

/// Draw footer
fn draw_footer(screen: &mut Screen, state: &SheetsState) {
    let footer_style = Style::default()
        .fg(Color::Rgb(0, 120, 215))
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    let footer = format!(
        "Scroll: ↑↓←→ | Page: PGUP/PGDN | Home: HOME | End: END | Quit: CTRL+C | Row: {}",
        state.selected_row() + 1
    );

    screen.draw_string(0, state.height() - 1, &footer, footer_style);
}
