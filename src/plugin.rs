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
        subscribe(&[
            EventType::Key,
            EventType::PermissionRequestResult,
            EventType::HostFolderChanged,
            EventType::FailedToChangeHostFolder,
        ]);
        set_selectable(true);
        self.initialize_from_config(configuration);

        if self.input_path.is_some() {
            request_permission(&[PermissionType::FullHdAccess]);
        }
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.status = Some("Permission granted. Loading file...".to_string());
                self.load_input();
                true
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                self.status =
                    Some("Permission denied. This plugin needs hard-drive access.".to_string());
                true
            }
            Event::HostFolderChanged(path) => {
                self.status = Some(format!("Host folder changed to {}", path.display()));
                true
            }
            Event::FailedToChangeHostFolder(error) => {
                self.status = Some(format!(
                    "Failed to change host folder: {}",
                    error.unwrap_or_else(|| "unknown error".to_string())
                ));
                true
            }
            Event::Key(key) => self.handle_key(key),
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        self.sheets.resize(cols, rows);

        if let Some(status) = &self.status {
            println!("Zellij Sheets");
            println!();
            println!("{}", status);
            println!();
            println!("Use plugin config: input=\"/absolute/path/to/file.csv\"");
            return;
        }

        let engine = LayoutEngine::new();
        let layouts = engine.resolve(&self.sheets.layout_cache, cols);
        let file_info = format!(
            "Zellij Sheets  {}  {} rows",
            self.sheets.file_name(),
            self.sheets.row_count(),
        );
        println!("{}", file_info);
        println!("{}", "-".repeat(cols));

        if let Some(headers) = self.sheets.headers() {
            println!("{}", build_row(headers, &layouts, true, false));
        }

        if self.sheets.row_count() == 0 {
            println!("File loaded, but it does not contain any data rows.");
        }

        let (start, end) = self.sheets.row_range();
        for row_idx in start..end {
            if let Some(values) = self.sheets.get_row(row_idx) {
                let is_selected = row_idx == self.sheets.selected_row();
                println!("{}", build_row(&values, &layouts, false, is_selected));
            }
        }

        println!("{}", "-".repeat(cols));
        println!("Keys: Up/Down  PgUp/PgDn  Home/End  q/Ctrl-C");
    }
}

fn to_host_path(path: &Path) -> PathBuf {
    if path.starts_with("/host") {
        return path.to_path_buf();
    }

    let relative = path.strip_prefix("/").unwrap_or(path);
    Path::new("/host").join(relative)
}

/// Build a single display row.
///
/// - `is_header`: plain text
/// - `is_selected`: prefixed with `>`
/// - plain data rows: prefixed with a space
fn build_row(
    values: &[String],
    layouts: &[ColumnLayout],
    is_header: bool,
    is_selected: bool,
) -> String {
    let cells = values
        .iter()
        .enumerate()
        .map(|(col, value)| {
            let width = layouts.get(col).map(|l| l.resolved_width).unwrap_or(8);
            fit_cell(value, width)
        })
        .collect::<Vec<_>>()
        .join(" | ");

    if is_header {
        cells
    } else if is_selected {
        format!(">{cells}")
    } else {
        format!(" {cells}")
    }
}
