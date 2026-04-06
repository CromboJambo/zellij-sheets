# Repository Guidelines

## Project Structure

```
zellij-sheets/
├── src/
│   ├── address.rs          # Cell/range/write address parsing helpers for the CLI
│   ├── lib.rs              # Public API surface — re-exports from all modules
│   ├── main.rs             # Native CLI entry point (file, stdin, address, static render)
│   ├── plugin.rs           # Zellij plugin entry point (WASM target)
│   ├── config.rs           # SheetsConfig, ThemeConfig, DisplayConfig, BehaviorConfig
│   ├── data_loader.rs      # CSV / Excel loading plus CSV reader/writer helpers
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

## Product Boundary

`zellij-sheets` owns spreadsheet-viewer mechanics:

- grid rendering
- cursor movement and scrolling
- search and viewport behavior
- cell/range addressing
- lightweight CSV/XLSX inspection and CSV write-back
- native spreadsheet TUI mechanics
- Zellij plugin UX

`zellij-sheets` does **not** own workflow-level pipeline semantics. If a feature is primarily about:

- named transformation steps
- provenance / lineage
- sidecar workflow state
- execution planning
- SQL or M-code generation
- field-oriented orchestration above the grid

it belongs in `nustage`, not here.

Use this rule:

- `zellij-sheets` answers: "How do I inspect and navigate tabular data interactively?"
- `nustage` answers: "How do I define and understand tabular workflows?"

---

## Current Status

The following gaps with `maaslalani/sheets` are now implemented in this repo:

- Column cursor + horizontal scrolling
- Vim-style navigation in the plugin/state layer
- Search in the plugin/state/UI layer
- Horizontal renderer slicing
- Cell/range/write addressing from the native CLI
- stdin / pipe support for CSV input

The next major work is no longer the basic parity layer above. The next major work is making the native binary a real interactive TUI instead of only a static renderer plus address/query commands.

---

## Next Priorities

These are the features that now matter most after closing the first-round gap with
[maaslalani/sheets](https://github.com/maaslalani/sheets). Each item below
corresponds to a concrete area of the codebase.

### 1. Native Interactive TUI (`main.rs`, `state.rs`, `ui.rs`)

The native binary should stop being only a static dump plus addressing helper
and become an actual interactive terminal app. Reuse `SheetsState` and
`UiRenderer`; do not fork behavior between plugin and CLI.

- Add a real event loop for key handling and redraw.
- Reuse vim navigation and search semantics from the plugin/state layer.
- Keep address/query/write behavior available as non-interactive CLI sub-modes.
- Treat the native TUI as the serious spreadsheet frontend; treat the plugin as
  the lighter embedded viewer.

### 2. Native/Plugin Behavior Parity (`main.rs`, `plugin.rs`, `state.rs`)

Behavior should not silently diverge between the native binary and the plugin.
If a key, cursor move, or search behavior exists in one interactive target, it
should usually exist in the other unless the host environment makes that
impossible.

- Keep `SheetsState` as the behavior source of truth.
- Avoid re-implementing navigation rules in per-target code.
- Prefer thin keybinding layers in `main.rs` and `plugin.rs`.

### 3. Editing Model (`main.rs`, `plugin.rs`, `state.rs`)

If editing is added, it must be intentional and safe. Navigation cannot
accidentally mutate data.

- Separate navigation/search modes from edit/write modes.
- Keep CSV write-back explicit.
- If undo/redo appears, design it as a state/history concern first, not a UI hack.

### 4. Renderer Refinement (`ui.rs`, `layout.rs`)

The rendering path now works, but it still needs to feel more like a tool and
less like a debug view.

- Improve native TUI presentation without breaking plugin rendering.
- Preserve Unicode-aware width behavior.
- Keep status/search/cursor visibility obvious.

### 5. CLI Ergonomics (`main.rs`, `address.rs`, `data_loader.rs`)

The CLI is already useful. Keep it sharp rather than letting it accumulate
random flags.

- Keep `file [address]` and stdin semantics stable.
- Prefer clear errors over clever parsing.
- Treat help text and examples as part of the product surface.

---

## Build & Development Commands

| Command | What it does |
|---|---|
| `cargo check` | Verify the project compiles without producing artifacts |
| `cargo build` | Build the native CLI binary (debug) |
| `cargo build --release --target wasm32-wasip1` | Build the Zellij WASM plugin |
| `cargo run -- --help` | Show native CLI usage and examples |
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
- **Behavior reuse**: when adding native TUI features, push reusable logic into `state.rs`, `ui.rs`, `layout.rs`, or `lib.rs` rather than duplicating it in `main.rs` and `plugin.rs`.

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
- **Address parser tests**: cover single cells (`B9`), ranges (`B1:B3`), write syntax (`B7=10`), edge cases (column `AA`, row 1, out-of-bounds).
- **CLI tests**: when changing `main.rs`, verify both direct file usage and stdin usage. Prefer command-style integration checks for user-facing behavior.

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
| Column cursor | ✅ | ✅ |
| Horizontal scrolling | ✅ | ✅ |
| Vim navigation (`hjkl`, `gg`/`G`, etc.) | ✅ | ✅ |
| Search (`/`, `?`, `n`, `N`) | ✅ | ✅ |
| Cell addressing CLI (`B9`, `B1:B3`) | ✅ | ✅ |
| stdin / pipe support | ✅ | ✅ |
| Cell editing + undo/redo | ✅ | ❌ not planned yet |
| Visual/row selection + yank/cut/paste | ✅ | ❌ not planned yet |
| Command mode (`:w`, `:e`, `:goto`) | ✅ | ❌ not planned yet |
| Formulas | ✅ | ❌ not planned yet |
| Zellij plugin (WASM) | ❌ | ✅ |
| Excel / xlsx support | ❌ | ✅ |
| Unicode-aware layout engine | ❌ | ✅ |
| TOML config file | ❌ | ✅ |
| Serializable state (JSON snapshot) | ❌ | ✅ |
