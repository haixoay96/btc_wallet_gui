use iced::overlay::menu;
use iced::widget::{text_input, pick_list};
use iced::{Background, Border, Color, Shadow, Theme, Vector};
use super::structure::{TextInputStyleFn, PickListStyleFn, MenuStyleFn};
use super::palette::get_theme_colors;
use super::structure::color_with_alpha;

/// Style for text input fields with focus states
pub fn input_style() -> Box<TextInputStyleFn> {
    Box::new(|theme: &Theme, status: text_input::Status| {
        let colors = get_theme_colors(theme);
        let (border_color, border_width, _shadow) = match status {
            text_input::Status::Focused => (
                colors.border_focused,
                2.0,
                Shadow { color: color_with_alpha(colors.accent_purple, 0.15), offset: Vector::new(0.0, 2.0), blur_radius: 8.0 },
            ),
            _ => (
                colors.border,
                1.5,
                Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.06), offset: Vector::new(0.0, 1.0), blur_radius: 4.0 },
            ),
        };

        text_input::Style {
            background: Background::Color(colors.bg_input),
            border: Border { radius: 12.0.into(), width: border_width, color: border_color },
            icon: colors.text_secondary,
            placeholder: colors.text_placeholder,
            value: colors.text_primary,
            selection: colors.accent_purple,
        }
    })
}

/// Style for dropdown/picklist buttons
pub fn pick_list_style() -> Box<PickListStyleFn> {
    Box::new(|theme: &Theme, status: pick_list::Status| {
        let colors = get_theme_colors(theme);
        let border_color = match status {
            pick_list::Status::Active => colors.border,
            pick_list::Status::Hovered | pick_list::Status::Opened => colors.border_focused,
        };

        pick_list::Style {
            text_color: colors.text_primary,
            placeholder_color: colors.text_muted,
            handle_color: colors.text_secondary,
            background: Background::Color(colors.bg_input),
            border: Border { radius: 12.0.into(), width: 1.0, color: border_color },
        }
    })
}

/// Style for picklist dropdown menu
pub fn pick_list_menu_style() -> Box<MenuStyleFn> {
    Box::new(|theme: &Theme| {
        let colors = get_theme_colors(theme);
        menu::Style {
            background: Background::Color(colors.bg_card),
            border: Border { radius: 12.0.into(), width: 1.0, color: colors.border },
            text_color: colors.text_primary,
            selected_text_color: colors.text_primary,
            selected_background: Background::Color(color_with_alpha(colors.accent_purple, 0.35)),
        }
    })
}
