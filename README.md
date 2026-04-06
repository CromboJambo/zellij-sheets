```                                                   
           _|                              _|      
   _|_|_|  _|_|_|      _|_|      _|_|    _|_|_|_|  
 _|_|      _|    _|  _|_|_|_|  _|_|_|_|    _|      
     _|_|  _|    _|  _|        _|          _|      
 _|_|_|    _|    _|    _|_|_|    _|_|_|      _|_|  
```                                                   

[![Crates.io](https://img.shields.io/crates/v/zellij-sheets.svg)](https://crates.io/crates/zellij-sheets) [![Docs.rs](https://docs.rs/zellij-sheets/badge.svg)](https://docs.rs/zellij-sheets) [![License: AGPL-3.0-or-later](https://img.shields.io/crates/l/zellij-sheets.svg)](https://github.com/CromboJambo/zellij-sheets/blob/main/LICENSE) [![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)

Terminal-native spreadsheet viewing for Zellij and the command line.

`zellij-sheets` is the grid/navigation layer: load tabular data, move around it quickly, search it, and inspect or update CSV cells from a native CLI. It is intentionally viewer-first. Workflow-level pipeline semantics belong in `nustage`, not here.

## What It Does

- Zellij plugin for spreadsheet browsing
- Native CLI for previewing files or stdin
- Cell, range, and CSV write addressing from the CLI
- Horizontal scrolling with a real column cursor
- Vim-style navigation
- Search with `/`, `?`, `n`, and `N`
- CSV and Excel (`.xlsx`, `.xls`) loading
- Unicode-aware column layout

## What It Is Not

- Not a workflow or transformation pipeline engine
- Not a formula system
- Not a sidecar/provenance tool
- Not a replacement for `nustage`

## Status

- Native CLI: usable for preview, stdin, cell/range reads, and CSV writes
- Zellij plugin: usable interactive viewer
- Working formats: CSV, Excel (`.xlsx`, `.xls`)
- Parquet: not supported yet
- Native full-screen interactive TUI: next major step

## Build

Build the native binary:

```bash
cargo build
```

Build the Zellij plugin:

```bash
cargo build --release --target wasm32-wasip1
```

Build the optional smoke-test plugin binaries:

```bash
cargo build --target wasm32-wasip1 --features plugin-smoke --bin plugin-smoke --bin plugin-state-smoke
```

The plugin artifact is:

```text
target/wasm32-wasip1/release/zellij-sheets.wasm
```

## Native CLI

Show help:

```bash
cargo run -- --help
```

Preview a file:

```bash
cargo run -- data.csv
```

Read a single cell:

```bash
cargo run -- data.csv B9
```

Read a range:

```bash
cargo run -- data.csv B1:B3
```

Write a CSV cell:

```bash
cargo run -- data.csv B7=10
```

Read CSV from stdin:

```bash
cat data.csv | cargo run -- B2
```

Or explicitly use `-`:

```bash
cat data.csv | cargo run -- - B2
```

## Zellij Install

Add the plugin to your Zellij config:

```kdl
plugins {
    zellij-sheets location="file:/home/youruser/path/to/zellij-sheets/target/wasm32-wasip1/release/zellij-sheets.wasm"
}
```

Then use it in a layout and pass the input file through plugin configuration:

```kdl
layout {
    pane {
        plugin location="zellij-sheets" {
            input "/tmp/zellij-sheets-sample.csv"
        }
    }
}
```

An example alias-based layout is included at `layouts/spreadsheet.kdl`. Create a sample file before launching it:

```bash
printf 'a,b\n1,2\n3,4\n' > /tmp/zellij-sheets-sample.csv
```

## Keys

Current plugin/native behavior is centered around the shared state model.

- `Up` / `Down` / `Left` / `Right`: move selection
- `h` / `j` / `k` / `l`: vim movement
- `gg` / `G`: jump to top / bottom
- `Ctrl-U` / `Ctrl-D`: half-page up / down
- `0` / `$`: first / last column
- `H` / `M` / `L`: top / middle / bottom visible row
- `/` / `?`: start forward / backward search
- `n` / `N`: next / previous search result
- `Esc`: cancel search
- `q` or `Ctrl-C`: close the plugin

## Repo Boundary

Keep `zellij-sheets` focused on:

- rendering
- cursoring
- scrolling
- search
- addressing
- light file I/O
- plugin/native TUI behavior

If a feature is primarily about named transformations, reproducible step pipelines, provenance, execution planning, or workflow state, it belongs in `nustage`.

## Docs

- API docs: <https://docs.rs/zellij-sheets>
- Repository: <https://github.com/CromboJambo/zellij-sheets>

## Notes

- The plugin requests hard-drive access and works through Zellij’s host filesystem model.
- The shared state/rendering path is used by both the native binary and the plugin.
- `cargo test --all-features` may still expose feature-gated smoke-test issues unrelated to the core viewer path; the main development loop also uses targeted `cargo test --lib`, integration tests, `cargo check`, and wasm builds.
