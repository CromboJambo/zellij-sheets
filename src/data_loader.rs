// Zellij Sheets - Data Loading Module
// Handles loading spreadsheet data from various formats

use calamine::{open_workbook, Reader, Xlsx};
use polars::prelude::*;
use std::path::Path;
use thiserror::Error;

/// Data loading error types
#[derive(Debug, Error)]
pub enum DataLoaderError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Excel file error: {0}")]
    ExcelError(#[from] calamine::Error),

    #[error("CSV parsing error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("Polars error: {0}")]
    PolarsError(#[from] PolarsError),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

/// Result type for data loading operations
pub type Result<T> = std::result::Result<T, DataLoaderError>;

/// Data source type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSource {
    Csv,
    Excel,
    Parquet,
}

/// Load data from a file path
pub fn load_data(path: &Path) -> Result<DataFrame> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| DataLoaderError::InvalidFormat("Unknown file format".to_string()))?;

    match extension.to_lowercase().as_str() {
        "csv" => load_csv(path),
        "xlsx" | "xls" => load_excel(path),
        "parquet" => load_parquet(path),
        _ => Err(DataLoaderError::InvalidFormat(format!(
            "Unsupported file format: {}",
            extension
        ))),
    }
}

/// Load data from CSV file
pub fn load_csv(path: &Path) -> Result<DataFrame> {
    let df = CsvReader::from_path(path)?
        .with_separator(b',')
        .has_header(true)
        .infer_schema(false)
        .finish()?;

    Ok(df)
}

/// Load data from Excel file
pub fn load_excel(path: &Path) -> Result<DataFrame> {
    let mut workbook: Reader<Xlsx<_, _>> = open_workbook(path)?;

    // Get the first sheet name
    let sheet_name = workbook
        .sheet_names()
        .first()
        .ok_or_else(|| DataLoaderError::InvalidFormat("Excel file has no sheets".to_string()))?;

    // Try to get the first sheet
    let range = workbook.worksheet_range(sheet_name)?;
    let df = range.to_dataframe()?;

    Ok(df)
}

/// Load data from Parquet file
pub fn load_parquet(path: &Path) -> Result<DataFrame> {
    let df = ParquetReader::new(path)
        .map_err(DataLoaderError::from)?
        .finish()
        .map_err(DataLoaderError::from)?;

    Ok(df)
}

/// Get data source type from file extension
pub fn get_data_source(path: &Path) -> Result<DataSource> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| DataLoaderError::InvalidFormat("Unknown file format".to_string()))?;

    match extension.to_lowercase().as_str() {
        "csv" => Ok(DataSource::Csv),
        "xlsx" | "xls" => Ok(DataSource::Excel),
        "parquet" => Ok(DataSource::Parquet),
        _ => Err(DataLoaderError::InvalidFormat(format!(
            "Unsupported file format: {}",
            extension
        ))),
    }
}

/// Get file name from path
pub fn get_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Get file extension from path
pub fn get_file_extension(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_string()
}

/// Check if file exists
pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

/// Get file size in bytes
pub fn get_file_size(path: &Path) -> Result<u64> {
    std::fs::metadata(path)
        .map(|meta| meta.len())
        .map_err(DataLoaderError::IoError)
}

/// Get file modification time
pub fn get_file_modification_time(path: &Path) -> Result<std::time::SystemTime> {
    std::fs::metadata(path)
        .map(|meta| meta.modified())
        .map_err(DataLoaderError::IoError)
}
