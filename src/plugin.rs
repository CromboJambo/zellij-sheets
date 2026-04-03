use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use zellij_sheets::{fit_cell, ColumnLayout, LayoutEngine, SheetsConfig, SheetsState};
use zellij_tile::prelude::*;

#[derive(Default)]
pub struct PluginState {
    sheets: SheetsState,
    input_path: Option<PathBuf>,
    status: Option<String>,
}

impl PluginState {
    fn initialize_from_config(&mut self, configuration: BTreeMap<String, String>) {
        self.sheets = SheetsState::new(Arc::new(SheetsConfig::default()));
        self.input_path = configuration
            .get("input")
            .or_else(|| configuration.get("input_path"))
            .map(PathBuf::from);

        self.status = match &self.input_path {
            Some(path) => Some(format!("Waiting for permission to open {}", path.display())),
            None => {
                Some("Set plugin config `input` or `input_path` to a spreadsheet file.".to_string())
            }
        };
    }

    fn load_input(&mut self) {
        let Some(input_path) = self.input_path.clone() else {
            return;
        };

        let host_path = to_host_path(&input_path);
        match self.sheets.load_file(host_path) {
            Ok(()) => {
                self.status = None;
            }
            Err(error) => {
                self.status = Some(format!(
                    "Failed to load {}: {}",
                    input_path.display(),
                    error
                ));
            }
        }
    }

    fn handle_key(&mut self, key: KeyWithModifier) -> bool {
        match key.bare_key {
            BareKey::Down => {
                self.sheets.select_down();
                true
            }
            BareKey::Up => {
                self.sheets.select_up();
                true
            }
            BareKey::PageDown => {
                self.sheets.page_down();
                true
            }
            BareKey::PageUp => {
                self.sheets.page_up();
                true
            }
            BareKey::Home => {
                self.sheets.go_to_top();
                true
            }
            BareKey::End => {
                self.sheets.go_to_bottom();
                true
            }
            BareKey::Char('q') if key.has_no_modifiers() => {
                close_self();
                false
            }
            BareKey::Char('c') if key.has_only_modifiers(&[KeyModifier::Ctrl]) => {
                close_self();
                false
            }
            _ => false,
        }
    }
}

impl ZellijPlugin for PluginState {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        subscribe(&[EventType::Key, EventType::PermissionRequestResult]);
        set_selectable(true);
        self.initialize_from_config(configuration);

        if self.input_path.is_some() {
            request_permission(&[
                PermissionType::ChangeApplicationState,
                PermissionType::FullHdAccess,
            ]);
        }
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                change_host_folder(PathBuf::from("/"));
                self.load_input();
                true
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                self.status =
                    Some("Permission denied. This plugin needs hard-drive access.".to_string());
                true
            }
            Event::Key(key) => self.handle_key(key),
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        self.sheets.resize(cols, rows);

        if let Some(status) = &self.status {
            print_text_with_coordinates(
                Text::new("Zellij Sheets").color_all(0),
                0,
                0,
                Some(cols),
                None,
            );
            print_text_with_coordinates(Text::new(status), 0, 2, Some(cols), None);
            print_text_with_coordinates(
                Text::new("Use plugin config: input=\"/absolute/path/to/file.csv\"").dim_all(),
                0,
                4,
                Some(cols),
                None,
            );
            return;
        }

        let engine = LayoutEngine::new();
        let layouts = engine.resolve(&self.sheets.layout_cache, cols);

        let mut y = 0;

        // Header bar
        let file_info = format!(
            "Zellij Sheets  {}  {} rows",
            self.sheets.file_name(),
            self.sheets.row_count(),
        );
        print_text_with_coordinates(
            Text::new(&file_info).selected().color_all(0),
            0,
            y,
            Some(cols),
            None,
        );
        y += 1;

        // Separator
        print_text_with_coordinates(
            Text::new("─".repeat(cols)).dim_all(),
            0,
            y,
            Some(cols),
            None,
        );
        y += 1;

        // Column headers row
        if let Some(headers) = self.sheets.headers() {
            let row: Vec<Text> = build_row(headers, &layouts, true, false);
            print_table_with_coordinates(
                Table::new().add_styled_row(row),
                0,
                y,
                Some(cols),
                None,
            );
            y += 1;
        }

        // Data rows
        let (start, end) = self.sheets.row_range();
        for row_idx in start..end {
            if let Some(values) = self.sheets.get_row(row_idx) {
                let is_selected = row_idx == self.sheets.selected_row();
                let row: Vec<Text> = build_row(&values, &layouts, false, is_selected);
                print_table_with_coordinates(
                    Table::new().add_styled_row(row),
                    0,
                    y,
                    Some(cols),
                    None,
                );
                y += 1;
            }
        }

        // Footer
        let footer = "Keys: ↑/↓  PgUp/PgDn  Home/End  q/Ctrl-C";
        print_text_with_coordinates(
            Text::new(footer).dim_all(),
            0,
            rows.saturating_sub(1),
            Some(cols),
            None,
        );
    }
}

fn to_host_path(path: &Path) -> PathBuf {
    if path.starts_with("/host") {
        return path.to_path_buf();
    }

    let relative = path.strip_prefix("/").unwrap_or(path);
    Path::new("/host").join(relative)
}

/// Build a single `Vec<Text>` row for `print_table_with_coordinates`.
///
/// - `is_header`: bold + color_all(0) (uses theme primary color)
/// - `is_selected`: selected() highlight
/// - plain data rows: unstyled
fn build_row(
    values: &[String],
    layouts: &[ColumnLayout],
    is_header: bool,
    is_selected: bool,
) -> Vec<Text> {
    values
        .iter()
        .enumerate()
        .map(|(col, value)| {
            let width = layouts.get(col).map(|l| l.resolved_width).unwrap_or(8);
            let fitted = fit_cell(value, width);
            let text = Text::new(fitted);
            let text = if is_header {
                text.color_all(0)
            } else if is_selected {
                text.selected()
            } else {
                text
            };
            text
        })
        .collect()
}
