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

pub mod config;
pub mod data_loader;
pub mod layout;
pub mod state;
pub mod ui;

/// Configuration and display settings for the spreadsheet viewer
pub use config::{BehaviorConfig, ColumnConfig, DisplayConfig, SheetsConfig, ThemeConfig};

/// Utility functions for file operations and data loading
pub use data_loader::{file_exists, get_file_extension, get_file_name, get_file_size, load_data};

/// Core spreadsheet state and data types
pub use state::{DataType, SheetsState};

/// Layout and rendering engine
pub use layout::{fit_cell, ColumnLayout, LayoutCache, LayoutEngine};

/// UI rendering types
pub use ui::{
    BehaviorConfig, Colors, DisplayConfig, ScrollSpeed, ThemeConfig, UiError, UiRenderer,
};
