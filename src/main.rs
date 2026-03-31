#[cfg(target_family = "wasm")]
mod plugin;
#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;
#[cfg(not(target_family = "wasm"))]
use std::sync::Arc;

#[cfg(not(target_family = "wasm"))]
use zellij_sheets::{ui::UiRenderer, SheetsConfig, SheetsState};

#[cfg(not(target_family = "wasm"))]
struct SheetsArgs {
    input_path: PathBuf,
}

#[cfg(not(target_family = "wasm"))]
fn main() -> anyhow::Result<()> {
    let args = parse_args()?;
    let config = SheetsConfig::default();
    let mut state = SheetsState::new(Arc::new(config));
    state.load_file(args.input_path)?;

    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(100);
    let height = std::env::var("LINES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(24);
    state.resize(width, height);

    let renderer = UiRenderer::new();
    println!(
        "{}",
        renderer
            .draw_ui(&state)
            .unwrap_or_else(|e| format!("Error: {}", e))
    );
    Ok(())
}

#[cfg(not(target_family = "wasm"))]
fn parse_args() -> anyhow::Result<SheetsArgs> {
    let args: Vec<String> = std::env::args().collect();
    let mut input_path: Option<PathBuf> = None;

    for (i, arg) in args.iter().enumerate() {
        if arg == "--input" && i + 1 < args.len() {
            input_path = Some(PathBuf::from(&args[i + 1]));
        }
    }

    let input_path =
        input_path.ok_or_else(|| anyhow::anyhow!("Usage: zellij-sheets --input <file>"))?;
    Ok(SheetsArgs { input_path })
}

#[cfg(target_family = "wasm")]
use plugin::PluginState;
#[cfg(target_family = "wasm")]
use zellij_tile::{shim::report_panic, ZellijPlugin};

#[cfg(target_family = "wasm")]
zellij_tile::register_plugin!(PluginState);
