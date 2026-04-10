use super::colors::{DarkColors, HighContrastColors, LightColors};
use iced::{Color, Theme};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

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
            bg_primary: LightColors::BG_PRIMARY,
            bg_secondary: LightColors::BG_SECONDARY,
            bg_card: LightColors::BG_CARD,
            bg_input: LightColors::BG_INPUT,
            bg_hover: LightColors::BG_HOVER,
            accent_purple: LightColors::ACCENT_PURPLE,
            accent_teal: LightColors::ACCENT_TEAL,
            accent_blue: LightColors::ACCENT_BLUE,
            text_primary: LightColors::TEXT_PRIMARY,
            text_secondary: LightColors::TEXT_SECONDARY,
            text_muted: LightColors::TEXT_MUTED,
            text_placeholder: Color::from_rgb(0.42, 0.44, 0.50),
            success: LightColors::SUCCESS,
            error: LightColors::ERROR,
            warning: LightColors::WARNING,
            gradient_start: LightColors::GRADIENT_START,
            border: LightColors::BORDER,
            border_focused: LightColors::BORDER_FOCUSED,
            border_subtle: LightColors::BORDER_SUBTLE,
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
