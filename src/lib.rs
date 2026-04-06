//! # Zellij Sheets
//!
//! A terminal-based spreadsheet viewer powered by Zellij.
//! This library provides spreadsheet viewing functionality for both Zellij plugins
//! and native CLI applications.
//!
//! ## Features
//!
//! - View CSV and Excel files in the terminal
//! - Configurable themes and display settings
//! - Search and filter support (planned)
//! - Sort functionality (planned)
//! - Parquet preview (planned)
//!
//! ## Usage
//!
//! For Zellij plugin:
//! ```kdl
//! plugins {
//!     zellij-sheets location="file:/path/to/zellij-sheets.wasm"
//! }
//! ```
//!
//! For native CLI:
//! ```bash
//! zellij-sheets --input /path/to/file.csv
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
