use iced::{Color, Theme};
use super::colors::{DarkColors, LightColors};
use super::structure::color_with_alpha;

/// Theme color palette that can be used dynamically
pub struct ThemeColorPalette {
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_card: Color,
    pub bg_input: Color,
    pub bg_hover: Color,
    pub accent_purple: Color,
    pub accent_teal: Color,
    pub accent_blue: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_placeholder: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub gradient_start: Color,
    pub border: Color,
    pub border_focused: Color,
    pub border_subtle: Color,
}

impl ThemeColorPalette {
    pub fn dark() -> Self {
        Self {
            bg_primary: DarkColors::BG_PRIMARY,
            bg_secondary: DarkColors::BG_SECONDARY,
            bg_card: DarkColors::BG_CARD,
            bg_input: DarkColors::BG_INPUT,
            bg_hover: DarkColors::BG_HOVER,
            accent_purple: DarkColors::ACCENT_PURPLE,
            accent_teal: DarkColors::ACCENT_TEAL,
            accent_blue: DarkColors::ACCENT_BLUE,
            text_primary: DarkColors::TEXT_PRIMARY,
            text_secondary: DarkColors::TEXT_SECONDARY,
            text_muted: DarkColors::TEXT_MUTED,
            text_placeholder: Color::from_rgb(0.45, 0.45, 0.55),
            success: DarkColors::SUCCESS,
            error: DarkColors::ERROR,
            warning: DarkColors::WARNING,
            gradient_start: DarkColors::GRADIENT_START,
            border: DarkColors::BORDER,
            border_focused: DarkColors::BORDER_FOCUSED,
            border_subtle: DarkColors::BORDER_SUBTLE,
        }
    }

    pub fn light() -> Self {
        Self {
            bg_primary: Color::from_rgb(0.87, 0.87, 0.90),
            bg_secondary: Color::from_rgb(0.84, 0.84, 0.88),
            bg_card: Color::from_rgb(0.91, 0.91, 0.94),
            bg_input: Color::from_rgb(0.89, 0.89, 0.92),
            bg_hover: Color::from_rgb(0.80, 0.80, 0.85),
            accent_purple: Color::from_rgb(0.38, 0.28, 0.90),
            accent_teal: Color::from_rgb(0.0, 0.62, 0.48),
            accent_blue: Color::from_rgb(0.20, 0.45, 0.90),
            text_primary: Color::from_rgb(0.06, 0.06, 0.08),
            text_secondary: Color::from_rgb(0.25, 0.25, 0.30),
            text_muted: Color::from_rgb(0.42, 0.42, 0.48),
            text_placeholder: Color::from_rgb(0.30, 0.30, 0.35),
            success: Color::from_rgb(0.0, 0.50, 0.35),
            error: Color::from_rgb(0.72, 0.08, 0.08),
            warning: Color::from_rgb(0.72, 0.42, 0.0),
            gradient_start: Color::from_rgb(0.38, 0.28, 0.90),
            border: Color::from_rgb(0.65, 0.65, 0.72),
            border_focused: Color::from_rgb(0.38, 0.28, 0.90),
            border_subtle: Color::from_rgb(0.75, 0.75, 0.80),
        }
    }
}

/// Get colors based on current theme
pub fn get_theme_colors(theme: &Theme) -> ThemeColorPalette {
    match theme {
        Theme::Light => ThemeColorPalette::light(),
        Theme::Dark | _ => ThemeColorPalette::dark(),
    }
}
