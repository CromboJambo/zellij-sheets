use std::sync::Arc;
use zellij_sheets::config::SheetsConfig;
use zellij_sheets::state::SheetsState;
use zellij_sheets::ui::{ThemeConfig, UiRenderer};

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(theme.header_fg, "255");
        assert_eq!(theme.header_bg, "33");
        assert_eq!(theme.selected_fg, "0");
        assert_eq!(theme.selected_bg, "39");
    }

    #[test]
    fn test_theme_config_custom() {
        let theme = ThemeConfig {
            header_fg: "255".to_string(),
            header_bg: "33".to_string(),
            selected_fg: "0".to_string(),
            selected_bg: "39".to_string(),
            current_row_fg: "255".to_string(),
            current_row_bg: "33".to_string(),
            separator_fg: "240".to_string(),
            data_type_number: "32".to_string(),
            data_type_string: "33".to_string(),
            data_type_boolean: "35".to_string(),
            data_type_empty: "90".to_string(),
            data_type_date: "33".to_string(),
        };

        assert_eq!(theme.header_fg, "255");
        assert_eq!(theme.selected_fg, "0");
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
