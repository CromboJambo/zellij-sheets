// Zellij Sheets - UI Rendering Module
// Handles UI rendering and styling for the spreadsheet viewer

use crate::config::SheetsConfig;
use crate::state::SheetsState;
use zellij_term::prelude::*;

/// Draw the complete UI
pub fn draw_ui(state: &SheetsState) {
    let mut screen = Screen::default();

    draw_header(&mut screen, state);
    draw_grid(&mut screen, state);
    draw_footer(&mut screen, state);
}

/// Draw the header section
pub fn draw_header(screen: &mut Screen, state: &SheetsState) {
    let theme = &state.config.theme;

    // Draw title bar
    let title_style = Style::default()
        .fg(Color::Rgb(0, 120, 215))
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    let title = format!("📊 Zellij Sheets - {}", state.file_name());
    screen.draw_string(0, 0, &title, title_style);

    // Draw info bar
    let info_style = Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0));

    let info = format!(
        "{} rows × {} columns | Selected: Row {}",
        state.row_count(),
        state.col_count(),
        state.selected_row() + 1
    );
    screen.draw_string(0, 1, &info, info_style);

    // Draw column headers
    if let Some(headers) = state.headers() {
        let header_style = Style::default()
            .fg(Color::Cyan)
            .bg(Color::Rgb(0, 0, 0))
            .add_modifier(Modifier::BOLD);

        let header_x = 0;
        let header_y = 2;

        for (col, header) in headers.iter().enumerate() {
            screen.draw_string(header_x, header_y, header, header_style);
            header_x += header.len() + 1;
        }
    }
}

/// Draw the main data grid
pub fn draw_grid(screen: &mut Screen, state: &SheetsState) {
    let theme = &state.config.theme;

    // Grid styles
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

/// Draw the footer section
pub fn draw_footer(screen: &mut Screen, state: &SheetsState) {
    let theme = &state.config.theme;

    // Draw footer bar
    let footer_style = Style::default()
        .fg(Color::Rgb(0, 120, 215))
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    let footer = format!(
        "Navigation: ↑↓←→ | Page: PGUP/PGDN | Home: HOME | End: END | Quit: CTRL+C | Search: /"
    );

    screen.draw_string(0, state.height() - 1, &footer, footer_style);
}

/// Draw a cell with appropriate styling
pub fn draw_cell(
    screen: &mut Screen,
    x: usize,
    y: usize,
    value: &str,
    state: &SheetsState,
    col: usize,
) {
    let theme = &state.config.theme;
    let display = &state.config.display;

    // Truncate long values if needed
    let display_value = if display.truncate_long_values && value.len() > display.max_cell_length {
        &value[..display.max_cell_length]
    } else {
        value
    };

    // Determine cell style based on data type and selection
    let style = if col == state.selected_row() {
        // Selected row styling
        Style::default()
            .fg(Color::White)
            .bg(Color::Rgb(0, 100, 200))
    } else {
        // Normal cell styling
        Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0))
    };

    screen.draw_string(x, y, display_value, style);
}

/// Draw a status message
pub fn draw_status_message(screen: &mut Screen, message: &str, state: &SheetsState) {
    let theme = &state.config.theme;

    let status_style = Style::default()
        .fg(Color::Rgb(0, 200, 100))
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    let status_x = 0;
    let status_y = state.height() - 2;

    screen.draw_string(status_x, status_y, message, status_style);
}

/// Draw a warning message
pub fn draw_warning_message(screen: &mut Screen, message: &str, state: &SheetsState) {
    let theme = &state.config.theme;

    let warning_style = Style::default()
        .fg(Color::Rgb(200, 100, 0))
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    let warning_x = 0;
    let warning_y = state.height() - 2;

    screen.draw_string(warning_x, warning_y, message, warning_style);
}

/// Draw an error message
pub fn draw_error_message(screen: &mut Screen, message: &str, state: &SheetsState) {
    let theme = &state.config.theme;

    let error_style = Style::default()
        .fg(Color::Rgb(200, 50, 50))
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    let error_x = 0;
    let error_y = state.height() - 2;

    screen.draw_string(error_x, error_y, message, error_style);
}

/// Draw a progress indicator
pub fn draw_progress(screen: &mut Screen, progress: f32, state: &SheetsState) {
    let theme = &state.config.theme;

    // Calculate bar width
    let bar_width = state.width as f32 * progress;

    let bar_style = Style::default()
        .fg(Color::Rgb(0, 200, 100))
        .bg(Color::Rgb(0, 0, 0));

    let bar_x = 0;
    let bar_y = state.height() - 2;

    // Draw progress bar
    for x in 0..bar_width as usize {
        screen.draw_string(x, bar_y, "█", bar_style);
    }

    // Draw percentage
    let percentage = format!("{:.1}%", progress * 100.0);
    let percent_style = Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0));

    screen.draw_string(bar_width as usize + 1, bar_y, &percentage, percent_style);
}

/// Draw a separator line
pub fn draw_separator(screen: &mut Screen, y: usize, state: &SheetsState) {
    let theme = &state.config.theme;

    let separator_style = Style::default()
        .fg(Color::Rgb(0, 120, 215))
        .bg(Color::Rgb(0, 0, 0));

    let separator = "─".repeat(state.width);
    screen.draw_string(0, y, &separator, separator_style);
}

/// Draw a help section
pub fn draw_help(screen: &mut Screen, state: &SheetsState) {
    let theme = &state.config.theme;

    let help_style = Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0));

    let help_title_style = Style::default()
        .fg(Color::Cyan)
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    let help_x = 0;
    let help_y = state.height() / 2;

    // Draw help title
    screen.draw_string(help_x, help_y, "Key Bindings:", help_title_style);

    // Draw help entries
    let help_entries = vec![
        ("Arrow Keys", "Navigate"),
        ("Home/End", "Go to top/bottom"),
        ("PageUp/PageDown", "Scroll by page"),
        ("/", "Search"),
        ("q", "Quit"),
    ];

    for (i, (key, description)) in help_entries.iter().enumerate() {
        let entry_style = Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0));

        screen.draw_string(
            help_x,
            help_y + i + 1,
            &format!("  {}: {}", key, description),
            entry_style,
        );
    }
}

/// Draw a data type indicator
pub fn draw_data_type_indicator(
    screen: &mut Screen,
    x: usize,
    y: usize,
    data_type: &str,
    state: &SheetsState,
) {
    let theme = &state.config.theme;

    let indicator_style = Style::default()
        .fg(Color::Cyan)
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    let indicator = format!("[{}]", data_type);
    screen.draw_string(x, y, &indicator, indicator_style);
}

/// Clear a section of the screen
pub fn clear_section(
    screen: &mut Screen,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    state: &SheetsState,
) {
    let clear_style = Style::default().bg(Color::Rgb(0, 0, 0));

    for row in y..(y + height) {
        for col in x..(x + width) {
            screen.draw_string(col, row, " ", clear_style);
        }
    }
}

/// Draw a border around a section
pub fn draw_border(
    screen: &mut Screen,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    state: &SheetsState,
) {
    let theme = &state.config.theme;

    let border_style = Style::default().fg(Color::Cyan).bg(Color::Rgb(0, 0, 0));

    // Top border
    for col in x..(x + width) {
        screen.draw_string(col, y, "─", border_style);
    }

    // Bottom border
    for col in x..(x + width) {
        screen.draw_string(col, y + height - 1, "─", border_style);
    }

    // Left and right borders
    for row in (y + 1)..(y + height - 1) {
        screen.draw_string(x, row, "│", border_style);
        screen.draw_string(x + width - 1, row, "│", border_style);
    }

    // Corners
    screen.draw_string(x, y, "┌", border_style);
    screen.draw_string(x + width - 1, y, "┐", border_style);
    screen.draw_string(x, y + height - 1, "└", border_style);
    screen.draw_string(x + width - 1, y + height - 1, "┘", border_style);
}

/// Render a loading screen
pub fn draw_loading_screen(screen: &mut Screen, message: &str, state: &SheetsState) {
    let theme = &state.config.theme;

    let loading_style = Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0));

    let message_style = Style::default()
        .fg(Color::Cyan)
        .bg(Color::Rgb(0, 0, 0))
        .add_modifier(Modifier::BOLD);

    // Clear screen
    clear_section(screen, 0, 0, state.width, state.height, state);

    // Draw centered message
    let message_x = (state.width / 2) - (message.len() / 2);
    let message_y = state.height / 2;

    screen.draw_string(message_x, message_y, message, message_style);

    // Draw loading dots
    let dots = format!("{}...", ".".repeat((state.width / 4) as usize % 3));
    let dots_x = (state.width / 2) - (dots.len() / 2);

    screen.draw_string(dots_x, message_y + 1, dots, loading_style);
}
