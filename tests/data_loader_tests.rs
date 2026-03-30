use std::path::Path;
use zellij_sheets::data_loader::{load_data, DataLoaderError, DataSource, file_exists, get_file_extension, get_file_name, get_file_size, get_file_modification_time};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_exists() {
        let test_file = Path::new("/tmp/test.txt");
        let result = file_exists(test_file);
        // This will return false since the file doesn't exist, but the function should work
        assert!(result);
    }

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
        let path = Path::new("/home/user/test.XLSX");
        let result = get_file_extension(path);
        assert_eq!(result, "xlsx");
    }

    #[test]
    fn test_get_file_extension_with_no_extension() {
        let path = Path::new("/home/user/test");
        let result = get_file_extension(path);
        assert_eq!(result, "");
    }

    #[test]
    fn test_get_file_size() {
        let path = Path::new("/tmp/test.txt");
        let result = get_file_size(path);
        // This will return an error since the file doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_get_file_modification_time() {
        let path = Path::new("/tmp/test.txt");
        let result = get_file_modification_time(path);
        // This will return an error since the file doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_data_source_csv() {
        let path = Path::new("/home/user/test.csv");
        let result = DataSource::from_path(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataSource::Csv);
    }

    #[test]
    fn test_data_source_excel() {
        let path = Path::new("/home/user/test.xlsx");
        let result = DataSource::from_path(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataSource::Excel);
    }

    #[test]
    fn test_data_source_parquet() {
        let path = Path::new("/home/user/test.parquet");
        let result = DataSource::from_path(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataSource::Parquet);
    }

    #[test]
    fn test_data_source_unsupported() {
        let path = Path::new("/home/user/test.txt");
        let result = DataSource::from_path(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_data_source_unknown_extension() {
        let path = Path::new("/home/user/test.unknown");
        let result = DataSource::from_path(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_data_error() {
        let path = Path::new("/nonexistent/file.csv");
        let result = load_data(path);
        assert!(result.is_err());
    }
}

fn main() {
    println!("Run tests with cargo test instead");
}
