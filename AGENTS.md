# Repository Guidelines

## Project Structure

```
zellij-sheets/
├── src/
│   ├── lib.rs              # Public API surface — re-exports from all modules
│   ├── main.rs             # Native CLI entry point (file arg, cell addressing, stdin)
│   ├── plugin.rs           # Zellij plugin entry point (WASM target)
│   ├── config.rs           # SheetsConfig, ThemeConfig, DisplayConfig, BehaviorConfig
│   ├── data_loader.rs      # CSV / Excel file loading and parsing
│   ├── layout.rs           # Column layout engine and rendering cache
│   ├── state.rs            # SheetsState — core spreadsheet state machine
│   ├── ui.rs               # UiRenderer, Colors, UiError
│   └── bin/
│       ├── plugin-smoke.rs        # Plugin smoke-test binary (feature-gated)
│       └── plugin-state-smoke.rs  # State smoke-test binary (feature-gated)
├── tests/                  # Integration test suite (separate Cargo.toml)
│   ├── data_loader_tests.rs
│   ├── state_tests.rs
│   └── ui_tests.rs
├── config/config.toml      # Default runtime configuration
├── layouts/                # Zellij KDL layout files
└── .github/workflows/      # CI pipelines (test, fmt, clippy, wasm)
```

**Two compilation targets exist side by side:**
- `src/main.rs` — native CLI binary, built with `cargo build`
- `src/plugin.rs` — Zellij WASM plugin, built with `--target wasm32-wasip1`

All public types are re-exported from `src/lib.rs`. Each module owns one concern; cross-module logic belongs in `state.rs` or `lib.rs`.

---

## Planned Features (in priority order)

These are the features being actively developed to close the gap with
[maaslalani/sheets](https://github.com/maaslalani/sheets). Each item below
corresponds to a concrete area of the codebase.

### 1. Column cursor + horizontal scrolling (`state.rs`, `ui.rs`, `plugin.rs`)

`SheetsState` needs a `selected_col: usize` and a `col_offset: usize` field
alongside the existing `selected_row` / `scroll_row`. The layout engine already
produces a `Vec<ColumnLayout>`; the renderer must slice it starting at
`col_offset` so wide tables scroll horizontally. The currently-empty
`scroll_left` / `scroll_right` stubs in `state.rs` must be implemented.

- `selected_col` — zero-based index of the active column.
- `col_offset` — first visible column (horizontal scroll position).
- `max_col_offset` — computed from column count minus visible columns; clamped
  on resize.
- `select_left()` / `select_right()` — move `selected_col`, advance
  `col_offset` when the cursor hits the right edge.
- The renderer highlights the intersection of `selected_row` × `selected_col`,
  not just the entire row.

### 2. Vim-style navigation (`state.rs`, `plugin.rs`)

Minimum viable key set (plugin and CLI):

| Key | Action |
|---|---|
| `h` / `l` | `select_left()` / `select_right()` |
| `j` / `k` | `select_down()` / `select_up()` |
| `g g` | `go_to_top()` |
| `G` | `go_to_bottom()` |
| `ctrl+u` | `half_page_up()` |
| `ctrl+d` | `half_page_down()` |
| `0` | `go_to_first_col()` |
| `$` | `go_to_last_col()` |
| `H` / `M` / `L` | jump to top / middle / bottom of visible rows |

Multi-key sequences (`gg`, `5G`, `gB9`) require a small pending-key buffer in
`PluginState`. Store it as `pending_key: Option<BareKey>` and flush it on the
next keypress or a timeout event.

### 3. Search (`state.rs`, `plugin.rs`, `ui.rs`)

The `search_query` field already exists in `SheetsState`. Wire it up:

- `begin_search(direction: SearchDirection)` — sets an `InputMode::Search`
  flag and clears the current query.
- `search_append(ch: char)` / `search_commit()` / `search_cancel()` — driven
  by keypresses while in search mode.
- `search_next()` / `search_prev()` — scan rows × cols from the current
  cursor, wrap around, jump `selected_row` / `selected_col` to the match.
- The status bar renders the query as `/pattern` while in search mode and
  highlights matches in the data area.
- Keys: `/` (forward), `?` (backward), `n` (next), `N` (previous), `Esc`
  (cancel).

### 4. Horizontal scroll in the renderer (`ui.rs`)

`build_row` (and the header row) must accept a `col_offset: usize` and only
render columns `col_offset..col_offset+visible_cols`. The separator between
the row-number gutter and the first visible column must be preserved. This is
purely a rendering change — no state changes required beyond feature 1.

### 5. Cell addressing from the CLI (`main.rs`)

Extend the argument parser to accept an optional positional address after the
file path:

```
zellij-sheets file.csv          # interactive TUI (future) or static dump
zellij-sheets file.csv B9       # print the value of cell B9 to stdout
zellij-sheets file.csv B1:B3    # print the range B1:B3, one value per line
zellij-sheets file.csv B7=10    # write a value (CSV only, round-trips the file)
```

Column letters map to zero-based indices with `col_letter_to_index` (A=0,
Z=25, AA=26, …). Row numbers are 1-based in the address but 0-based internally.
Implement the address parser in a new `src/address.rs` module and re-export it
from `lib.rs`.

### 6. stdin support (`main.rs`, `data_loader.rs`)

When no file argument is given (or `-` is passed), read CSV from stdin:

```bash
zellij-sheets <<< "ID,Name,Age\n1,Alice,24"
cat data.csv | zellij-sheets
```

Detect via `std::io::stdin().is_terminal()` (requires the `is-terminal` crate
or `rustix`). If not a terminal, read stdin to a `String` and pass it through
a new `load_csv_from_reader(r: impl Read)` function in `data_loader.rs`.

---

## Build & Development Commands

| Command | What it does |
|---|---|
| `cargo check` | Verify the project compiles without producing artifacts |
| `cargo build` | Build the native CLI binary (debug) |
| `cargo build --release --target wasm32-wasip1` | Build the Zellij WASM plugin |
| `cargo test --all-features` | Run the full test suite |
| `cargo fmt` | Format all source files |
| `cargo clippy -- -D warnings` | Lint — all warnings are treated as errors |
| `cargo run --bin plugin-smoke --features plugin-smoke` | Run the plugin smoke-test binary |
| `cargo run --bin plugin-state-smoke --features plugin-smoke` | Run the state smoke-test binary |

**WASM setup (first time only):**
```bash
rustup target add wasm32-wasip1
```

---

## Coding Style & Conventions

- **Formatter**: `rustfmt` with default settings — run `cargo fmt` before every commit.
- **Linter**: Clippy with `-D warnings` — all warnings must be resolved before pushing.
- **Naming**: `snake_case` for functions and variables, `PascalCase` for types and traits.
- **Error handling**: use `thiserror` for library error types, `anyhow` for application-level propagation. Avoid `.unwrap()` outside of tests.
- **Trait implementations**: derive `Default` where possible; never implement it twice for the same type.
- **New modules**: add the module file under `src/`, declare it in `lib.rs` with `pub mod`, and add any public re-exports to the `pub use` block at the bottom of `lib.rs`.
- **Stub methods**: if a method body is not yet implemented, use `todo!()` with a comment rather than leaving a silent no-op. Silent no-ops hide bugs.

---

## State Design Rules

These rules apply to `SheetsState` and any future state structs:

- **Cursor invariants**: `selected_row < row_count()` and `selected_col < col_count()` must hold at all times (or both be 0 when the table is empty). Every method that mutates either field must clamp or guard against out-of-bounds.
- **Scroll follows cursor**: after any cursor movement, call the scroll-adjustment helper so the cursor is always within the visible viewport. Do not let `scroll_row` or `col_offset` drift independently.
- **Pure state, no I/O**: `SheetsState` methods must not perform file I/O or terminal I/O. Loading lives in `data_loader.rs`; rendering lives in `ui.rs`.
- **`SheetsStateSnapshot`**: when adding a new serializable field to `SheetsState`, add the matching field to `SheetsStateSnapshot` and update both `serialize_state` and `deserialize_state`. Runtime-only fields (`Arc<…>`, `SystemTime`, channel handles) stay out of the snapshot.

---

## Testing Guidelines

- Integration tests live in `tests/` as a standalone crate (`zellij-sheets-tests`) with a `path = ".."` dependency on this library.
- Unit tests belong in an inline `#[cfg(test)]` block at the bottom of the relevant `src/*.rs` file.
- Run everything from the repo root with `cargo test --all-features` — this picks up both the inline tests and the `tests/` crate.
- Name test functions as `test_<module>_<scenario>`, e.g. `test_data_loader_empty_csv`, `test_state_scroll_past_end`.
- Every new public function should have at least one test for the expected case and one for a failure or edge case.
- **Navigation tests**: for every new cursor/scroll method, add a test that starts from a known state, applies the method, and asserts the exact resulting `selected_row`, `selected_col`, `scroll_row`, and `col_offset`. Test boundary conditions (first row, last row, first col, last col, empty table).
- **Search tests**: test that `search_next()` wraps around, that `search_prev()` moves backward, and that an empty query is a no-op.
- **Address parser tests** (once `address.rs` exists): cover single cells (`B9`), ranges (`B1:B3`), write syntax (`B7=10`), edge cases (column `AA`, row 1, out-of-bounds).

---

## Commit & Pull Request Guidelines

- **Commit messages**: write a short, descriptive summary of what changed — no format rules, just make it clear enough that someone reading the log understands the change without opening the diff.
- **PRs**: target `main`. The CI pipeline (test, fmt, clippy, wasm build) must pass before merging.
- **PR description**: briefly explain what changed and why. Link related issues if any exist.
- **WIP / experimental work**: label the PR title with `[WIP]` or `[experimental]` so it's clear the branch isn't ready to merge.
- **Feature branches**: name them after the feature area, e.g. `col-cursor`, `vim-nav`, `search`, `cell-address`, `stdin`.

---

## Configuration

Runtime behaviour is controlled by `config/config.toml`, which maps directly to the structs in `src/config.rs`:

| TOML section | Rust struct |
|---|---|
| `[theme]` | `ThemeConfig` |
| `[display]` | `DisplayConfig` |
| `[behavior]` | `BehaviorConfig` |
| `[column]` | `ColumnConfig` |

When adding a new config option: update the struct in `config.rs`, add a matching entry with a comment in `config.toml`, and keep the two in sync. If `config.toml` is absent at runtime, all fields fall back to their `Default` implementations.

---

## Feature Comparison Reference

Summary of what `maaslalani/sheets` (Go) has and the current status in this project:

| Feature | sheets (Go) | zellij-sheets status |
|---|---|---|
| Column cursor | ✅ | 🔲 planned (feature 1) |
| Horizontal scrolling | ✅ | 🔲 planned (features 1 + 4) |
| Vim navigation (`hjkl`, `gg`/`G`, etc.) | ✅ | 🔲 planned (feature 2) |
| Search (`/`, `?`, `n`, `N`) | ✅ | 🔲 planned (feature 3) |
| Cell addressing CLI (`B9`, `B1:B3`) | ✅ | 🔲 planned (feature 5) |
| stdin / pipe support | ✅ | 🔲 planned (feature 6) |
| Cell editing + undo/redo | ✅ | ❌ not planned yet |
| Visual/row selection + yank/cut/paste | ✅ | ❌ not planned yet |
| Command mode (`:w`, `:e`, `:goto`) | ✅ | ❌ not planned yet |
| Formulas | ✅ | ❌ not planned yet |
| Zellij plugin (WASM) | ❌ | ✅ |
| Excel / xlsx support | ❌ | ✅ |
| Unicode-aware layout engine | ❌ | ✅ |
| TOML config file | ❌ | ✅ |
| Serializable state (JSON snapshot) | ❌ | ✅ |
