use iced::{
    widget::{column, container, row, text, Space},
    Color, Element, Length,
};

use crate::theme::{text_color, Colors};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};

/// Info box with icon and text - supports i18n via String
pub fn info_box(title: impl Into<String>, description: impl Into<String>) -> Element<'static, ()> {
    let title = title.into();
    let description = description.into();
    
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

/// Warning box - supports i18n via String
pub fn warning_box(title: impl Into<String>, description: impl Into<String>) -> Element<'static, ()> {
    let title = title.into();
    let description = description.into();
    
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
