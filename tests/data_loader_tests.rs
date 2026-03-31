use std::path::{Path, PathBuf};
use zellij_sheets::data_loader::{
    get_data_source, get_file_extension, get_file_name, load_data, DataSource,
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
}
