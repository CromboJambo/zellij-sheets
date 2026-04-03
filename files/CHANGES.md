# Wiring layout.rs into zellij-sheets

## 1. Cargo.toml — add one dependency

```toml
unicode-width = "0.2"
```

Add it under `[dependencies]` alongside the existing ones.

---

## 2. src/lib.rs — expose the new module

```rust
pub mod config;
pub mod data_loader;
pub mod layout;   // ← add this
pub mod state;
pub mod ui;

pub use config::{BehaviorConfig, ColumnConfig, DisplayConfig, SheetsConfig, ThemeConfig};
pub use data_loader::{file_exists, get_file_extension, get_file_name, get_file_size, load_data};
pub use layout::{fit_cell, ColumnLayout, LayoutCache, LayoutEngine};  // ← add this
pub use state::{DataType, SheetsState};
```

---

## 3. src/state.rs — add LayoutCache field

Add the import at the top:
```rust
use crate::layout::LayoutCache;
```

Add the field to `SheetsState`:
```rust
pub struct SheetsState {
    // ... existing fields ...
    pub layout_cache: LayoutCache,   // ← add this
}
```

Initialize it in `SheetsState::new`:
```rust
layout_cache: LayoutCache::default(),
```

Populate it in `SheetsState::init` — this is the prepare phase, run once on load:
```rust
pub fn init(&mut self, data: LoadedData) -> Result<()> {
    self.headers = data.headers;
    self.rows = data.rows;
    self.selected_row = 0;
    self.scroll_row = 0;
    // ← add this line:
    self.layout_cache = LayoutCache::prepare(&self.headers, &self.rows);
    self.sync_bounds();
    Ok(())
}
```

---

## 4. src/ui.rs — use LayoutEngine and fit_cell in the renderer

Add the imports:
```rust
use crate::layout::{fit_cell, LayoutEngine};
```

Add `layout_engine` to `UiRenderer`:
```rust
pub struct UiRenderer {
    use_colors: bool,
    theme: Option<ThemeConfig>,
    layout_engine: LayoutEngine,   // ← add this
}
```

Update `UiRenderer::new` and `UiRenderer::with_theme`:
```rust
pub fn new() -> Self {
    Self {
        use_colors: true,
        theme: None,
        layout_engine: LayoutEngine::new(),   // ← add this
    }
}

pub fn with_theme(theme: ThemeConfig) -> Self {
    Self {
        use_colors: true,
        theme: Some(theme),
        layout_engine: LayoutEngine::new(),   // ← add this
    }
}
```

Replace `draw_data_rows` — pass layouts down from the resolved cache:
```rust
fn draw_data_rows(&self, lines: &mut Vec<String>, state: &SheetsState) -> Result<()> {
    let width = state.get_width().unwrap_or(80);

    // Layout phase: pure arithmetic against the prepared cache.
    let layouts = self.layout_engine.resolve(&state.layout_cache, width);

    let (start, end) = state.row_range();

    if let Some(headers) = state.headers() {
        let header_line = self.render_row(headers, state, &layouts, true)?;
        lines.push(header_line);
    }

    for row in start..end {
        if let Some(values) = state.get_row(row) {
            let is_selected = row == state.selected_row();
            let prefix = if is_selected { ">" } else { " " };
            let row_line = self.render_row(&values, state, &layouts, false)?;
            lines.push(format!("{}{}", prefix, row_line));
        }
    }

    Ok(())
}
```

Replace `render_row` — use `fit_cell` and resolved widths instead of hardcoded truncation:
```rust
fn render_row(
    &self,
    values: &[String],
    state: &SheetsState,
    layouts: &[crate::layout::ColumnLayout],
    is_header: bool,
) -> Result<String> {
    let mut cells = Vec::new();
    let theme = self.get_theme();

    for (col, value) in values.iter().enumerate() {
        let width = layouts
            .get(col)
            .map(|l| l.resolved_width)
            .unwrap_or(8);

        let fitted = fit_cell(value, width);

        let cell_value = if is_header {
            format!("\x1b[1;38;5;{}m{}\x1b[0m", theme.header_fg, fitted)
        } else {
            let data_type = state.get_data_type(col).unwrap_or(DataType::String);
            let color_code = match data_type {
                DataType::Number  => "32",
                DataType::Boolean => "35",
                DataType::Empty   => "90",
                DataType::String  => "33",
            };
            format!("\x1b[{}m{}\x1b[0m", color_code, fitted)
        };

        cells.push(cell_value);
    }

    Ok(cells.join(" | "))
}
```
