use iced::{Color, Theme, widget::Text};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use super::colors::{DarkColors, LightColors, HighContrastColors};
use super::structure::color_with_alpha;

/// Global high contrast state
static HIGH_CONTRAST_ENABLED: AtomicBool = AtomicBool::new(false);

/// Global font scale (stored as u32 = scale * 1000 for atomic operations)
static FONT_SCALE_RAW: AtomicU32 = AtomicU32::new(1000); // 1.0 * 1000

/// Set global high contrast mode
pub fn set_high_contrast(enabled: bool) {
    HIGH_CONTRAST_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Get current high contrast state
pub fn is_high_contrast() -> bool {
    HIGH_CONTRAST_ENABLED.load(Ordering::Relaxed)
}

/// Set global font scale
pub fn set_font_scale(scale: f64) {
    FONT_SCALE_RAW.store((scale * 1000.0) as u32, Ordering::Relaxed);
}

/// Get current font scale
pub fn get_font_scale() -> f64 {
    FONT_SCALE_RAW.load(Ordering::Relaxed) as f64 / 1000.0
}

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

    pub fn high_contrast() -> Self {
        Self {
            bg_primary: HighContrastColors::BG_PRIMARY,
            bg_secondary: HighContrastColors::BG_SECONDARY,
            bg_card: HighContrastColors::BG_CARD,
            bg_input: HighContrastColors::BG_INPUT,
            bg_hover: HighContrastColors::BG_HOVER,
            accent_purple: HighContrastColors::ACCENT_PURPLE,
            accent_teal: HighContrastColors::ACCENT_TEAL,
            accent_blue: HighContrastColors::ACCENT_BLUE,
            text_primary: HighContrastColors::TEXT_PRIMARY,
            text_secondary: HighContrastColors::TEXT_SECONDARY,
            text_muted: HighContrastColors::TEXT_MUTED,
            text_placeholder: Color::from_rgb(0.70, 0.70, 0.70),
            success: HighContrastColors::SUCCESS,
            error: HighContrastColors::ERROR,
            warning: HighContrastColors::WARNING,
            gradient_start: HighContrastColors::GRADIENT_START,
            border: HighContrastColors::BORDER,
            border_focused: HighContrastColors::BORDER_FOCUSED,
            border_subtle: HighContrastColors::BORDER_SUBTLE,
        }
    }
}

/// Get colors based on current theme and global high contrast setting
pub fn get_theme_colors(theme: &Theme) -> ThemeColorPalette {
    if is_high_contrast() {
        ThemeColorPalette::high_contrast()
    } else {
        match theme {
            Theme::Light => ThemeColorPalette::light(),
            Theme::Dark | _ => ThemeColorPalette::dark(),
        }
    }
}
