// Zellij Sheets - UI Rendering Module
// Enhanced with better error handling, color support, and improved rendering

use crate::state::{DataType, SheetsState, StatusLevel, StatusMessage, ViewMode};
use std::fmt;

/// Color codes for terminal output
pub struct Colors {
    pub foreground: Option<String>,
    pub background: Option<String>,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            foreground: None,
            background: None,
        }
    }
}

impl Colors {
    pub fn with_fg(fg: impl Into<String>) -> Self {
        Self {
            foreground: Some(fg.into()),
            ..Default::default()
        }
    }

    pub fn with_bg(bg: impl Into<String>) -> Self {
        Self {
            background: Some(bg.into()),
            ..Default::default()
        }
    }

    pub fn reset(&self) -> String {
        format!("\x1b[0m")
    }

    pub fn apply(&self) -> String {
        let mut result = String::new();
        if let Some(fg) = &self.foreground {
            result.push_str(&format!("\x1b[{}m", fg));
        }
        if let Some(bg) = &self.background {
            result.push_str(&format!("\x1b[{}m", bg));
        }
        result
    }
}

/// Error types for UI operations
#[derive(Debug, Error)]
pub enum UiError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Rendering error: {0}")]
    RenderError(String),

    #[error("Color formatting error: {0}")]
    ColorError(String),
}

pub type Result<T> = std::result::Result<T, UiError>;

/// Main UI renderer for the spreadsheet viewer
pub struct UiRenderer {
    use_colors: bool,
    theme: Option<ThemeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub header_fg: String,
    pub header_bg: String,
    pub selected_fg: String,
    pub selected_bg: String,
    pub current_row_fg: String,
    pub current_row_bg: String,
    pub separator_fg: String,
    pub data_type_number: String,
    pub data_type_string: String,
    pub data_type_boolean: String,
    pub data_type_empty: String,
    pub data_type_date: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            header_fg: "255".to_string(),
            header_bg: "33".to_string(),
            selected_fg: "0".to_string(),
            selected_bg: "39".to_string(),
            current_row_fg: "255".to_string(),
            current_row_bg: "33".to_string(),
            separator_fg: "240".to_string(),
            data_type_number: "32".to_string(),
            data_type_string: "33".to_string(),
            data_type_boolean: "35".to_string(),
            data_type_empty: "90".to_string(),
            data_type_date: "33".to_string(),
        }
    }
}

impl UiRenderer {
    pub fn new() -> Self {
        Self {
            use_colors: true,
            theme: None,
        }
    }

    pub fn with_theme(theme: ThemeConfig) -> Self {
        Self {
            use_colors: true,
            theme: Some(theme),
        }
    }

    pub fn set_use_colors(&mut self, use_colors: bool) {
        self.use_colors = use_colors;
    }

    pub fn set_theme(&mut self, theme: ThemeConfig) {
        self.theme = Some(theme);
    }

    /// Main draw function that renders the entire UI
    pub fn draw_ui(&self, state: &SheetsState) -> Result<String> {
        let mut lines = Vec::new();

        // Draw header
        lines.push(self.draw_header(state)?);

        // Draw separator
        lines.push(self.draw_separator(state)?);

        // Draw data rows
        self.draw_data_rows(&mut lines, state)?;

        // Draw footer
        lines.push(self.draw_footer(state)?);

        Ok(lines.join("\n"))
    }

    /// Draw the header section
    fn draw_header(&self, state: &SheetsState) -> Result<String> {
        let theme = self.get_theme();
        let header_fg = self.get_color(theme.header_fg);
        let header_bg = self.get_color(theme.header_bg);
        let separator_fg = self.get_color(theme.separator_fg);

        let mut header = format!(
            "{}{} {} {}{}{}",
            header_bg,
            header_fg,
            "Zellij Sheets",
            separator_fg,
            state.file_name()
        );

        // Add view mode indicator
        if let Ok(mode) = state.get_view_mode() {
            header.push_str(&format!(" | {} {}", separator_fg, mode));
        }

        // Add row count
        header.push_str(&format!(" | {} {}", separator_fg, state.row_count()));

        Ok(header)
    }

    /// Draw the separator line
    fn draw_separator(&self, state: &SheetsState) -> Result<String> {
        let theme = self.get_theme();
        let separator_fg = self.get_color(theme.separator_fg);
        let width = state.width().unwrap_or(80);
        Ok(separator_fg.repeat(width))
    }

    /// Draw the data rows
    fn draw_data_rows(&self, lines: &mut Vec<String>, state: &SheetsState) -> Result<()> {
        let (start, end) = state.row_range()?;
        let theme = self.get_theme();

        // Draw headers
        if let Some(headers) = state.headers() {
            let header_line = self.render_row(&headers, state, true)?;
            lines.push(header_line);
        }

        // Draw data rows
        for row in start..end {
            if let Some(values) = state.get_row(row) {
                let is_selected = row == state.selected_row();
                let prefix = if is_selected { ">" } else { " " };
                let row_line = self.render_row(&values, state, false)?;
                lines.push(format!("{}{}", prefix, row_line));
            }
        }

        Ok(())
    }

    /// Draw the footer section
    fn draw_footer(&self, state: &SheetsState) -> Result<String> {
        let theme = self.get_theme();
        let separator_fg = self.get_color(theme.separator_fg);

        let mut footer = format!(
            "{} Keys: Up/Down, PgUp/PgDn, Home/End, q/Ctrl-C",
            separator_fg
        );

        // Add status messages if any
        if let Ok(messages) = state.get_status_messages() {
            if !messages.is_empty() {
                footer.push_str(&format!(" | {}", separator_fg));
                for msg in messages.iter().rev().take(1) {
                    let level_color = match msg.level {
                        StatusLevel::Info => "34".to_string(),    // Blue
                        StatusLevel::Success => "32".to_string(), // Green
                        StatusLevel::Warning => "33".to_string(), // Yellow
                        StatusLevel::Error => "31".to_string(),   // Red
                    };
                    footer.push_str(&format!("{}{}", self.get_color(level_color), msg.message));
                }
            }
        }

        Ok(footer)
    }

    /// Render a single row with proper formatting
    fn render_row(
        &self,
        values: &[String],
        state: &SheetsState,
        is_header: bool,
    ) -> Result<String> {
        let theme = self.get_theme();
        let max_width = state
            .get_config()
            .map(|config| config.display.max_cell_length)
            .unwrap_or(24);

        let mut rendered = Vec::new();

        for (i, value) in values.iter().enumerate() {
            let cell_value = self.format_cell(value, state, i, is_header)?;
            rendered.push(cell_value);
        }

        Ok(rendered.join(" | "))
    }

    /// Format a single cell value
    fn format_cell(
        &self,
        value: &str,
        state: &SheetsState,
        col: usize,
        is_header: bool,
    ) -> Result<String> {
        let theme = self.get_theme();
        let max_width = state
            .get_config()
            .map(|config| config.display.max_cell_length)
            .unwrap_or(24);

        // Truncate long values
        let mut formatted = if value.chars().count() > max_width {
            let truncated = value
                .chars()
                .take(max_width.saturating_sub(1))
                .collect::<String>();
            format!("{}~", truncated)
        } else {
            value.to_string()
        };

        // Add data type color if enabled
        if state.get_show_data_types().unwrap_or(false) {
            if let Some(data_type) = state.get_data_type(col) {
                let color = match data_type {
                    DataType::Number => self.get_color(theme.data_type_number),
                    DataType::String => self.get_color(theme.data_type_string),
                    DataType::Boolean => self.get_color(theme.data_type_boolean),
                    DataType::Empty => self.get_color(theme.data_type_empty),
                };
                formatted = format!("{}{}", color, formatted);
            }
        }

        Ok(formatted)
    }

    /// Get theme configuration
    fn get_theme(&self) -> ThemeConfig {
        self.theme.clone().unwrap_or_default()
    }

    /// Get color code for terminal
    fn get_color(&self, color_code: String) -> String {
        if self.use_colors {
            format!("\x1b[{}m", color_code)
        } else {
            String::new()
        }
    }
}

/// Draw a simple error message
pub fn draw_error(message: &str) -> String {
    format!("\x1b[31mError: {}\x1b[0m", message)
}

/// Draw a warning message
pub fn draw_warning(message: &str) -> String {
    format!("\x1b[33mWarning: {}\x1b[0m", message)
}

/// Draw an informational message
pub fn draw_info(message: &str) -> String {
    format!("\x1b[34mInfo: {}\x1b[0m", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SheetsConfig;
    use std::sync::Arc;

    #[test]
    fn test_ui_renderer_creation() {
        let renderer = UiRenderer::new();
        assert!(renderer
            .draw_ui(&SheetsState::new(Arc::new(SheetsConfig::default())))
            .is_ok());
    }

    #[test]
    fn test_theme_config_default() {
        let theme = ThemeConfig::default();
        assert_eq!(theme.header_fg, "255");
        assert_eq!(theme.header_bg, "33");
    }

    #[test]
    fn test_colors_default() {
        let colors = Colors::default();
        assert!(colors.foreground.is_none());
        assert!(colors.background.is_none());
    }
}
