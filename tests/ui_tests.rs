use std::sync::Arc;
use zellij_sheets::config::{AccentColors, SheetsConfig, ThemeConfig};
use zellij_sheets::data_loader::{DataSource, LoadedData};
use zellij_sheets::state::{SearchDirection, SheetsState};
use zellij_sheets::ui::UiRenderer;

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
    fn test_ui_renderer_with_theme() {
        let theme = ThemeConfig::default();
        let renderer = UiRenderer::with_theme(theme);
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
        let colors = zellij_sheets::ui::Colors::default();
        assert!(colors.foreground.is_none());
        assert!(colors.background.is_none());
    }

    #[test]
    fn test_colors_with_fg() {
        let colors = zellij_sheets::ui::Colors::with_fg("31");
        assert_eq!(colors.foreground, Some("31".to_string()));
        assert!(colors.background.is_none());
    }

    #[test]
    fn test_colors_with_bg() {
        let colors = zellij_sheets::ui::Colors::with_bg("44");
        assert!(colors.foreground.is_none());
        assert_eq!(colors.background, Some("44".to_string()));
    }

    #[test]
    fn test_colors_apply() {
        let colors = zellij_sheets::ui::Colors {
            foreground: Some("31".to_string()),
            background: Some("44".to_string()),
        };
        let result = colors.apply();
        assert!(result.contains("\x1b[31"));
        assert!(result.contains("\x1b[44"));
    }

    #[test]
    fn test_colors_reset() {
        let colors = zellij_sheets::ui::Colors::default();
        let result = colors.reset();
        assert_eq!(result, "\x1b[0m");
    }

    #[test]
    fn test_ui_renderer_set_use_colors() {
        let mut renderer = UiRenderer::new();
        renderer.set_use_colors(false);

        let config = Arc::new(SheetsConfig::default());
        let state = SheetsState::new(config);

        assert!(renderer.draw_ui(&state).is_ok());
    }

    #[test]
    fn test_ui_renderer_set_theme() {
        let mut renderer = UiRenderer::new();
        let theme = ThemeConfig::default();
        renderer.set_theme(theme);

        let config = Arc::new(SheetsConfig::default());
        let state = SheetsState::new(config);

        assert!(renderer.draw_ui(&state).is_ok());
    }

    #[test]
    fn test_ui_renderer_shows_active_search_query() {
        let renderer = UiRenderer::new();
        let config = Arc::new(SheetsConfig::default());
        let mut state = SheetsState::new(config);
        state.init(sample_data()).unwrap();
        state.begin_search(SearchDirection::Forward);
        state.search_append('a');
        state.search_append('u');

        let rendered = renderer.draw_ui(&state).unwrap();
        assert!(rendered.contains("/au"));
    }

    #[test]
    fn test_draw_error() {
        let result = zellij_sheets::ui::draw_error("Test error");
        assert!(result.contains("Error: Test error"));
        assert!(result.contains("\x1b[31"));
    }

    #[test]
    fn test_draw_warning() {
        let result = zellij_sheets::ui::draw_warning("Test warning");
        assert!(result.contains("Warning: Test warning"));
        assert!(result.contains("\x1b[33"));
    }

    #[test]
    fn test_draw_info() {
        let result = zellij_sheets::ui::draw_info("Test info");
        assert!(result.contains("Info: Test info"));
        assert!(result.contains("\x1b[34"));
    }
}
