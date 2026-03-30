// Zellij Sheets - Main Entry Point
// Terminal-based spreadsheet viewer powered by Zellij's TUI capabilities

use calamine::{open_workbook, Reader, Xlsx};
use polars::prelude::*;
use std::path::Path;
use std::sync::Arc;
use zellij_term::prelude::*;

mod config;
mod data_loader;
mod state;
mod ui;

use data_loader::load_data;
use state::SheetsState;

/// Command line arguments
struct SheetsArgs {
    input_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args = parse_args()?;

    // Load data from file
    let data = load_data(&args.input_path)?;
    let headers = data
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Load configuration
    let config = config::SheetsConfig::default();

    // Create state
    let mut state = SheetsState::new(Arc::new(config));
    state.init(data, headers)?;
    state.resize(80, 24);

    // Create event handler
    let state = Arc::new(state);
    let handler = EventLoop::new(state);

    // Start the plugin
    handler.run()
}

/// Parse command line arguments
fn parse_args() -> anyhow::Result<SheetsArgs> {
    let args: Vec<String> = std::env::args().collect();
    let mut input_path: Option<PathBuf> = None;

    for (i, arg) in args.iter().enumerate() {
        if arg == "--input" && i + 1 < args.len() {
            input_path = Some(PathBuf::from(&args[i + 1]));
        }
    }

    if input_path.is_none() {
        eprintln!("Usage: zellij-sheets --input <file>");
        std::process::exit(1);
    }

    Ok(SheetsArgs {
        input_path: input_path.unwrap(),
    })
}

/// Event loop for handling plugin events
struct EventLoop {
    state: Arc<SheetsState>,
}

impl EventLoop {
    fn new(state: Arc<SheetsState>) -> Self {
        Self { state }
    }

    fn run(self) -> anyhow::Result<()> {
        let mut plugin = ZellijPlugin::new();
        plugin.set_event_handler(Box::new(move |event| self.handle_event(event)));
        plugin.start()
    }

    fn handle_event(&self, event: ZellijEvent) -> ZellijResult<Option<ZellijEvent>> {
        match event {
            ZellijEvent::Init => {
                ui::draw_ui(&self.state);
                Ok(Some(ZellijEvent::Draw))
            }
            ZellijEvent::Resize { width, height } => {
                let mut state = self.state.clone();
                state.resize(width, height);
                ui::draw_ui(&state);
                Ok(Some(ZellijEvent::Draw))
            }
            ZellijEvent::Input { key, mod_mask } => {
                let mut state = self.state.clone();
                handle_input(key, mod_mask, &mut state);
                ui::draw_ui(&state);
                Ok(Some(ZellijEvent::Draw))
            }
            ZellijEvent::Draw => {
                ui::draw_ui(&self.state);
                Ok(None)
            }
            ZellijEvent::Quit => {
                let mut state = self.state.clone();
                state.quit();
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
