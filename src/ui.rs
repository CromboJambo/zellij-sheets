//! UI rendering helpers for zellij-sheets.
//!
//! Owns the terminal rendering logic and color scheme. The `UiRenderer` is a
//! thin wrapper around the layout engine and drawing functions.
//!
//! This module is shared between the plugin and native CLI.

use crate::{
    layout::ColumnLayout,
    state::{SearchDirection, SheetsState, ViewMode},
};
use thiserror::Error;

/// Error type for UI rendering operations.
#[derive(Debug, Error)]
pub enum UiError {
    #[error("Render error: {0}")]
    RenderError(String),

    #[error("Format error: {0}")]
    FmtError(#[from] std::fmt::Error),
}

/// Color scheme used by the renderer.
///
/// Each field must be a named ANSI terminal color recognized by
/// [`UiRenderer::get_color`]: `"black"`, `"red"`, `"green"`, `"yellow"`,
/// `"blue"`, `"magenta"`, `"cyan"`, `"white"`, or `"none"` / `""` to
/// suppress coloring entirely.  Hex strings are **not** supported — the
/// renderer emits SGR escape sequences directly and does not do palette
/// look-ups.
#[derive(Debug, Clone)]
pub struct Colors {
    pub header_background: String,
    pub header_text: String,
    pub selected_background: String,
    pub selected_text: String,
    pub separator: String,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            header_background: "blue".to_string(),
            header_text: "white".to_string(),
            selected_background: "cyan".to_string(),
            selected_text: "black".to_string(),
            separator: "none".to_string(),
        }
    }
}

/// Terminal UI renderer for zellij-sheets.
///
/// The `UiRenderer` is a thin wrapper around the layout engine and drawing
/// functions. It owns the color scheme and terminal escape sequences.
pub struct UiRenderer {
    theme: Colors,
}

impl Default for UiRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl UiRenderer {
    /// Create a new UI renderer with the default color scheme.
    pub fn new() -> Self {
        Self {
            theme: Colors::default(),
        }
    }

    /// Get the current color scheme.
    pub fn get_theme(&self) -> &Colors {
        &self.theme
    }

    /// Set the color scheme.
    pub fn set_theme(&mut self, theme: Colors) {
        self.theme = theme;
    }

    /// Draw the entire UI for a given state.
    ///
    /// This is the main entry point for rendering the spreadsheet grid.
    pub fn draw_ui(&self, state: &SheetsState) -> Result<String, UiError> {
        let mut lines = Vec::new();
        lines.push(self.draw_header(state)?);
        self.draw_data_rows(&mut lines, state)?;
        lines.push(self.draw_footer(state)?);
        Ok(lines.join("\n"))
    }

    fn draw_header(&self, state: &SheetsState) -> Result<String, UiError> {
        let theme = self.get_theme();
        let header_style = self.get_color(&theme.header_background);
        let text_style = self.get_color(&theme.header_text);
        let reset = "\x1b[0m";
        let mode_label = match state.get_view_mode() {
            ViewMode::Grid => "grid",
            ViewMode::List => "list",
            ViewMode::Compact => "compact",
            ViewMode::Raw => "raw",
        };

        let header = format!(
            "{}{}Zellij Sheets{} | {} | {} rows | {}{}",
            header_style,
            text_style,
            reset,
            state.file_name(),
            mode_label,
            state.row_count(),
            reset
        );
        Ok(header)
    }

    fn draw_data_rows(&self, lines: &mut Vec<String>, state: &SheetsState) -> Result<(), UiError> {
        let col_offset = state.col_offset();
        let visible_cols = state.visible_cols_from_offset(col_offset);

        let layouts = crate::layout::LayoutEngine::new()
            .resolve(&state.layout_cache, state.width());

        for row_index in state.scroll_row()..state.scroll_row().saturating_add(state.visible_rows())
        {
            let Some(row) = state.get_row(row_index) else {
                break;
            };
            lines.push(self.build_row(
                &row,
                state,
                &layouts,
                false,
                Some(row_index),
                visible_cols,
            ));
        }

        Ok(())
    }

    fn draw_footer(&self, state: &SheetsState) -> Result<String, UiError> {
        let theme = self.get_theme();
        let sep_style = self.get_color(&theme.separator);
        let reset = "\x1b[0m";
        let mut footer = format!(
            "{}Keys: Arrows, h/j/k/l, / ? n N, PgUp/PgDn, Home/End, q/Ctrl-C{}",
            sep_style, reset
        );
        footer.push_str(&format!(
            " | row {} col {}",
            state.selected_row() + 1,
            state.selected_col() + 1
        ));
        if let Some(query) = state.get_search_query() {
            let prefix = match state.search_direction() {
                SearchDirection::Forward => '/',
                SearchDirection::Backward => '?',
            };
            if state.is_search_active() || !query.is_empty() {
                footer.push_str(&format!(" | {prefix}{query}"));
            }
        }

        Ok(footer)
    }

    /// Build a single display row.
    ///
    /// - `is_header`: plain text
    /// - `is_selected`: prefixed with `>`
    /// - plain data rows: prefixed with a space
    fn build_row(
        &self,
        values: &[String],
        state: &SheetsState,
        layouts: &[ColumnLayout],
        is_header: bool,
        row_index: Option<usize>,
        visible_cols: usize,
    ) -> String {
        let theme = self.get_theme();

        let cells = values
            .iter()
            .enumerate()
            .skip(state.col_offset())
            .take(visible_cols)
            .map(|(col, value)| {
                let width = layouts.get(col).map(|l| l.resolved_width).unwrap_or(8);
                let fitted = crate::layout::fit_cell(value, width);
                let is_selected_col = col == state.selected_col();
                let is_selected_cell = row_index == Some(state.selected_row()) && is_selected_col;
                let matches_search = state
                    .get_search_query()
                    .is_some_and(|query| crate::state::cell_matches_query(value, query));

                if is_header {
                    if is_selected_col {
                        let bg_color = self.get_color(&theme.selected_background);
                        let text_color = self.get_color(&theme.selected_text);
                        format!("{}{}{}\x1b[0m", bg_color, text_color, fitted)
                    } else {
                        fitted
                    }
                } else if is_selected_cell && width >= 2 {
                    let inner = crate::layout::fit_cell(value, width.saturating_sub(2));
                    format!("[{inner}]")
                } else if matches_search && width >= 2 {
                    let inner = crate::layout::fit_cell(value, width.saturating_sub(2));
                    format!("{{{inner}}}")
                } else {
                    fitted
                }
            })
            .collect::<Vec<_>>()
            .join(" | ");

        if is_header {
            cells
        } else if row_index == Some(state.selected_row()) {
            format!(">{cells}")
        } else {
            format!(" {cells}")
        }
    }

    /// Map a named color to an ANSI SGR foreground escape sequence.
    ///
    /// Accepts the eight standard ANSI color names.  Returns an empty string
    /// for `"none"`, `""`, or any unrecognized value — callers should treat
    /// an empty return as "no coloring".
    fn get_color(&self, color: &str) -> String {
        match color {
            "black" => "\x1b[30m".to_string(),
            "red" => "\x1b[31m".to_string(),
            "green" => "\x1b[32m".to_string(),
            "yellow" => "\x1b[33m".to_string(),
            "blue" => "\x1b[34m".to_string(),
            "magenta" => "\x1b[35m".to_string(),
            "cyan" => "\x1b[36m".to_string(),
            "white" => "\x1b[37m".to_string(),
            // "none", "", or anything else → suppress coloring
            _ => String::new(),
        }
    }
}
