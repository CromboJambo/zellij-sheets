// Zellij Sheets - Library Module
// Exports all modules for the spreadsheet viewer

pub mod config;
pub mod data_loader;
pub mod state;
pub mod ui;

// Re-export commonly used types
pub use config::{BehaviorConfig, ColumnConfig, DisplayConfig, SheetsConfig, ThemeConfig};
pub use data_loader::{file_exists, get_file_extension, get_file_name, get_file_size, load_data};
pub use state::{DataType, SheetsState};
