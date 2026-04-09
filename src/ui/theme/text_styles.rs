use super::palette::{get_font_scale, get_theme_colors};
use super::structure::TextStyleFn;
use iced::widget::text;
use iced::{Color, Theme};

/// Create text element with auto-scaled size based on global font scale
pub fn text_scaled<T: std::fmt::Display>(
    content: T,
    base_size: u16,
) -> iced::widget::Text<'static> {
    let scale = get_font_scale();
    let scaled_size = (base_size as f64 * scale).round() as u16;
    text(content.to_string()).size(scaled_size)
}

/// Text style với màu custom
pub fn text_color(color: Color) -> Box<TextStyleFn> {
    Box::new(move |_theme: &Theme| text::Style { color: Some(color) })
}

/// Text primary color - theme aware
pub fn text_primary_color() -> Box<TextStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        text::Style {
            color: Some(colors.text_primary),
        }
    })
}

/// Text secondary color - theme aware
pub fn text_secondary_color() -> Box<TextStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        text::Style {
            color: Some(colors.text_secondary),
        }
    })
}

/// Text muted color - theme aware
pub fn text_muted_color() -> Box<TextStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        text::Style {
            color: Some(colors.text_muted),
        }
    })
}
