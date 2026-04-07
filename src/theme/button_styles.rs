use iced::widget::button;
use iced::{Background, Border, Color, Shadow, Theme, Vector};
use super::structure::ButtonStyleFn;
use super::palette::get_theme_colors;
use super::structure::color_with_alpha;

/// Style for primary buttons (teal accent)
pub fn primary_button_style() -> Box<ButtonStyleFn> {
    Box::new(|theme: &Theme, status: button::Status| {
        let colors = get_theme_colors(theme);
        let (background, shadow_color, shadow_blur) = match status {
            button::Status::Hovered => (
                Background::Color(colors.accent_teal),
                color_with_alpha(colors.accent_teal, 0.35),
                16.0,
            ),
            _ => (
                Background::Color(colors.accent_teal),
                color_with_alpha(colors.accent_teal, 0.25),
                12.0,
            ),
        };

        button::Style {
            background: Some(background),
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
            border: Border { radius: 12.0.into(), width: 0.0, color: Color::TRANSPARENT },
            shadow: Shadow { color: shadow_color, offset: Vector::new(0.0, 6.0), blur_radius: shadow_blur },
        }
    })
}

/// Style for gradient buttons (purple accent)
pub fn gradient_button_style() -> Box<ButtonStyleFn> {
    Box::new(|theme: &Theme, status: button::Status| {
        let colors = get_theme_colors(theme);
        let (background, shadow_color, shadow_blur) = match status {
            button::Status::Hovered => (
                Background::Color(colors.accent_purple),
                color_with_alpha(colors.gradient_start, 0.40),
                16.0,
            ),
            _ => (
                Background::Color(colors.gradient_start),
                color_with_alpha(colors.gradient_start, 0.30),
                12.0,
            ),
        };

        button::Style {
            background: Some(background),
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
            border: Border { radius: 12.0.into(), width: 0.0, color: Color::TRANSPARENT },
            shadow: Shadow { color: shadow_color, offset: Vector::new(0.0, 6.0), blur_radius: shadow_blur },
        }
    })
}

/// Style for selected/active buttons
pub fn selected_button_style() -> Box<ButtonStyleFn> {
    Box::new(|theme: &Theme, status: button::Status| {
        let colors = get_theme_colors(theme);
        let background = match status {
            button::Status::Hovered => Background::Color(color_with_alpha(colors.accent_purple, 0.28)),
            _ => Background::Color(color_with_alpha(colors.accent_purple, 0.18)),
        };

        button::Style {
            background: Some(background),
            text_color: colors.text_primary,
            border: Border { radius: 12.0.into(), width: 1.0, color: color_with_alpha(colors.accent_purple, 0.55) },
            shadow: Shadow::default(),
        }
    })
}

/// Style for muted/disabled buttons
pub fn muted_button_style() -> Box<ButtonStyleFn> {
    Box::new(|theme: &Theme, _status: button::Status| {
        let colors = get_theme_colors(theme);
        button::Style {
            background: Some(Background::Color(colors.bg_input)),
            text_color: colors.text_muted,
            border: Border { radius: 12.0.into(), width: 1.0, color: colors.border_subtle },
            shadow: Shadow::default(),
        }
    })
}

/// Style for secondary buttons (outline style)
pub fn secondary_button_style() -> Box<ButtonStyleFn> {
    Box::new(|theme: &Theme, status: button::Status| {
        let colors = get_theme_colors(theme);
        let (background, border_color, shadow) = match status {
            button::Status::Hovered => (
                Background::Color(colors.bg_hover),
                colors.border_focused,
                Shadow { color: color_with_alpha(colors.border, 0.3), offset: Vector::new(0.0, 3.0), blur_radius: 8.0 },
            ),
            _ => (
                Background::Color(colors.bg_card),
                colors.border,
                Shadow { color: color_with_alpha(colors.border, 0.2), offset: Vector::new(0.0, 2.0), blur_radius: 6.0 },
            ),
        };

        button::Style {
            background: Some(background),
            text_color: colors.text_primary,
            border: Border { radius: 12.0.into(), width: 1.0, color: border_color },
            shadow,
        }
    })
}

/// Style for info buttons (blue accent)
pub fn info_style() -> Box<ButtonStyleFn> {
    Box::new(|theme: &Theme, status: button::Status| {
        let colors = get_theme_colors(theme);
        let (background, shadow_color) = match status {
            button::Status::Hovered => (
                Background::Color(colors.bg_hover),
                color_with_alpha(colors.accent_blue, 0.5),
            ),
            _ => (
                Background::Color(colors.accent_blue),
                color_with_alpha(colors.accent_blue, 0.3),
            ),
        };

        button::Style {
            background: Some(background),
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
            border: Border { radius: 12.0.into(), width: 0.0, color: Color::TRANSPARENT },
            shadow: Shadow { color: shadow_color, offset: Vector::new(0.0, 4.0), blur_radius: 12.0 },
        }
    })
}

/// Style for warning buttons
pub fn warning_style() -> Box<ButtonStyleFn> {
    Box::new(|theme: &Theme, status: button::Status| {
        let colors = get_theme_colors(theme);
        let (background, shadow_color) = match status {
            button::Status::Hovered => (
                Background::Color(colors.warning),
                color_with_alpha(colors.warning, 0.5),
            ),
            _ => (
                Background::Color(colors.warning),
                color_with_alpha(colors.warning, 0.3),
            ),
        };

        button::Style {
            background: Some(background),
            text_color: Color::from_rgb(0.0, 0.0, 0.0),
            border: Border { radius: 12.0.into(), width: 0.0, color: Color::TRANSPARENT },
            shadow: Shadow { color: shadow_color, offset: Vector::new(0.0, 4.0), blur_radius: 12.0 },
        }
    })
}

/// Style for danger/delete buttons
pub fn danger_button_style() -> Box<ButtonStyleFn> {
    Box::new(|theme: &Theme, _status: button::Status| {
        let colors = get_theme_colors(theme);
        button::Style {
            background: Some(Background::Color(colors.error)),
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
            border: Border { radius: 12.0.into(), width: 0.0, color: Color::TRANSPARENT },
            shadow: Shadow { color: color_with_alpha(colors.error, 0.3), offset: Vector::new(0.0, 4.0), blur_radius: 12.0 },
        }
    })
}
