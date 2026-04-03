use std::collections::BTreeMap;
use zellij_tile::prelude::*;

#[derive(Default)]
struct SmokePlugin;

impl ZellijPlugin for SmokePlugin {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {}

    fn render(&mut self, rows: usize, cols: usize) {
        println!("zellij-sheets smoke plugin");
        println!("rows={rows} cols={cols}");
        println!("If you can read this, plugin rendering works.");
    }
}

register_plugin!(SmokePlugin);
