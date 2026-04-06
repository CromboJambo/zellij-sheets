use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use zellij_sheets::config::{validate_config, ConfigError, SheetsConfig};
use zellij_sheets::data_loader::{
    get_data_source, get_file_extension, get_file_name, load_csv_from_reader, load_data, write_csv,
    DataSource, LoadedData,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_name() {
        let path = Path::new("/home/user/test.csv");
        let result = get_file_name(path);
        assert_eq!(result, "test.csv");
    }

    #[test]
    fn test_get_file_extension() {
        let path = Path::new("/home/user/test.csv");
        let result = get_file_extension(path);
        assert_eq!(result, "csv");
    }

    #[test]
    fn test_get_file_extension_with_uppercase() {
        // get_file_extension returns the raw extension; get_data_source does the lowercasing
        let path = Path::new("/home/user/test.XLSX");
        let result = get_file_extension(path);
        assert_eq!(result.to_ascii_lowercase(), "xlsx");
    }

    #[test]
    fn test_get_file_extension_with_no_extension() {
        let path = Path::new("/home/user/test");
        let result = get_file_extension(path);
        assert_eq!(result, "");
    }

    #[test]
    fn test_data_source_csv() {
        let path = Path::new("/home/user/test.csv");
        let result = get_data_source(path);
        assert!(matches!(result, Ok(DataSource::Csv)));
    }

    #[test]
    fn test_data_source_excel() {
        let path = Path::new("/home/user/test.xlsx");
        let result = get_data_source(path);
        assert!(matches!(result, Ok(DataSource::Excel)));
    }

    #[test]
    fn test_data_source_parquet() {
        let path = Path::new("/home/user/test.parquet");
        let result = get_data_source(path);
        assert!(matches!(result, Ok(DataSource::Parquet)));
    }

    #[test]
    fn test_data_source_unsupported() {
        // Unsupported extensions are an error (InvalidFormat), not a special variant
        let path = Path::new("/home/user/test.txt");
        let result = get_data_source(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_data_source_unknown_extension() {
        let path = Path::new("/home/user/test.unknown");
        let result = get_data_source(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_data_error() {
        let path = PathBuf::from("/nonexistent/file.csv");
        let result = load_data(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_csv_from_reader() {
        let result = load_csv_from_reader(Cursor::new("id,name\n1,Alice\n2,Bob\n")).unwrap();
        assert_eq!(result.source, DataSource::Csv);
        assert_eq!(result.headers, vec!["id", "name"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[1][1], "Bob");
    }

    #[test]
    fn test_load_csv_from_reader_empty_input() {
        let result = load_csv_from_reader(Cursor::new("")).unwrap();
        assert!(result.headers.is_empty());
        assert!(result.rows.is_empty());
    }

    #[test]
    fn test_write_csv_round_trip() {
        let path =
            std::env::temp_dir().join(format!("zellij-sheets-test-{}.csv", std::process::id()));
        let data = LoadedData {
            headers: vec!["id".into(), "name".into()],
            rows: vec![vec!["1".into(), "Alice".into()]],
            source: DataSource::Csv,
        };

        write_csv(&path, &data).unwrap();
        let loaded = load_data(&path).unwrap();
        assert_eq!(loaded.headers, data.headers);
        assert_eq!(loaded.rows, data.rows);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_write_csv_invalid_path() {
        let path = PathBuf::from("/definitely/missing/dir/test.csv");
        let data = LoadedData {
            headers: vec!["id".into()],
            rows: vec![vec!["1".into()]],
            source: DataSource::Csv,
        };

        assert!(write_csv(&path, &data).is_err());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = SheetsConfig::default();
        let result = validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validation_invalid_preview_rows() {
        let mut config = SheetsConfig::default();
        config.display.preview_rows = 0;
        let result = validate_config(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidConfig(msg) => {
                assert!(msg.contains("preview_rows must be greater than 0"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_config_validation_invalid_max_cell_length() {
        let mut config = SheetsConfig::default();
        config.display.max_cell_length = 0;
        let result = validate_config(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidConfig(msg) => {
                assert!(msg.contains("max_cell_length must be greater than 0"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_config_validation_invalid_auto_refresh_interval() {
        let mut config = SheetsConfig::default();
        config.behavior.auto_refresh_interval = 0;
        let result = validate_config(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidConfig(msg) => {
                assert!(msg.contains("auto_refresh_interval must be greater than 0"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_config_validation_invalid_page_size() {
        let mut config = SheetsConfig::default();
        config.behavior.page_size = 0;
        let result = validate_config(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidConfig(msg) => {
                assert!(msg.contains("page_size must be greater than 0"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_config_validation_invalid_min_column_width() {
        let mut config = SheetsConfig::default();
        config.columns.min_column_width = 0;
        let result = validate_config(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidConfig(msg) => {
                assert!(msg.contains("min_column_width must be greater than 0"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_config_validation_invalid_max_column_width() {
        let mut config = SheetsConfig::default();
        config.columns.max_column_width = 0;
        let result = validate_config(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidConfig(msg) => {
                assert!(msg.contains("max_column_width must be greater than 0"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_config_validation_min_exceeds_max_column_width() {
        let mut config = SheetsConfig::default();
        config.columns.min_column_width = 20;
        config.columns.max_column_width = 10;
        let result = validate_config(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidConfig(msg) => {
                assert!(msg.contains("min_column_width must not exceed max_column_width"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_config_validation_invalid_fixed_widths() {
        let mut config = SheetsConfig::default();
        config.columns.fixed_widths = vec![0, 10, 20];
        let result = validate_config(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidConfig(msg) => {
                assert!(msg.contains("fixed_widths must be greater than 0"));
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
