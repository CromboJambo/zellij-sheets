//! Data loading module for spreadsheet files
//!
//! Provides functionality to load and parse spreadsheet data from various formats
//! including CSV and Excel files.

use calamine::{open_workbook_auto, Data, Reader};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during data loading
#[derive(Debug, Error)]
pub enum DataLoaderError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Excel file error: {0}")]
    ExcelError(#[from] calamine::Error),

    #[error("CSV parsing error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
}

/// Result type for data loading operations
pub type Result<T> = std::result::Result<T, DataLoaderError>;

/// Data source type for spreadsheet files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSource {
    /// CSV file format
    Csv,
    /// Excel file format (.xlsx, .xls)
    Excel,
    /// Parquet file format (not yet supported)
    Parquet,
}

/// Loaded spreadsheet data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedData {
    /// Column headers
    pub headers: Vec<String>,
    /// Data rows
    pub rows: Vec<Vec<String>>,
    /// Source file format
    pub source: DataSource,
}

/// Load spreadsheet data from a file path
///
/// # Arguments
///
/// * `path` - Path to the spreadsheet file
///
/// # Returns
///
/// Returns `LoadedData` on success or `DataLoaderError` on failure
pub fn load_data(path: &Path) -> Result<LoadedData> {
    let source = get_data_source(path)?;
    match source {
        DataSource::Csv => load_csv(path),
        DataSource::Excel => load_excel(path),
        DataSource::Parquet => Err(DataLoaderError::InvalidFormat(
            "Parquet preview is not supported in the rebuilt plugin yet".to_string(),
        )),
    }
}

/// Load data from a CSV file
///
/// # Arguments
///
/// * `path` - Path to the CSV file
///
/// # Returns
///
/// Returns `LoadedData` on success or `DataLoaderError` on failure
pub fn load_csv(path: &Path) -> Result<LoadedData> {
    let mut reader = csv::Reader::from_path(path)?;
    let headers = reader
        .headers()?
        .iter()
        .enumerate()
        .map(|(index, value)| normalize_header(value, index))
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record?;
        let mut row = record.iter().map(ToOwned::to_owned).collect::<Vec<_>>();
        row.resize(headers.len(), String::new());
        rows.push(row);
    }

    Ok(LoadedData {
        headers,
        rows,
        source: DataSource::Csv,
    })
}

pub fn load_excel(path: &Path) -> Result<LoadedData> {
    let mut workbook = open_workbook_auto(path)?;
    let sheet_names = workbook.sheet_names().to_owned();
    let sheet_name = sheet_names
        .first()
        .ok_or_else(|| DataLoaderError::InvalidFormat("Excel file has no sheets".to_string()))?;
    let range = workbook.worksheet_range(sheet_name)?;
    let mut rows_iter = range.rows();
    let header_row = rows_iter
        .next()
        .ok_or_else(|| DataLoaderError::InvalidFormat("Excel sheet is empty".to_string()))?;

    let headers = header_row
        .iter()
        .enumerate()
        .map(|(index, cell)| normalize_header(&excel_cell_to_string(cell), index))
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for row in rows_iter {
        let mut rendered = row.iter().map(excel_cell_to_string).collect::<Vec<_>>();
        rendered.resize(headers.len(), String::new());
        rows.push(rendered);
    }

    Ok(LoadedData {
        headers,
        rows,
        source: DataSource::Excel,
    })
}

pub fn get_data_source(path: &Path) -> Result<DataSource> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| DataLoaderError::InvalidFormat("Unknown file format".to_string()))?;

    match extension.to_ascii_lowercase().as_str() {
        "csv" => Ok(DataSource::Csv),
        "xlsx" | "xls" => Ok(DataSource::Excel),
        "parquet" => Ok(DataSource::Parquet),
        _ => Err(DataLoaderError::InvalidFormat(format!(
            "Unsupported file format: {extension}"
        ))),
    }
}

pub fn get_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string()
}

pub fn get_file_extension(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_string()
}

pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

pub fn get_file_size(path: &Path) -> Result<u64> {
    std::fs::metadata(path)
        .map(|meta| meta.len())
        .map_err(DataLoaderError::IoError)
}

pub fn get_file_modification_time(path: &Path) -> Result<std::time::SystemTime> {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .map_err(DataLoaderError::IoError)
}

fn normalize_header(value: &str, index: usize) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        format!("column_{}", index + 1)
    } else {
        trimmed.to_string()
    }
}

fn excel_cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) | Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
        Data::Int(value) => value.to_string(),
        Data::Float(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::Error(value) => format!("{value:?}"),
    }
}
