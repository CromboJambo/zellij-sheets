// Zellij Sheets - UI Rendering Module

use crate::config::ThemeConfig;
use crate::layout::{fit_cell, ColumnLayout, LayoutEngine};
use crate::state::{DataType, SheetsState, StatusLevel};
use thiserror::Error;

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
            "\x1b[48;5;{}m\x1b[38;5;{}m",
            theme.header_background, theme.header_text
        );
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
            reset,
        );

        Ok(header)
    }

    fn draw_separator(&self, state: &SheetsState) -> Result<String> {
        let theme = self.get_theme();
        let width = state.get_width().unwrap_or(80);
        Ok(format!(
            "\x1b[48;5;{}m\x1b[38;5;{}m{}\x1b[0m",
            theme.header_background,
            theme.header_text,
            "─".repeat(width)
        ))
    }

    fn draw_data_rows(&self, lines: &mut Vec<String>, state: &SheetsState) -> Result<()> {
        let width = state.get_width().unwrap_or(80);
        let layouts = self.layout_engine.resolve(&state.layout_cache, width);
        let (start, end) = state.row_range();
        let visible_cols = state.visible_cols();

        if let Some(headers) = state.headers() {
            lines.push(self.render_row(headers, state, &layouts, true, None, visible_cols)?);
        }

        for row in start..end {
            if let Some(values) = state.get_row(row) {
                let is_selected = row == state.selected_row();
                let prefix = if is_selected { ">" } else { " " };
                let row_line =
                    self.render_row(&values, state, &layouts, false, Some(row), visible_cols)?;
                lines.push(format!("{}{}", prefix, row_line));
            }
        }

        Ok(())
    }

    fn draw_footer(&self, state: &SheetsState) -> Result<String> {
        let theme = self.get_theme();
        let sep_style = format!(
            "\x1b[48;5;{}m\x1b[38;5;{}m",
            theme.header_background, theme.header_text
        );
        let reset = "\x1b[0m";
        let mut footer = format!(
            "{}Keys: Arrows, h/j/k/l, PgUp/PgDn, Home/End, q/Ctrl-C{}",
            sep_style, reset
        );
        footer.push_str(&format!(
            " | row {} col {}",
            state.selected_row() + 1,
            state.selected_col() + 1
        ));

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
        row_index: Option<usize>,
        visible_cols: usize,
    ) -> Result<String> {
        let mut cells = Vec::new();
        let theme = self.get_theme();
        let col_offset = state.col_offset();

        for (col, value) in values
            .iter()
            .enumerate()
            .skip(col_offset)
            .take(visible_cols)
        {
            let width = layouts.get(col).map(|l| l.resolved_width).unwrap_or(8);
            let fitted = fit_cell(value, width);
            let is_selected_col = col == state.selected_col();
            let is_selected_cell = row_index == Some(state.selected_row()) && is_selected_col;

            let cell_value = if is_header {
                if is_selected_col {
                    format!(
                        "\x1b[1;48;5;{}m\x1b[38;5;{}m{}\x1b[0m",
                        theme.selected_background, theme.selected_text, fitted
                    )
                } else {
                    format!(
                        "\x1b[1;48;5;{}m\x1b[38;5;{}m{}\x1b[0m",
                        theme.header_background, theme.header_text, fitted
                    )
                }
            } else if is_selected_cell {
                format!(
                    "\x1b[48;5;{}m\x1b[38;5;{}m{}\x1b[0m",
                    theme.selected_background, theme.selected_text, fitted
                )
            } else {
                let color_code = match state.get_data_type(col).unwrap_or(DataType::String) {
                    DataType::Number => theme.accent_colors.number.as_str(),
                    DataType::Boolean => theme.accent_colors.boolean.as_str(),
                    DataType::Empty => theme.accent_colors.date.as_str(),
                    DataType::String => theme.accent_colors.string.as_str(),
                };
                format!(
                    "\x1b[48;5;{}m\x1b[38;5;{}m{}\x1b[0m",
                    theme.background, color_code, fitted
                )
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
            format!("\x1b[38;5;{}m", color)
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
        assert_eq!(theme.header_background, "#0055AA");
        assert_eq!(theme.header_text, "#FFFFFF");
    }

    #[test]
    fn test_colors_default() {
        let colors = Colors::default();
        assert!(colors.foreground.is_none());
        assert!(colors.background.is_none());
    }
}
