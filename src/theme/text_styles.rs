use iced::widget::text;
use iced::{Color, Theme};
use super::structure::TextStyleFn;
use super::palette::get_theme_colors;

/// Text style với màu custom
pub fn text_color(color: Color) -> Box<TextStyleFn> {
    Box::new(move |_theme: &Theme| text::Style { color: Some(color) })
}

/// Text primary color - theme aware
pub fn text_primary_color() -> Box<TextStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        text::Style { color: Some(colors.text_primary) }
    })
}

/// Text secondary color - theme aware
pub fn text_secondary_color() -> Box<TextStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        text::Style { color: Some(colors.text_secondary) }
    })
}

/// Text muted color - theme aware
pub fn text_muted_color() -> Box<TextStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        text::Style { color: Some(colors.text_muted) }
    })
}
