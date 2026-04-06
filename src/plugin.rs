use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use zellij_sheets::{
    cell_matches_query, fit_cell, ColumnLayout, LayoutEngine, SearchDirection, SheetsConfig,
    SheetsState,
};
use zellij_tile::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Search,
}

#[derive(Default)]
pub struct PluginState {
    sheets: SheetsState,
    input_path: Option<PathBuf>,
    status: Option<String>,
    pending_key: Option<BareKey>,
    input_mode: InputMode,
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

        match self.sheets.load_file(input_path.clone()) {
            Ok(()) => {
                self.status = None;
                self.pending_key = None;
                self.input_mode = InputMode::Normal;
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
        if self.input_mode == InputMode::Search {
            return self.handle_search_key(key);
        }

        if self.handle_pending_key(&key) {
            return true;
        }

        match key.bare_key {
            BareKey::Down => {
                self.sheets.select_down();
                true
            }
            BareKey::Up => {
                self.sheets.select_up();
                true
            }
            BareKey::Left => {
                self.sheets.select_left();
                true
            }
            BareKey::Right => {
                self.sheets.select_right();
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
            BareKey::Char(character)
                if character.eq_ignore_ascii_case(&'u')
                    && key.has_only_modifiers(&[KeyModifier::Ctrl]) =>
            {
                self.sheets.half_page_up();
                true
            }
            BareKey::Char(character)
                if character.eq_ignore_ascii_case(&'d')
                    && key.has_only_modifiers(&[KeyModifier::Ctrl]) =>
            {
                self.sheets.half_page_down();
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
            BareKey::Char('/') if key.has_no_modifiers() => {
                self.pending_key = None;
                self.input_mode = InputMode::Search;
                self.sheets.begin_search(SearchDirection::Forward);
                true
            }
            BareKey::Char('?') if key.has_no_modifiers() => {
                self.pending_key = None;
                self.input_mode = InputMode::Search;
                self.sheets.begin_search(SearchDirection::Backward);
                true
            }
            BareKey::Char(character) if character == 'h' && key.has_no_modifiers() => {
                self.sheets.select_left();
                true
            }
            BareKey::Char(character) if character == 'j' && key.has_no_modifiers() => {
                self.sheets.select_down();
                true
            }
            BareKey::Char(character) if character == 'n' && key.has_no_modifiers() => {
                self.sheets.search_next();
                true
            }
            BareKey::Char(character) if character == 'g' && key.has_no_modifiers() => {
                self.pending_key = Some(BareKey::Char('g'));
                true
            }
            BareKey::Char(character)
                if character.eq_ignore_ascii_case(&'g')
                    && key.has_only_modifiers(&[KeyModifier::Shift]) =>
            {
                self.sheets.go_to_bottom();
                true
            }
            BareKey::Char(character) if character == 'k' && key.has_no_modifiers() => {
                self.sheets.select_up();
                true
            }
            BareKey::Char(character) if character == 'l' && key.has_no_modifiers() => {
                self.sheets.select_right();
                true
            }
            BareKey::Char(character)
                if character.eq_ignore_ascii_case(&'n')
                    && key.has_only_modifiers(&[KeyModifier::Shift]) =>
            {
                self.sheets.search_prev();
                true
            }
            BareKey::Char(character)
                if character.eq_ignore_ascii_case(&'h')
                    && key.has_only_modifiers(&[KeyModifier::Shift]) =>
            {
                self.sheets.go_to_top_visible();
                true
            }
            BareKey::Char(character)
                if character.eq_ignore_ascii_case(&'m')
                    && key.has_only_modifiers(&[KeyModifier::Shift]) =>
            {
                self.sheets.go_to_middle_visible();
                true
            }
            BareKey::Char(character)
                if character.eq_ignore_ascii_case(&'l')
                    && key.has_only_modifiers(&[KeyModifier::Shift]) =>
            {
                self.sheets.go_to_bottom_visible();
                true
            }
            BareKey::Char('0') if key.has_no_modifiers() => {
                self.sheets.go_to_first_col();
                true
            }
            BareKey::Char('$') if key.has_no_modifiers() => {
                self.sheets.go_to_last_col();
                true
            }
            BareKey::Char('4') if key.has_only_modifiers(&[KeyModifier::Shift]) => {
                self.sheets.go_to_last_col();
                true
            }
            BareKey::Esc => {
                self.pending_key = None;
                true
            }
            BareKey::Char(character)
                if character.eq_ignore_ascii_case(&'c')
                    && key.has_only_modifiers(&[KeyModifier::Ctrl]) =>
            {
                close_self();
                false
            }
            _ => false,
        }
    }

    fn handle_search_key(&mut self, key: KeyWithModifier) -> bool {
        match key.bare_key {
            BareKey::Esc => {
                self.sheets.search_cancel();
                self.input_mode = InputMode::Normal;
                true
            }
            BareKey::Enter => {
                self.sheets.search_commit();
                self.input_mode = InputMode::Normal;
                true
            }
            BareKey::Backspace => {
                self.sheets.search_backspace();
                true
            }
            BareKey::Char(character)
                if key.has_no_modifiers() || key.has_only_modifiers(&[KeyModifier::Shift]) =>
            {
                self.sheets.search_append(character);
                true
            }
            _ => false,
        }
    }

    fn handle_pending_key(&mut self, key: &KeyWithModifier) -> bool {
        match self.pending_key.take() {
            Some(BareKey::Char('g'))
                if matches!(key.bare_key, BareKey::Char('g')) && key.has_no_modifiers() =>
            {
                self.sheets.go_to_top();
                true
            }
            Some(_) => false,
            None => false,
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
        let visible_cols = self.sheets.visible_cols();
        let file_info = format!(
            "Zellij Sheets  {}  {} rows  r{} c{}",
            self.sheets.file_name(),
            self.sheets.row_count(),
            self.sheets.selected_row() + 1,
            self.sheets.selected_col() + 1,
        );
        println!("{}", file_info);
        println!("{}", "-".repeat(cols));

        if let Some(headers) = self.sheets.headers() {
            println!(
                "{}",
                build_row(headers, &self.sheets, &layouts, true, None, visible_cols,)
            );
        }

        if self.sheets.row_count() == 0 {
            println!("File loaded, but it does not contain any data rows.");
        }

        let (start, end) = self.sheets.row_range();
        for row_idx in start..end {
            if let Some(values) = self.sheets.get_row(row_idx) {
                println!(
                    "{}",
                    build_row(
                        &values,
                        &self.sheets,
                        &layouts,
                        false,
                        Some(row_idx),
                        visible_cols,
                    )
                );
            }
        }

        println!("{}", "-".repeat(cols));
        if self.input_mode == InputMode::Search {
            let prefix = match self.sheets.search_direction() {
                SearchDirection::Forward => '/',
                SearchDirection::Backward => '?',
            };
            let query = self
                .sheets
                .get_search_query()
                .ok()
                .flatten()
                .unwrap_or_default();
            println!("Search: {prefix}{query}  Enter=commit Esc=cancel Backspace=delete");
        } else {
            println!("Keys: Arrows hjkl gg/G H/M/L 0/$ / ? n N Ctrl-U/D q/Ctrl-C");
        }
    }
}

/// Build a single display row.
///
/// - `is_header`: plain text
/// - `is_selected`: prefixed with `>`
/// - plain data rows: prefixed with a space
fn build_row(
    values: &[String],
    sheets: &SheetsState,
    layouts: &[ColumnLayout],
    is_header: bool,
    row_index: Option<usize>,
    visible_cols: usize,
) -> String {
    let cells = values
        .iter()
        .enumerate()
        .skip(sheets.col_offset())
        .take(visible_cols)
        .map(|(col, value)| {
            let width = layouts.get(col).map(|l| l.resolved_width).unwrap_or(8);
            let fitted = fit_cell(value, width);
            let matches_search = sheets
                .get_search_query()
                .ok()
                .flatten()
                .is_some_and(|query| cell_matches_query(value, &query));

            if col == sheets.selected_col()
                && (is_header || row_index == Some(sheets.selected_row()))
                && width >= 2
            {
                let inner = fit_cell(value, width.saturating_sub(2));
                format!("[{inner}]")
            } else if !is_header && matches_search && width >= 2 {
                let inner = fit_cell(value, width.saturating_sub(2));
                format!("{{{inner}}}")
            } else {
                fitted
            }
        })
        .collect::<Vec<_>>()
        .join(" | ");

    if is_header {
        cells
    } else if row_index == Some(sheets.selected_row()) {
        format!(">{cells}")
    } else {
        format!(" {cells}")
    }
}
