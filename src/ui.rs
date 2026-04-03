// Zellij Sheets - UI Rendering Module

use crate::layout::{fit_cell, ColumnLayout, LayoutEngine};
use crate::state::{DataType, SheetsState, StatusLevel};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Default)]
pub struct Colors {
    pub foreground: Option<String>,
    pub background: Option<String>,
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
        "\x1b[0m".to_string()
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

pub struct UiRenderer {
    use_colors: bool,
    theme: Option<ThemeConfig>,
    layout_engine: LayoutEngine,
}

impl Default for UiRenderer {
    fn default() -> Self {
        Self::new()
    }
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
            layout_engine: LayoutEngine::new(),
        }
    }

    pub fn with_theme(theme: ThemeConfig) -> Self {
        Self {
            use_colors: true,
            theme: Some(theme),
            layout_engine: LayoutEngine::new(),
        }
    }

    pub fn set_use_colors(&mut self, use_colors: bool) {
        self.use_colors = use_colors;
    }

    pub fn set_theme(&mut self, theme: ThemeConfig) {
        self.theme = Some(theme);
    }

    pub fn draw_ui(&self, state: &SheetsState) -> Result<String> {
        let mut lines = Vec::new();
        lines.push(self.draw_header(state)?);
        lines.push(self.draw_separator(state)?);
        self.draw_data_rows(&mut lines, state)?;
        lines.push(self.draw_footer(state)?);
        Ok(lines.join("\n"))
    }

    fn draw_header(&self, state: &SheetsState) -> Result<String> {
        let theme = self.get_theme();
        let header_style = format!(
            "\x1b[38;5;{}m\x1b[48;5;{}m",
            theme.header_fg, theme.header_bg
        );
        let sep_style = format!("\x1b[38;5;{}m", theme.separator_fg);
        let reset = "\x1b[0m";
        let mode_label = state
            .get_view_mode()
            .map(|m| format!("{:?}", m))
            .unwrap_or_default();

        let header = format!(
            "{}Zellij Sheets{} | {} | {} rows | {}{}",
            header_style,
            reset,
            state.file_name(),
            state.row_count(),
            mode_label,
            sep_style,
        );

        Ok(format!("{}{}", header, reset))
    }

    fn draw_separator(&self, state: &SheetsState) -> Result<String> {
        let width = state.get_width().unwrap_or(80);
        Ok(format!("\x1b[38;5;240m{}\x1b[0m", "─".repeat(width)))
    }

    fn draw_data_rows(&self, lines: &mut Vec<String>, state: &SheetsState) -> Result<()> {
        let width = state.get_width().unwrap_or(80);
        let layouts = self.layout_engine.resolve(&state.layout_cache, width);
        let (start, end) = state.row_range();

        if let Some(headers) = state.headers() {
            lines.push(self.render_row(headers, state, &layouts, true)?);
        }

        for row in start..end {
            if let Some(values) = state.get_row(row) {
                let is_selected = row == state.selected_row();
                let prefix = if is_selected { ">" } else { " " };
                let row_line = self.render_row(&values, state, &layouts, false)?;
                lines.push(format!("{}{}", prefix, row_line));
            }
        }

        Ok(())
    }

    fn draw_footer(&self, state: &SheetsState) -> Result<String> {
        let theme = self.get_theme();
        let sep_style = format!("\x1b[38;5;{}m", theme.separator_fg);
        let reset = "\x1b[0m";
        let mut footer = format!(
            "{}Keys: Up/Down, PgUp/PgDn, Home/End, q/Ctrl-C{}",
            sep_style, reset
        );

        if let Ok(messages) = state.get_status_messages() {
            if let Some(msg) = messages.iter().next_back() {
                let level_color = match msg.level {
                    StatusLevel::Info => "34",
                    StatusLevel::Success => "32",
                    StatusLevel::Warning => "33",
                    StatusLevel::Error => "31",
                };
                footer.push_str(&format!(" | \x1b[{}m{}\x1b[0m", level_color, msg.message));
            }
        }

        Ok(footer)
    }

    fn render_row(
        &self,
        values: &[String],
        state: &SheetsState,
        layouts: &[ColumnLayout],
        is_header: bool,
    ) -> Result<String> {
        let mut cells = Vec::new();
        let theme = self.get_theme();

        for (col, value) in values.iter().enumerate() {
            let width = layouts.get(col).map(|l| l.resolved_width).unwrap_or(8);
            let fitted = fit_cell(value, width);

            let cell_value = if is_header {
                format!("\x1b[1;38;5;{}m{}\x1b[0m", theme.header_fg, fitted)
            } else {
                let color_code = match state.get_data_type(col).unwrap_or(DataType::String) {
                    DataType::Number => theme.data_type_number.as_str(),
                    DataType::Boolean => theme.data_type_boolean.as_str(),
                    DataType::Empty => theme.data_type_empty.as_str(),
                    DataType::String => theme.data_type_string.as_str(),
                };
                format!("\x1b[{}m{}\x1b[0m", color_code, fitted)
            };

            cells.push(cell_value);
        }

        Ok(cells.join(" | "))
    }

    fn get_theme(&self) -> ThemeConfig {
        self.theme.clone().unwrap_or_default()
    }

    pub fn get_color(&self, color_code: &str) -> String {
        let color = if color_code.is_empty() {
            "0"
        } else {
            color_code
        };
        if self.use_colors {
            format!("\x1b[{}m", color)
        } else {
            String::new()
        }
    }
}

pub fn draw_error(message: &str) -> String {
    format!("\x1b[31mError: {}\x1b[0m", message)
}

pub fn draw_warning(message: &str) -> String {
    format!("\x1b[33mWarning: {}\x1b[0m", message)
}

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
