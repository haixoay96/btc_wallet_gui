use iced::Theme;

/// Detect system theme preference
/// Returns `Theme::Light` or `Theme::Dark` based on OS settings.
/// Fallbacks to `Theme::Dark` if detection fails.
pub fn detect_system_theme() -> Theme {
    match dark_light::detect() {
        Ok(dark_light::Mode::Dark) => Theme::Dark,
        Ok(dark_light::Mode::Light) => Theme::Light,
        Ok(_) | Err(_) => Theme::Dark, // Fallback
    }
}
