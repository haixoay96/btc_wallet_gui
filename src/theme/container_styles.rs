use iced::widget::container;
use iced::{Background, Border, Color, Shadow, Theme, Vector};
use super::structure::{ContainerStyleFn, NoticeTone};
use super::palette::get_theme_colors;
use super::structure::color_with_alpha;

/// Style for cards with subtle shadow
pub fn card_style() -> Box<ContainerStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        container::Style {
            background: Some(Background::Color(colors.bg_card)),
            border: Border { radius: 18.0.into(), width: 1.0, color: colors.border_subtle },
            shadow: Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.08), offset: Vector::new(0.0, 4.0), blur_radius: 12.0 },
            text_color: Some(colors.text_primary),
        }
    })
}

/// Style for main screen background
pub fn screen_background_style() -> Box<ContainerStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        container::Style {
            background: Some(Background::Color(colors.bg_primary)),
            border: Border { radius: 0.0.into(), width: 0.0, color: Color::TRANSPARENT },
            shadow: Shadow::default(),
            text_color: Some(colors.text_primary),
        }
    })
}

/// Style for popup overlay (semi-transparent)
pub fn popup_overlay_style() -> Box<ContainerStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        container::Style {
            background: Some(Background::Color(color_with_alpha(colors.bg_secondary, 0.7))),
            border: Border { radius: 0.0.into(), width: 0.0, color: Color::TRANSPARENT },
            shadow: Shadow::default(),
            text_color: Some(colors.text_primary),
        }
    })
}

/// Style for popup dialog card
pub fn popup_dialog_style() -> Box<ContainerStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        container::Style {
            background: Some(Background::Color(colors.bg_card)),
            border: Border { radius: 20.0.into(), width: 1.0, color: color_with_alpha(colors.accent_purple, 0.45) },
            shadow: Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.8), offset: Vector::new(0.0, 16.0), blur_radius: 48.0 },
            text_color: Some(colors.text_primary),
        }
    })
}

/// Style for notice/info boxes based on tone
pub fn notice_style(tone: NoticeTone) -> Box<ContainerStyleFn> {
    Box::new(move |theme: &Theme| {
        let colors = get_theme_colors(theme);
        let (accent, background) = match tone {
            NoticeTone::Info => (colors.accent_blue, color_with_alpha(colors.accent_blue, 0.12)),
            NoticeTone::Success => (colors.success, color_with_alpha(colors.success, 0.12)),
            NoticeTone::Warning => (colors.warning, color_with_alpha(colors.warning, 0.14)),
            NoticeTone::Error => (colors.error, color_with_alpha(colors.error, 0.14)),
        };

        container::Style {
            background: Some(Background::Color(background)),
            border: Border { radius: 16.0.into(), width: 1.0, color: color_with_alpha(accent, 0.45) },
            shadow: Shadow::default(),
            text_color: Some(colors.text_primary),
        }
    })
}

/// Style for sidebar navigation
pub fn sidebar_style() -> Box<ContainerStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        container::Style {
            background: Some(Background::Color(colors.bg_secondary)),
            border: Border { radius: 0.0.into(), width: 0.0, color: Color::TRANSPARENT },
            shadow: Shadow::default(),
            text_color: Some(colors.text_primary),
        }
    })
}

/// Background primary style
pub fn bg_primary_style() -> Box<ContainerStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        container::Style {
            background: Some(Background::Color(colors.bg_primary)),
            border: Border::default(),
            shadow: Shadow::default(),
            text_color: Some(colors.text_primary),
        }
    })
}

/// Background secondary style
pub fn bg_secondary_style() -> Box<ContainerStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        container::Style {
            background: Some(Background::Color(colors.bg_secondary)),
            border: Border::default(),
            shadow: Shadow::default(),
            text_color: Some(colors.text_primary),
        }
    })
}
