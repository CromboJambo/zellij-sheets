use std::collections::BTreeMap;
use std::sync::Arc;
use zellij_sheets::{SheetsConfig, SheetsState};
use zellij_tile::prelude::*;

#[derive(Default)]
struct StateSmokePlugin {
    state: Option<SheetsState>,
}

impl ZellijPlugin for StateSmokePlugin {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        self.state = Some(SheetsState::new(Arc::new(SheetsConfig::default())));
    }

    fn render(&mut self, rows: usize, cols: usize) {
        let file_name = self
            .state
            .as_ref()
            .map(|s| s.file_name().to_string())
            .unwrap_or_else(|| "missing".to_string());
        println!("zellij-sheets state smoke");
        println!("rows={rows} cols={cols}");
        println!("file_name={file_name}");
        println!("If you can read this, the shared crate loads in wasm.");
    }
}

register_plugin!(StateSmokePlugin);
