use iced::Theme;

/// Detect system theme preference
/// Returns `Theme::Light` or `Theme::Dark` based on OS settings.
/// Fallbacks to `Theme::Dark` if detection fails.
pub fn detect_system_theme() -> Theme {
    match dark_light::detect() {
        dark_light::Mode::Dark => Theme::Dark,
        dark_light::Mode::Light => Theme::Light,
        dark_light::Mode::Default => Theme::Dark, // Fallback
    }
}
