```                                                   
           _|                              _|      
   _|_|_|  _|_|_|      _|_|      _|_|    _|_|_|_|  
 _|_|      _|    _|  _|_|_|_|  _|_|_|_|    _|      
     _|_|  _|    _|  _|        _|          _|      
 _|_|_|    _|    _|    _|_|_|    _|_|_|      _|_|  
```                                                   
      
[![Crates.io](https://img.shields.io/crates/v/zellij-sheets.svg)](https://crates.io/crates/zellij-sheets) [![Docs.rs](https://docs.rs/zellij-sheets/badge.svg)](https://docs.rs/zellij-sheets) [![License: AGPL-3.0-or-later](https://img.shields.io/crates/l/zellij-sheets.svg)](https://github.com/CromboJambo/zellij-sheets/blob/main/LICENSE) [![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
                                             
A spreadsheet preview plugin for Zellij with a small native CLI for local smoke testing.

## Status

- Plugin target: `wasm32-wasip1`
- Working formats: CSV, Excel (`.xlsx`, `.xls`)
- Pending: Parquet preview

## Docs

- API docs: <https://docs.rs/zellij-sheets>
- Repository: <https://github.com/CromboJambo/zellij-sheets>

## Build

Build the native CLI:

```bash
cargo build
```

Build the Zellij plugin:

```bash
cargo build --release --target wasm32-wasip1
```

The plugin artifact is:

```text
target/wasm32-wasip1/release/zellij-sheets.wasm
```

## Install In Zellij

Add the plugin to your Zellij config:

```kdl
plugins {
    zellij-sheets location="file:/home/youruser/path/to/zellij-sheets/target/wasm32-wasip1/release/zellij-sheets.wasm"
}
```

Then use it in a layout and pass the file path through plugin configuration:

```kdl
layout {
    pane {
        plugin location="zellij-sheets" {
            input "/tmp/zellij-sheets-sample.csv"
        }
    }
}
```

An example layout is included at `layouts/spreadsheet.kdl`.
Create the sample file before launching the example layout:

```bash
printf 'a,b\n1,2\n3,4\n' > /tmp/zellij-sheets-sample.csv
```

## CLI Usage

The native binary renders a plain-text preview and is useful for quick local checks:

```bash
cargo run -- --input /absolute/path/to/data.csv
```

## Keys

- `Up` / `Down`: move selection
- `PageUp` / `PageDown`: move by page
- `Home` / `End`: jump to start/end
- `q` or `Ctrl-C`: close the plugin

## Notes

- The plugin requests hard-drive access and remaps the configured file path through `/host/...`, matching Zellij's host filesystem model.
- The shared data layer was rebuilt to be wasm-safe, so the plugin and CLI now use the same lightweight grid model.
