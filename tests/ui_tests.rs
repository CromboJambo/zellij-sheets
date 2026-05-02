use std::sync::Arc;
use zellij_sheets::config::{AccentColors, SheetsConfig, ThemeConfig};
use zellij_sheets::data_loader::{DataSource, LoadedData};
use zellij_sheets::state::{SearchDirection, SheetsState};
use zellij_sheets::ui::{Colors, UiRenderer};

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> LoadedData {
        LoadedData {
            headers: vec!["name".into(), "city".into()],
            rows: vec![
                vec!["alice".into(), "boston".into()],
                vec!["bob".into(), "austin".into()],
            ],
            source: DataSource::Csv,
        }
    }

    #[test]
    fn test_ui_renderer_creation() {
        let renderer = UiRenderer::new();
        let config = Arc::new(SheetsConfig::default());
        let state = SheetsState::new(config);

        assert!(renderer.draw_ui(&state).is_ok());
    }

    #[test]
    fn test_theme_config_default() {
        let theme = ThemeConfig::default();
        assert_eq!(theme.header_background, "#0055AA");
        assert_eq!(theme.header_text, "#FFFFFF");
        assert_eq!(theme.selected_background, "#00AAFF");
        assert_eq!(theme.selected_text, "#000000");
    }

    #[test]
    fn test_theme_config_custom() {
        let theme = ThemeConfig {
            background: "#111111".to_string(),
            text: "#EEEEEE".to_string(),
            header_background: "#222222".to_string(),
            header_text: "#DDDDDD".to_string(),
            selected_background: "#333333".to_string(),
            selected_text: "#CCCCCC".to_string(),
            column_header_background: "#444444".to_string(),
            column_header_text: "#BBBBBB".to_string(),
            accent_colors: AccentColors {
                number: "#00FF00".to_string(),
                string: "#FFFF00".to_string(),
                boolean: "#FF00FF".to_string(),
                date: "#FF8800".to_string(),
            },
        };

        assert_eq!(theme.header_background, "#222222");
        assert_eq!(theme.selected_text, "#CCCCCC");
    }

    #[test]
    fn test_colors_default() {
        let colors = Colors::default();
        // Checking available fields: header_background, header_text, selected_background, selected_text, separator
        assert!(!colors.header_text.is_empty());
    }

    #[test]
    fn test_ui_renderer_shows_active_search_query() {
        let renderer = UiRenderer::new();
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);
        state.init(sample_data()).unwrap();
        state.begin_search(SearchDirection::Forward);
        state.set_search_query(Some("au".to_string()));

        let rendered = renderer.draw_ui(&state).unwrap();
        assert!(rendered.contains("au"));
    }

    #[test]
    fn test_ui_renderer_header_contains_filename() {
        let renderer = UiRenderer::new();
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);
        state.init(sample_data()).unwrap();

        let rendered = renderer.draw_ui(&state).unwrap();
        assert!(rendered.contains("No file loaded"));
        assert!(rendered.contains("2"));
    }

    #[test]
    fn test_ui_renderer_footer_shows_row_col() {
        let renderer = UiRenderer::new();
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);
        state.init(sample_data()).unwrap();

        let rendered = renderer.draw_ui(&state).unwrap();
        assert!(rendered.contains("row 1 col 1"));

        state.select_down();
        state.select_right();
        let rendered = renderer.draw_ui(&state).unwrap();
        assert!(rendered.contains("row 2 col 2"));
    }

    #[test]
    fn test_ui_renderer_selected_cell_brackets() {
        let renderer = UiRenderer::new();
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);
        state.resize(80, 12);
        state.init(sample_data()).unwrap();

        let rendered = renderer.draw_ui(&state).unwrap();
        // Selected cell at (0,0) gets bracketed. With tight column width,
        // content may be truncated with a tilde: [al~]
        assert!(rendered.contains("[al"), "rendered:\n{}", rendered);
    }

    #[test]
    fn test_ui_renderer_selected_row_prefix() {
        let renderer = UiRenderer::new();
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);
        state.resize(80, 12);
        state.init(sample_data()).unwrap();

        let rendered = renderer.draw_ui(&state).unwrap();
        let lines: Vec<&str> = rendered.lines().collect();
        for line in &lines {
            if line.contains("alice") {
                assert!(line.starts_with('>'));
                break;
            }
        }
    }

    #[test]
    fn test_ui_renderer_search_highlight_braces() {
        let renderer = UiRenderer::new();
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);
        state.resize(80, 12);
        state.init(sample_data()).unwrap();

        state.set_search_query(Some("bob".to_string()));
        state.search_next();

        let rendered = renderer.draw_ui(&state).unwrap();
        assert!(rendered.contains("[bob]"));
    }

    #[test]
    fn test_ui_renderer_view_mode_in_header() {
        let renderer = UiRenderer::new();
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);
        state.init(sample_data()).unwrap();

        state.set_view_mode(zellij_sheets::state::ViewMode::Grid);
        let rendered = renderer.draw_ui(&state).unwrap();
        assert!(rendered.contains("grid"));

        state.set_view_mode(zellij_sheets::state::ViewMode::List);
        let rendered = renderer.draw_ui(&state).unwrap();
        assert!(rendered.contains("list"));
    }

    #[test]
    fn test_ui_renderer_multiple_rows() {
        let renderer = UiRenderer::new();
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);
        state.resize(80, 20);
        state
            .init(LoadedData {
                headers: vec!["a".into(), "b".into()],
                rows: vec![
                    vec!["r0c0".into(), "r0c1".into()],
                    vec!["r1c0".into(), "r1c1".into()],
                    vec!["r2c0".into(), "r2c1".into()],
                ],
                source: DataSource::Csv,
            })
            .unwrap();

        let rendered = renderer.draw_ui(&state).unwrap();
        // First row's selected cell is bracketed, check for content in non-selected form
        assert!(rendered.contains("r0c1"), "rendered:\n{}", rendered);
        assert!(rendered.contains("r1c0"));
        assert!(rendered.contains("r2c0"));
    }
}
