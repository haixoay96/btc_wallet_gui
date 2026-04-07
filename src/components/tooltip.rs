use iced::{
    widget::{column, container, mouse_area, row, text, Space},
    Color, Element, Length,
};

use crate::theme::{text_scaled,text_color,
    text_primary_color, text_secondary_color, text_muted_color,
    Colors};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};

/// Info box with icon and text - supports i18n via String
pub fn info_box(title: impl Into<String>, description: impl Into<String>) -> Element<'static, ()> {
    let title = title.into();
    let description = description.into();

    container(
        row![
            text(Bootstrap::InfoCircle.to_string()).size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_color(Colors::ACCENT_BLUE)),
            Space::with_width(8),
            column![
                text(title).size(13)
                    .style(text_primary_color()),
                Space::with_height(2),
                text(description).size(11)
                    .style(text_secondary_color()),
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
            text(Bootstrap::ExclamationTriangle.to_string()).size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_color(Colors::WARNING)),
            Space::with_width(8),
            column![
                text(title).size(13)
                    .style(text_primary_color()),
                Space::with_height(2),
                text(description).size(11)
                    .style(text_secondary_color()),
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

/// Tooltip content with multi-language support
pub struct TooltipContent {
    pub title_vi: &'static str,
    pub title_en: &'static str,
    pub description_vi: &'static str,
    pub description_en: &'static str,
}

impl TooltipContent {
    pub fn new(
        title_vi: &'static str,
        title_en: &'static str,
        description_vi: &'static str,
        description_en: &'static str,
    ) -> Self {
        Self {
            title_vi,
            title_en,
            description_vi,
            description_en,
        }
    }
}

/// Hover tooltip that appears after delay
/// Returns a widget wrapped with mouse_area for hover detection
pub fn hover_tooltip_with_trigger<'a, Message: Clone + 'a>(
    content: TooltipContent,
    on_hover_start: Message,
    on_hover_end: Message,
) -> impl Fn(Element<'a, Message>) -> Element<'a, Message> {
    move |child: Element<'a, Message>| {
        let tooltip_view = tooltip_bubble(&content);
        
        // For now, return child without tooltip overlay
        // Full hover implementation requires state management in parent view
        // This provides the tooltip content for use in parent's view
        child
    }
}

/// Tooltip bubble widget - the actual tooltip display
pub fn tooltip_bubble(content: &TooltipContent) -> Element<'static, ()> {
    container(
        column![
            row![
                text(Bootstrap::InfoCircle.to_string()).size(12)
                    .font(BOOTSTRAP_FONT)
                    .style(text_color(Colors::ACCENT_BLUE)),
                Space::with_width(4),
                text(crate::i18n::t(content.title_vi, content.title_en)).size(12)
                    .style(text_primary_color()),
            ]
            .align_y(iced::Alignment::Center),
            Space::with_height(4),
            text(crate::i18n::t(content.description_vi, content.description_en)).size(10)
                .style(text_secondary_color())
                .width(Length::Shrink),
        ]
        .spacing(0)
        .width(Length::Shrink),
    )
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(0.1, 0.1, 0.15, 0.95))),
        border: iced::border::rounded(6),
        ..Default::default()
    })
    .padding(8)
    .width(Length::Shrink)
    .into()
}

/// Help topic definition for contextual help system
#[derive(Clone)]
pub struct HelpTopic {
    pub id: &'static str,
    pub icon: Bootstrap,
    pub title_vi: &'static str,
    pub title_en: &'static str,
    pub description_vi: &'static str,
    pub description_en: &'static str,
    pub detail_vi: Option<&'static str>,
    pub detail_en: Option<&'static str>,
}

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
            id,
            icon,
            title_vi,
            title_en,
            description_vi,
            description_en,
            detail_vi: None,
            detail_en: None,
        }
    }

    pub fn with_detail(
        mut self,
        detail_vi: &'static str,
        detail_en: &'static str,
    ) -> Self {
        self.detail_vi = Some(detail_vi);
        self.detail_en = Some(detail_en);
        self
    }

    /// Get title based on current language
    pub fn title(&self) -> &str {
        crate::i18n::t(self.title_vi, self.title_en)
    }

    /// Get description based on current language
    pub fn description(&self) -> &str {
        crate::i18n::t(self.description_vi, self.description_en)
    }

    /// Get detail based on current language
    pub fn detail(&self) -> Option<&str> {
        match crate::i18n::current_language() {
            crate::i18n::AppLanguage::Vietnamese => self.detail_vi,
            crate::i18n::AppLanguage::English => self.detail_en,
        }
    }
}

/// Render a help topic as an inline expandable panel
pub fn help_topic_panel<'a, Message: Clone + 'a>(
    topic_id: &'a str,
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
        text(icon.to_string()).size(14)
            .font(BOOTSTRAP_FONT)
            .style(text_color(Colors::ACCENT_TEAL)),
        Space::with_width(6),
        text(title).size(12)
            .style(text_primary_color())
            .width(Length::Fill),
        text(icon_char).size(10)
            .font(BOOTSTRAP_FONT)
            .style(text_secondary_color()),
    ]
    .align_y(iced::Alignment::Center)
    .into();

    let header_element: Element<'_, Message> = mouse_area(header)
        .on_press(on_toggle)
        .into();
    let mut content = column![header_element].spacing(0);

    if is_expanded {
        let description_text: Element<'_, Message> = text(description).size(11)
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

        let mut body = column![
            Space::with_height(6),
            desc_container,
        ];

        if let Some(detail_text) = detail {
            body = body.push(Space::with_height(6));
            body = body.push(
                text(detail_text).size(10)
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

/// Help button icon - can be placed next to form fields
pub fn help_button<'a, Message: Clone + 'a>(on_press: Message) -> Element<'a, Message> {
    container(
        text(Bootstrap::QuestionCircle.to_string()).size(14)
            .font(BOOTSTRAP_FONT)
            .style(text_secondary_color()),
    )
    .padding(4)
    .into()
}
