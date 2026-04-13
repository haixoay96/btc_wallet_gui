use super::structure::HelpTopic;

use iced::{
    widget::{column, container, mouse_area, row, text, Space},
    Color, Element, Length,
};

use crate::ui::theme::{
    text_accent_blue_color, text_accent_teal_color, text_muted_color, text_primary_color,
    text_secondary_color, text_warning_color,
};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

/// Info box with icon and text - supports i18n via String
pub fn info_box(title: impl Into<String>, description: impl Into<String>) -> Element<'static, ()> {
    let title = title.into();
    let description = description.into();

    container(
        row![
            text(Bootstrap::InfoCircle.to_string())
                .size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_accent_blue_color()),
            Space::with_width(8),
            column![
                text(title).size(13).style(text_primary_color()),
                Space::with_height(2),
                text(description).size(11).style(text_secondary_color()),
            ]
            .spacing(0),
        ]
        .align_y(iced::Alignment::Center),
    )
    .style(|_| iced::widget::container::Style {
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

/// Warning box - supports i18n via String
pub fn warning_box(
    title: impl Into<String>,
    description: impl Into<String>,
) -> Element<'static, ()> {
    let title = title.into();
    let description = description.into();

    container(
        row![
            text(Bootstrap::ExclamationTriangle.to_string())
                .size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_warning_color()),
            Space::with_width(8),
            column![
                text(title).size(13).style(text_primary_color()),
                Space::with_height(2),
                text(description).size(11).style(text_secondary_color()),
            ]
            .spacing(0),
        ]
        .align_y(iced::Alignment::Center),
    )
    .style(|_| iced::widget::container::Style {
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

/// Help topic definition for contextual help system

impl HelpTopic {
    pub fn new(
        id: &'static str,
        icon: Bootstrap,
        title_vi: &'static str,
        title_en: &'static str,
        description_vi: &'static str,
        description_en: &'static str,
    ) -> Self {
        Self {
            id: id.to_string(),
            icon,
            title_vi,
            title_en,
            description_vi,
            description_en,
            detail_vi: None,
            detail_en: None,
        }
    }

    pub fn with_detail(mut self, detail_vi: &'static str, detail_en: &'static str) -> Self {
        self.detail_vi = Some(detail_vi);
        self.detail_en = Some(detail_en);
        self
    }
}

/// Render a help topic as an inline expandable panel
pub fn help_topic_panel<'a, Message: Clone + 'a>(
    _topic_id: &str,
    icon: Bootstrap,
    title: &'a str,
    description: &'a str,
    detail: Option<&'a str>,
    is_expanded: bool,
    on_toggle: Message,
) -> Element<'a, Message> {
    let icon_char = if is_expanded {
        Bootstrap::ChevronUp.to_string()
    } else {
        Bootstrap::ChevronDown.to_string()
    };

    let header: Element<'_, Message> = row![
        text(icon.to_string())
            .size(14)
            .font(BOOTSTRAP_FONT)
            .style(text_accent_teal_color()),
        Space::with_width(6),
        text(title)
            .size(12)
            .style(text_primary_color())
            .width(Length::Fill),
        text(icon_char)
            .size(10)
            .font(BOOTSTRAP_FONT)
            .style(text_secondary_color()),
    ]
    .align_y(iced::Alignment::Center)
    .into();

    let header_element: Element<'_, Message> = mouse_area(header).on_press(on_toggle).into();
    let mut content = column![header_element].spacing(0);

    if is_expanded {
        let description_text: Element<'_, Message> = text(description)
            .size(11)
            .style(text_secondary_color())
            .width(Length::Fill)
            .into();

        let desc_container: Element<'_, Message> = container(description_text)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.2, 0.2, 0.25, 0.3,
                ))),
                border: iced::border::rounded(4),
                ..Default::default()
            })
            .padding(8)
            .into();

        let mut body = column![Space::with_height(6), desc_container,];

        if let Some(detail_text) = detail {
            body = body.push(Space::with_height(6));
            body = body.push(
                text(detail_text)
                    .size(10)
                    .style(text_muted_color())
                    .width(Length::Fill),
            );
        }

        content = content.push(container(body).padding(iced::padding::Padding {
            top: 4.0,
            right: 0.0,
            bottom: 4.0,
            left: 20.0,
        }));
    }

    container(content)
        .style(|_| iced::widget::container::Style {
            border: iced::border::rounded(6),
            ..Default::default()
        })
        .padding(iced::padding::Padding {
            top: 6.0,
            right: 8.0,
            bottom: 6.0,
            left: 8.0,
        })
        .width(Length::Fill)
        .into()
}
