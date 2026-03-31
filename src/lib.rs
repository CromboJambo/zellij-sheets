//! # Zellij Sheets
//!
//! A terminal-based spreadsheet viewer powered by Zellij.
//! This library provides spreadsheet viewing functionality for both Zellij plugins
//! and native CLI applications.

pub mod config;
pub mod data_loader;
pub mod state;
pub mod ui;

/// Configuration and display settings for the spreadsheet viewer
pub use config::{BehaviorConfig, ColumnConfig, DisplayConfig, SheetsConfig, ThemeConfig};

/// Utility functions for file operations and data loading
pub use data_loader::{file_exists, get_file_extension, get_file_name, get_file_size, load_data};

/// Core spreadsheet state and data types
pub use state::{DataType, SheetsState};
