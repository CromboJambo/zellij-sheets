
# Repository Guidelines

## Project Structure

```
zellij-sheets/
├── src/
│   ├── lib.rs              # Public API surface — re-exports from all modules
│   ├── main.rs             # Native CLI entry point
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

---

## Testing Guidelines

- Integration tests live in `tests/` as a standalone crate (`zellij-sheets-tests`) with a `path = ".."` dependency on this library.
- Unit tests belong in an inline `#[cfg(test)]` block at the bottom of the relevant `src/*.rs` file.
- Run everything from the repo root with `cargo test --all-features` — this picks up both the inline tests and the `tests/` crate.
- Name test functions as `test_<module>_<scenario>`, e.g. `test_data_loader_empty_csv`, `test_state_scroll_past_end`.
- Every new public function should have at least one test for the expected case and one for a failure or edge case.

---

## Commit & Pull Request Guidelines

- **Commit messages**: write a short, descriptive summary of what changed — no format rules, just make it clear enough that someone reading the log understands the change without opening the diff.
- **PRs**: target `main`. The CI pipeline (test, fmt, clippy, wasm build) must pass before merging.
- **PR description**: briefly explain what changed and why. Link related issues if any exist.
- **WIP / experimental work**: label the PR title with `[WIP]` or `[experimental]` so it's clear the branch isn't ready to merge.

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