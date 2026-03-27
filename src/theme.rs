use iced::overlay::menu;
use iced::widget::{button, container, pick_list, text, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// Exodus-inspired theme for Bitcoin Wallet GUI
#[derive(Debug, Clone, Copy)]
pub struct ExodusTheme;

/// Color palette inspired by Exodus wallet
pub struct Colors;

impl Colors {
    // Background colors
    pub const BG_PRIMARY: Color = Color::from_rgb(0.09, 0.09, 0.14); // #171724
    pub const BG_SECONDARY: Color = Color::from_rgb(0.12, 0.12, 0.18); // #1F1F2E
    pub const BG_CARD: Color = Color::from_rgb(0.15, 0.15, 0.22); // #262638
    pub const BG_INPUT: Color = Color::from_rgb(0.18, 0.18, 0.26); // #2E2E42
    pub const BG_HOVER: Color = Color::from_rgb(0.22, 0.22, 0.32); // #383852

    // Accent colors
    pub const ACCENT_PURPLE: Color = Color::from_rgb(0.48, 0.38, 1.0); // #7B61FF
    pub const ACCENT_TEAL: Color = Color::from_rgb(0.0, 0.83, 0.67); // #00D4AA
    pub const ACCENT_PINK: Color = Color::from_rgb(1.0, 0.42, 0.62); // #FF6B9D
    pub const ACCENT_BLUE: Color = Color::from_rgb(0.4, 0.7, 1.0); // #66B3FF

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.95, 0.95, 0.95); // #F2F2F2
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.65, 0.65, 0.75); // #A6A6BF
    pub const TEXT_MUTED: Color = Color::from_rgb(0.45, 0.45, 0.55); // #73738C

    // Status colors
    pub const SUCCESS: Color = Color::from_rgb(0.0, 0.83, 0.67); // #00D4AA
    pub const ERROR: Color = Color::from_rgb(1.0, 0.35, 0.35); // #FF5959
    pub const WARNING: Color = Color::from_rgb(1.0, 0.75, 0.0); // #FFBF00

    // Gradient colors
    pub const GRADIENT_START: Color = Color::from_rgb(0.48, 0.38, 1.0); // Purple
    pub const GRADIENT_END: Color = Color::from_rgb(0.0, 0.83, 0.67); // Teal

    // Border colors
    pub const BORDER: Color = Color::from_rgb(0.25, 0.25, 0.35); // #404059
    pub const BORDER_FOCUSED: Color = Color::from_rgb(0.48, 0.38, 1.0); // Purple
}

/// Helper function to create a Color with alpha
pub fn color_with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

/// Style for primary buttons (gradient purple to teal)
pub fn primary_button_style() -> Box<dyn Fn(&Theme, button::Status) -> button::Style> {
    Box::new(|_theme: &Theme, _status: button::Status| button::Style {
        background: Some(Background::Color(Colors::ACCENT_PURPLE)),
        text_color: Colors::TEXT_PRIMARY,
        border: Border {
            radius: 12.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow {
            color: color_with_alpha(Colors::ACCENT_PURPLE, 0.3),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
    })
}

/// Style for secondary buttons
pub fn secondary_button_style() -> Box<dyn Fn(&Theme, button::Status) -> button::Style> {
    Box::new(|_theme: &Theme, _status: button::Status| button::Style {
        background: Some(Background::Color(Colors::BG_CARD)),
        text_color: Colors::TEXT_PRIMARY,
        border: Border {
            radius: 12.0.into(),
            width: 1.0,
            color: Colors::BORDER,
        },
        shadow: Shadow::default(),
    })
}

/// Style for danger buttons
pub fn danger_button_style() -> Box<dyn Fn(&Theme, button::Status) -> button::Style> {
    Box::new(|_theme: &Theme, _status: button::Status| button::Style {
        background: Some(Background::Color(Colors::ERROR)),
        text_color: Colors::TEXT_PRIMARY,
        border: Border {
            radius: 12.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow {
            color: color_with_alpha(Colors::ERROR, 0.3),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
    })
}

/// Style for cards
pub fn card_style() -> Box<dyn Fn(&Theme) -> container::Style> {
    Box::new(|_theme: &Theme| container::Style {
        background: Some(Background::Color(Colors::BG_CARD)),
        border: Border {
            radius: 16.0.into(),
            width: 1.0,
            color: Colors::BORDER,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        text_color: Some(Colors::TEXT_PRIMARY),
    })
}

/// Style for input fields
pub fn input_style() -> Box<dyn Fn(&Theme, text_input::Status) -> text_input::Style> {
    Box::new(
        |_theme: &Theme, _status: text_input::Status| text_input::Style {
            background: Background::Color(Colors::BG_INPUT),
            border: Border {
                radius: 12.0.into(),
                width: 1.0,
                color: Colors::BORDER,
            },
            icon: Colors::TEXT_MUTED,
            placeholder: Colors::TEXT_MUTED,
            value: Colors::TEXT_PRIMARY,
            selection: Colors::ACCENT_PURPLE,
        },
    )
}

/// Style for focused input fields
pub fn input_focused_style() -> Box<dyn Fn(&Theme, text_input::Status) -> text_input::Style> {
    Box::new(
        |_theme: &Theme, _status: text_input::Status| text_input::Style {
            background: Background::Color(Colors::BG_INPUT),
            border: Border {
                radius: 12.0.into(),
                width: 2.0,
                color: Colors::BORDER_FOCUSED,
            },
            icon: Colors::TEXT_MUTED,
            placeholder: Colors::TEXT_MUTED,
            value: Colors::TEXT_PRIMARY,
            selection: Colors::ACCENT_PURPLE,
        },
    )
}

/// Style for dropdown / pick list
pub fn pick_list_style() -> Box<dyn Fn(&Theme, pick_list::Status) -> pick_list::Style> {
    Box::new(|_theme: &Theme, status: pick_list::Status| {
        let border_color = match status {
            pick_list::Status::Active => Colors::BORDER,
            pick_list::Status::Hovered | pick_list::Status::Opened => Colors::BORDER_FOCUSED,
        };

        pick_list::Style {
            text_color: Colors::TEXT_PRIMARY,
            placeholder_color: Colors::TEXT_MUTED,
            handle_color: Colors::TEXT_SECONDARY,
            background: Background::Color(Colors::BG_INPUT),
            border: Border {
                radius: 12.0.into(),
                width: 1.0,
                color: border_color,
            },
        }
    })
}

/// Style for dropdown menu items
pub fn pick_list_menu_style() -> Box<dyn Fn(&Theme) -> menu::Style> {
    Box::new(|_theme: &Theme| menu::Style {
        background: Background::Color(Colors::BG_CARD),
        border: Border {
            radius: 12.0.into(),
            width: 1.0,
            color: Colors::BORDER,
        },
        text_color: Colors::TEXT_PRIMARY,
        selected_text_color: Colors::TEXT_PRIMARY,
        selected_background: Background::Color(color_with_alpha(Colors::ACCENT_PURPLE, 0.35)),
    })
}

/// Style for sidebar
pub fn sidebar_style() -> Box<dyn Fn(&Theme) -> container::Style> {
    Box::new(|_theme: &Theme| container::Style {
        background: Some(Background::Color(Colors::BG_SECONDARY)),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        text_color: Some(Colors::TEXT_PRIMARY),
    })
}

/// Text style functions
pub fn text_color(color: Color) -> Box<dyn Fn(&Theme) -> text::Style> {
    Box::new(move |_theme: &Theme| text::Style { color: Some(color) })
}
