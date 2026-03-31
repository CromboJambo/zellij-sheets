use std::path::PathBuf;
use std::sync::Arc;
use zellij_sheets::config::SheetsConfig;
use zellij_sheets::state::{
    deserialize_state, serialize_state, DataType, SheetsState, SortDirection, StatusLevel,
    StatusMessage, ViewMode,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let config = Arc::new(SheetsConfig::default());
        let state = SheetsState::new(config);

        assert_eq!(state.row_count(), 0);
        assert_eq!(state.col_count(), 0);
        assert!(!state.file_name().is_empty());
    }

    #[test]
    fn test_view_mode_default() {
        let config = Arc::new(SheetsConfig::default());
        let state = SheetsState::new(config);

        assert_eq!(state.get_view_mode().unwrap(), ViewMode::Grid);
    }

    #[test]
    fn test_set_view_mode() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.set_view_mode(ViewMode::List);
        assert_eq!(state.get_view_mode().unwrap(), ViewMode::List);
    }

    #[test]
    fn test_sort_direction() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.set_sort(Some("column1".to_string()), SortDirection::Ascending);
        assert_eq!(
            state.get_sort_column().unwrap(),
            Some("column1".to_string())
        );
        assert_eq!(
            state.get_sort_direction().unwrap(),
            SortDirection::Ascending
        );
    }

    #[test]
    fn test_status_messages() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        let message = StatusMessage {
            message: "Test message".to_string(),
            timestamp: std::time::SystemTime::now(),
            level: StatusLevel::Info,
            duration_secs: 5,
        };

        state.add_status_message(message);
        let messages = state.get_status_messages().unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_clear_status_messages() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.add_status_message(StatusMessage {
            message: "Test".to_string(),
            timestamp: std::time::SystemTime::now(),
            level: StatusLevel::Info,
            duration_secs: 5,
        });

        state.clear_status_messages();
        assert_eq!(state.get_status_messages().unwrap().len(), 0);
    }

    #[test]
    fn test_data_type_inference() {
        assert_eq!(infer_data_type("123"), DataType::Number);
        assert_eq!(infer_data_type("3.14"), DataType::Number);
        assert_eq!(infer_data_type("true"), DataType::Boolean);
        assert_eq!(infer_data_type("false"), DataType::Boolean);
        assert_eq!(infer_data_type("hello"), DataType::String);
        assert_eq!(infer_data_type(""), DataType::Empty);
    }

    #[test]
    fn test_navigation() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        // Empty state: navigation should be a no-op
        state.select_down();
        assert_eq!(state.selected_row(), 0);

        state.select_up();
        assert_eq!(state.selected_row(), 0);
    }

    #[test]
    fn test_scroll() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        // Empty state: scroll should be a no-op
        state.scroll_down();
        assert_eq!(state.scroll_row(), 0);

        state.scroll_up();
        assert_eq!(state.scroll_row(), 0);
    }

    #[test]
    fn test_page_navigation() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        // Empty state: page navigation should be a no-op
        state.page_down();
        assert_eq!(state.selected_row(), 0);

        state.page_up();
        assert_eq!(state.selected_row(), 0);
    }

    #[test]
    fn test_go_to_top_bottom() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        // Empty state: both top and bottom are row 0
        state.go_to_bottom();
        assert_eq!(state.selected_row(), 0);
        assert_eq!(state.scroll_row(), 0);

        state.go_to_top();
        assert_eq!(state.selected_row(), 0);
        assert_eq!(state.scroll_row(), 0);
    }

    #[test]
    fn test_config_access() {
        let config = Arc::new(SheetsConfig::default());
        let state = SheetsState::new(config);

        let retrieved_config = state.get_config().unwrap();
        assert_eq!(retrieved_config.display.preview_rows, 20);
    }

    #[test]
    fn test_file_info() {
        let config = Arc::new(SheetsConfig::default());
        let state = SheetsState::new(config);

        // No file loaded: should return the sentinel string
        assert_eq!(state.file_name(), "No file loaded");
        assert_eq!(state.get_file_name().unwrap(), "No file loaded");
    }

    #[test]
    fn test_width_height() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.resize(100, 50);
        assert_eq!(state.get_width().unwrap(), 100);
        assert_eq!(state.get_height().unwrap(), 50);
    }

    #[test]
    fn test_visible_rows() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.resize(100, 50);
        // visible_rows = height.saturating_sub(5).max(1) = 45
        assert_eq!(state.visible_rows(), 45);
    }

    #[test]
    fn test_row_range() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.resize(100, 50);
        // Empty state: end is clamped to row_count() = 0
        let (start, end) = state.row_range();
        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn test_at_top_bottom() {
        let config = Arc::new(SheetsConfig::default());
        let state = SheetsState::new(config);

        // Empty state is always at_top and at_bottom simultaneously
        assert!(state.at_top());
        assert!(state.at_bottom());
    }

    #[test]
    fn test_show_options() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.set_show_row_numbers(true);
        assert!(state.get_show_row_numbers().unwrap());

        state.set_show_column_numbers(false);
        assert!(!state.get_show_column_numbers().unwrap());

        state.set_show_grid_lines(false);
        assert!(!state.get_show_grid_lines().unwrap());
    }

    #[test]
    fn test_set_config() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        let new_config = SheetsConfig::default();
        state.set_config(new_config);

        assert_eq!(state.get_config().unwrap().display.preview_rows, 20);
    }

    #[test]
    fn test_file_path() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.set_file_path(PathBuf::from("/test/file.csv"));
        assert_eq!(
            state.get_file_path().unwrap(),
            Some(PathBuf::from("/test/file.csv"))
        );
    }

    #[test]
    fn test_search_query() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.set_search_query(Some("test".to_string()));
        assert_eq!(state.get_search_query().unwrap(), Some("test".to_string()));

        state.set_search_query(None);
        assert_eq!(state.get_search_query().unwrap(), None);
    }

    #[test]
    fn test_filter_expression() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.set_filter_expr(Some("column1 > 10".to_string()));
        assert_eq!(
            state.get_filter_expr().unwrap(),
            Some("column1 > 10".to_string())
        );

        state.set_filter_expr(None);
        assert_eq!(state.get_filter_expr().unwrap(), None);
    }

    #[test]
    fn test_error_handling() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.set_last_error(Some("Test error".to_string()));
        assert_eq!(
            state.get_last_error().unwrap(),
            Some("Test error".to_string())
        );

        state.clear_last_error();
        assert_eq!(state.get_last_error().unwrap(), None);
    }

    #[test]
    fn test_serialize_deserialize_state() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.set_view_mode(ViewMode::List);
        state.set_search_query(Some("test".to_string()));

        let serialized = serialize_state(&state).unwrap();
        assert!(!serialized.is_empty());

        let deserialized = deserialize_state(&serialized).unwrap();
        assert_eq!(deserialized.get_view_mode().unwrap(), ViewMode::List);
        assert_eq!(
            deserialized.get_search_query().unwrap(),
            Some("test".to_string())
        );
    }

    #[test]
    fn test_serialize_deserialize_with_data() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.set_view_mode(ViewMode::Grid);
        state.set_search_query(Some("search".to_string()));

        let serialized = serialize_state(&state).unwrap();
        assert!(!serialized.is_empty());

        let deserialized = deserialize_state(&serialized).unwrap();
        assert_eq!(deserialized.get_view_mode().unwrap(), ViewMode::Grid);
        assert_eq!(
            deserialized.get_search_query().unwrap(),
            Some("search".to_string())
        );
    }

    #[test]
    fn test_serialize_deserialize_error() {
        let result = deserialize_state("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_invalid_state() {
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);

        state.set_view_mode(ViewMode::List);

        let result = serialize_state(&state);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_default_state() {
        let state = SheetsState::default();

        assert_eq!(state.row_count(), 0);
        assert_eq!(state.col_count(), 0);
        assert_eq!(state.get_view_mode().unwrap(), ViewMode::Grid);
        assert!(!state.get_show_row_numbers().unwrap());
        assert!(state.get_show_column_numbers().unwrap());
        assert!(state.get_show_grid_lines().unwrap());
        assert!(!state.get_show_data_types().unwrap());
    }

    fn infer_data_type(value: &str) -> DataType {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return DataType::Empty;
        }
        if trimmed.eq_ignore_ascii_case("true") || trimmed.eq_ignore_ascii_case("false") {
            return DataType::Boolean;
        }
        if trimmed.parse::<i64>().is_ok() || trimmed.parse::<f64>().is_ok() {
            return DataType::Number;
        }
        DataType::String
    }
}
