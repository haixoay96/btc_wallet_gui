use iced::overlay::menu;
use iced::widget::{button, container, pick_list, text, text_input};
use iced::{Color, Theme};

// Type aliases cho style functions
pub type ButtonStyleFn = dyn Fn(&Theme, button::Status) -> button::Style;
pub type ContainerStyleFn = dyn Fn(&Theme) -> container::Style;
pub type MenuStyleFn = dyn Fn(&Theme) -> menu::Style;
pub type PickListStyleFn = dyn Fn(&Theme, pick_list::Status) -> pick_list::Style;
pub type TextInputStyleFn = dyn Fn(&Theme, text_input::Status) -> text_input::Style;
pub type TextStyleFn = dyn Fn(&Theme) -> text::Style;

/// Notice tone cho các thông báo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeTone {
    Info,
    Success,
    Warning,
    Error,
}

/// Helper function tạo Color với alpha
pub fn color_with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

/// Default Colors type alias - trỏ về DarkColors để tương thích ngược
pub use super::colors::DarkColors as Colors;
