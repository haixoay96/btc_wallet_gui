use iced::{
    widget::{column, container, row, text, Space},
    Color, Element, Length,
};

use crate::theme::{text_color, Colors};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};

/// Simple help icon
pub fn help_icon() -> Element<'static, ()> {
    container(
        text(Bootstrap::QuestionCircle.to_string())
            .size(14)
            .font(BOOTSTRAP_FONT)
            .style(text_color(Colors::TEXT_MUTED)),
    )
    .padding(6)
    .into()
}

/// Help icon with label
pub fn help_tooltip(content: Element<'static, ()>) -> Element<'static, ()> {
    row![
        content,
        Space::with_width(6),
        help_icon(),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}

/// Info box with icon and text
pub fn info_box(title: &'static str, description: &'static str) -> Element<'static, ()> {
    container(
        row![
            text(Bootstrap::InfoCircle.to_string())
                .size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_color(Colors::ACCENT_BLUE)),
            Space::with_width(8),
            column![
                text(title)
                    .size(13)
                    .style(text_color(Colors::TEXT_PRIMARY)),
                Space::with_height(2),
                text(description)
                    .size(11)
                    .style(text_color(Colors::TEXT_SECONDARY)),
            ]
            .spacing(0),
        ]
        .align_y(iced::Alignment::Center),
    )
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(0.4, 0.7, 1.0, 0.1))),
        border: iced::border::rounded(8),
        ..Default::default()
    })
    .padding(12)
    .width(Length::Fill)
    .into()
}

/// Warning box
pub fn warning_box(title: &'static str, description: &'static str) -> Element<'static, ()> {
    container(
        row![
            text(Bootstrap::ExclamationTriangle.to_string())
                .size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_color(Colors::WARNING)),
            Space::with_width(8),
            column![
                text(title)
                    .size(13)
                    .style(text_color(Colors::TEXT_PRIMARY)),
                Space::with_height(2),
                text(description)
                    .size(11)
                    .style(text_color(Colors::TEXT_SECONDARY)),
            ]
            .spacing(0),
        ]
        .align_y(iced::Alignment::Center),
    )
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(1.0, 0.75, 0.0, 0.1))),
        border: iced::border::rounded(8),
        ..Default::default()
    })
    .padding(12)
    .width(Length::Fill)
    .into()
}
