use iced::{
    widget::{button, container, row, text, Space},
    Element, Length,
};

use crate::theme::{secondary_button_style, text_color, Colors};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};

/// Tooltip with help icon
pub fn help_tooltip(content: Element<'static, ()>, tooltip_text: &str) -> Element<'static, ()> {
    row![
        content,
        Space::with_width(6),
        container(
            text(Bootstrap::QuestionCircle.to_string())
                .size(14)
                .font(BOOTSTRAP_FONT)
                .style(text_color(Colors::TEXT_MUTED)),
        )
        .style(secondary_button_style())
        .padding(4),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}

/// Simple help icon button
pub fn help_icon(tooltip_text: String) -> Element<'static, ()> {
    container(
        text(Bootstrap::QuestionCircle.to_string())
            .size(14)
            .font(BOOTSTRAP_FONT)
            .style(text_color(Colors::TEXT_MUTED)),
    )
    .style(secondary_button_style())
    .padding(6)
    .into()
}

/// Info box with icon and text
pub fn info_box(title: &str, description: &str) -> Element<'static, ()> {
    container(
        row![
            text(Bootstrap::InfoCircle.to_string())
                .size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_color(Colors::ACCENT_BLUE)),
            Space::with_width(8),
            iced::widget::column![
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
    .style(move |_| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.4, 0.7, 1.0, 0.1,
        ))),
        border: iced::border::rounded(8),
        ..Default::default()
    })
    .padding(12)
    .width(Length::Fill)
    .into()
}

/// Warning box
pub fn warning_box(title: &str, description: &str) -> Element<'static, ()> {
    container(
        row![
            text(Bootstrap::ExclamationTriangle.to_string())
                .size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_color(Colors::WARNING)),
            Space::with_width(8),
            iced::widget::column![
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
    .style(move |_| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            1.0, 0.75, 0.0, 0.1,
        ))),
        border: iced::border::rounded(8),
        ..Default::default()
    })
    .padding(12)
    .width(Length::Fill)
    .into()
}
