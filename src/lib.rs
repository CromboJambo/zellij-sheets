//! # Zellij Sheets
//!
//! Terminal-native spreadsheet viewing for Zellij and the command line.
//!
//! `zellij-sheets` is the grid and navigation layer for tabular data. It loads
//! CSV and Excel files, renders them with a Unicode-aware layout engine, and
//! exposes shared state and rendering logic used by both the Zellij plugin and
//! the native CLI.
//!
//! This crate is intentionally viewer-first. Workflow-level pipeline semantics,
//! provenance, and transformation orchestration belong in `nustage`, not here.
//!
//! ## Features
//!
//! - CSV and Excel (`.xlsx`, `.xls`) loading
//! - Horizontal scrolling with a real column cursor
//! - Vim-style navigation primitives
//! - Search state and matching helpers
//! - Cell/range/write address parsing for the native CLI
//! - CSV loading from stdin and CSV write-back helpers
//! - Serializable spreadsheet state snapshots
//! - Unicode-aware column measurement and layout
//!
//! ## Public Surface
//!
//! - [`state`] contains [`SheetsState`], cursor/search behavior, and snapshot support.
//! - [`layout`] contains the measurement and width-resolution engine.
//! - [`ui`] contains terminal rendering helpers.
//! - [`data_loader`] contains CSV/Excel loading helpers.
//! - [`address`] contains cell/range/write address parsing for the native CLI.
//!
//! ## Usage
//!
//! Read a file into shared state:
//!
//! ```no_run
//! use std::path::PathBuf;
//! use std::sync::Arc;
//! use zellij_sheets::{SheetsConfig, SheetsState};
//!
//! let mut state = SheetsState::new(Arc::new(SheetsConfig::default()));
//! state.load_file(PathBuf::from("data.csv"))?;
//! # Ok::<(), zellij_sheets::state::StateError>(())
//! ```
//!
//! Parse a CLI-style cell address:
//!
//! ```
//! use zellij_sheets::{parse_address_command, AddressCommand, CellAddress};
//!
//! let command = parse_address_command("B9")?;
//! assert_eq!(command, AddressCommand::Cell(CellAddress { row: 8, col: 1 }));
//! # Ok::<(), zellij_sheets::address::AddressError>(())
//! ```
//!
//! Example Zellij plugin config:
//! ```kdl
//! plugins {
//!     zellij-sheets location="file:/path/to/zellij-sheets.wasm"
//! }
//! ```
//!
//! Example native CLI usage:
//! ```bash
//! zellij-sheets data.csv
//! zellij-sheets data.csv B9
//! zellij-sheets data.csv B1:B3
//! cat data.csv | zellij-sheets B2
//! ```

pub mod address;
pub mod config;
pub mod data_loader;
pub mod layout;
pub mod state;
pub mod ui;

/// CLI address parsing helpers
pub use address::{
    col_letter_to_index, index_to_col_letters, parse_address_command, AddressCommand, CellAddress,
};

/// Configuration and display settings for the spreadsheet viewer
pub use config::{
    BehaviorConfig, ColumnConfig, DisplayConfig, ScrollSpeed, SheetsConfig, ThemeConfig,
};

/// Utility functions for file operations and data loading
pub use data_loader::{
    file_exists, get_file_extension, get_file_name, get_file_size, load_csv_from_reader, load_data,
    write_csv,
};

/// Core spreadsheet state and data types
pub use state::{cell_matches_query, DataType, SearchDirection, SheetsState};

/// Layout and rendering engine
pub use layout::{fit_cell, ColumnLayout, LayoutCache, LayoutEngine};

/// UI rendering types
pub use ui::{Colors, UiError, UiRenderer};
