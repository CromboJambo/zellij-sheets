// Zellij Sheets - Configuration Module
// Handles plugin configuration and settings

use serde::{Deserialize, Serialize};
use std::default::Default;

/// Main configuration structure for the spreadsheet viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetsConfig {
    /// Theme configuration
    pub theme: ThemeConfig,

    /// Display settings
    pub display: DisplayConfig,

    /// Behavior settings
    pub behavior: BehaviorConfig,

    /// Column configuration
    pub columns: ColumnConfig,
}

/// Theme configuration for the spreadsheet viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Background color
    pub background: String,

    /// Text color
    pub text: String,

    /// Header background color
    pub header_background: String,

    /// Header text color
    pub header_text: String,

    /// Selected row background color
    pub selected_background: String,

    /// Selected row text color
    pub selected_text: String,

    /// Column header background color
    pub column_header_background: String,

    /// Column header text color
    pub column_header_text: String,

    /// Accent colors for different data types
    pub accent_colors: AccentColors,
}

/// Accent colors for different data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccentColors {
    /// Number color
    pub number: String,

    /// String color
    pub string: String,

    /// Boolean color
    pub boolean: String,

    /// Date color
    pub date: String,
}

/// Display configuration for the spreadsheet viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Number of preview rows
    pub preview_rows: usize,

    /// Show column numbers
    pub show_column_numbers: bool,

    /// Show row numbers
    pub show_row_numbers: bool,

    /// Truncate long cell values
    pub truncate_long_values: bool,

    /// Maximum cell value length before truncation
    pub max_cell_length: usize,

    /// Show data type indicators
    pub show_data_types: bool,
}

/// Behavior configuration for the spreadsheet viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Auto-refresh when file changes
    pub auto_refresh: bool,

    /// Auto-refresh interval in seconds
    pub auto_refresh_interval: u64,

    /// Default scroll speed
    pub scroll_speed: ScrollSpeed,

    /// Page size for page navigation
    pub page_size: usize,
}

/// Scroll speed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollSpeed {
    /// Normal scroll speed
    pub normal: f32,

    /// Fast scroll speed
    pub fast: f32,
}

/// Column configuration for the spreadsheet viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnConfig {
    /// Auto-width columns based on content
    pub auto_width: bool,

    /// Fixed column widths
    pub fixed_widths: Vec<usize>,

    /// Minimum column width
    pub min_column_width: usize,

    /// Maximum column width
    pub max_column_width: usize,

    /// Column width mode
    pub width_mode: ColumnWidthMode,
}

/// Column width mode for the spreadsheet viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnWidthMode {
    /// Auto-width based on content
    Auto,
    /// Fixed width for all columns
    Fixed,
    /// Mixed mode with some auto and some fixed
    Mixed,
}

/// Default configuration
impl Default for SheetsConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            display: DisplayConfig::default(),
            behavior: BehaviorConfig::default(),
            columns: ColumnConfig::default(),
        }
    }
}

/// Default configuration
impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: "#000000".to_string(),
            text: "#FFFFFF".to_string(),
            header_background: "#0055AA".to_string(),
            header_text: "#FFFFFF".to_string(),
            selected_background: "#00AAFF".to_string(),
            selected_text: "#000000".to_string(),
            column_header_background: "#004488".to_string(),
            column_header_text: "#00FFFF".to_string(),
            accent_colors: AccentColors::default(),
        }
    }
}

/// Default configuration
impl Default for AccentColors {
    fn default() -> Self {
        Self {
            number: "#00FF00".to_string(),
            string: "#FFFF00".to_string(),
            boolean: "#FF00FF".to_string(),
            date: "#FF8800".to_string(),
        }
    }
}

/// Default configuration
impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            preview_rows: 20,
            show_column_numbers: true,
            show_row_numbers: false,
            truncate_long_values: true,
            max_cell_length: 50,
            show_data_types: false,
        }
    }
}

/// Default configuration
impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            auto_refresh: false,
            auto_refresh_interval: 5,
            scroll_speed: ScrollSpeed::default(),
            page_size: 10,
        }
    }
}

/// Default configuration
impl Default for ScrollSpeed {
    fn default() -> Self {
        Self {
            normal: 1.0,
            fast: 3.0,
        }
    }
}

/// Default configuration
impl Default for ColumnConfig {
    fn default() -> Self {
        Self {
            auto_width: true,
            fixed_widths: Vec::new(),
            min_column_width: 8,
            max_column_width: 40,
            width_mode: ColumnWidthMode::Auto,
        }
    }
}

/// Load configuration from TOML file
pub fn load_config(path: &str) -> Result<SheetsConfig, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: SheetsConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Save configuration to TOML file
pub fn save_config(config: &SheetsConfig, path: &str) -> Result<(), ConfigError> {
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Get default configuration path
pub fn default_config_path() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let config_dir = format!("{}/.config/zellij-sheets", home);
    std::fs::create_dir_all(&config_dir).ok()?;
    Some(format!("{}/config.toml", config_dir))
}

/// Validate configuration
pub fn validate_config(config: &SheetsConfig) -> Result<(), ConfigError> {
    // Validate theme colors
    validate_color(&config.theme.background)?;
    validate_color(&config.theme.text)?;
    validate_color(&config.theme.header_background)?;
    validate_color(&config.theme.header_text)?;
    validate_color(&config.theme.selected_background)?;
    validate_color(&config.theme.selected_text)?;
    validate_color(&config.theme.column_header_background)?;
    validate_color(&config.theme.column_header_text)?;

    // Validate accent colors
    validate_color(&config.theme.accent_colors.number)?;
    validate_color(&config.theme.accent_colors.string)?;
    validate_color(&config.theme.accent_colors.boolean)?;
    validate_color(&config.theme.accent_colors.date)?;

    // Validate display settings
    if config.display.preview_rows == 0 {
        return Err(ConfigError::InvalidConfig(
            "preview_rows must be greater than 0".to_string(),
        ));
    }

    if config.display.max_cell_length == 0 {
        return Err(ConfigError::InvalidConfig(
            "max_cell_length must be greater than 0".to_string(),
        ));
    }

    // Validate behavior settings
    if config.behavior.auto_refresh_interval == 0 {
        return Err(ConfigError::InvalidConfig(
            "auto_refresh_interval must be greater than 0".to_string(),
        ));
    }

    if config.behavior.page_size == 0 {
        return Err(ConfigError::InvalidConfig(
            "page_size must be greater than 0".to_string(),
        ));
    }

    // Validate column configuration
    if config.columns.min_column_width == 0 {
        return Err(ConfigError::InvalidConfig(
            "min_column_width must be greater than 0".to_string(),
        ));
    }

    if config.columns.max_column_width == 0 {
        return Err(ConfigError::InvalidConfig(
            "max_column_width must be greater than 0".to_string(),
        ));
    }

    if config.columns.min_column_width > config.columns.max_column_width {
        return Err(ConfigError::InvalidConfig(
            "min_column_width must not exceed max_column_width".to_string(),
        ));
    }

    // Validate fixed widths if provided
    for &width in &config.columns.fixed_widths {
        if width == 0 {
            return Err(ConfigError::InvalidConfig(
                "fixed_widths must be greater than 0".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validate color format (hex or named)
fn validate_color(color: &str) -> Result<(), ConfigError> {
    if color.starts_with('#') {
        // Hex color
        let hex = color.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 3 {
            return Err(ConfigError::InvalidConfig(format!(
                "Invalid hex color format: {}",
                color
            )));
        }
    }
    // Allow named colors (we'll let the terminal handle them)
    Ok(())
}
