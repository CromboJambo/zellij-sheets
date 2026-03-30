// Zellij Sheets - UI Rendering Module
// Renders a plain terminal preview from application state

use crate::state::SheetsState;

pub fn draw_ui(state: &SheetsState) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Zellij Sheets | {}", state.file_name()));
    lines.push(format!(
        "Rows: {} | Columns: {} | Selected row: {}",
        state.row_count(),
        state.col_count(),
        state.selected_row() + 1
    ));

    if let Some(headers) = state.headers() {
        lines.push(render_row(
            headers.iter().map(String::as_str).collect(),
            state,
        ));
        lines.push("-".repeat(state.get_width().unwrap_or(80).min(120)));
    }

    let (start, end) = state.row_range();
    for row in start..end {
        if let Some(values) = state.get_row(row) {
            let prefix = if row == state.selected_row() {
                ">"
            } else {
                " "
            };
            let rendered = render_row(values.iter().map(String::as_str).collect(), state);
            lines.push(format!("{prefix}{rendered}"));
        }
    }

    lines.push("-".repeat(state.get_width().unwrap_or(80).min(120)));
    lines.push("Keys: Up/Down, PgUp/PgDn, Home/End, Ctrl+C".to_string());
    lines.join("\n")
}

fn render_row(values: Vec<&str>, state: &SheetsState) -> String {
    let max_width = state
        .get_config()
        .map(|config| config.display.max_cell_length)
        .unwrap_or(24);

    values
        .into_iter()
        .map(|value| truncate(value, max_width))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn truncate(value: &str, max_width: usize) -> String {
    if value.chars().count() <= max_width {
        return value.to_string();
    }

    let truncated = value
        .chars()
        .take(max_width.saturating_sub(1))
        .collect::<String>();
    format!("{truncated}~")
}
