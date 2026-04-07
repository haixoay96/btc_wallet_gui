use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::i18n::t;
use crate::theme::{text_scaled,secondary_button_style,
    text_primary_color, text_secondary_color, text_muted_color,
    text_color, Colors};
use iced::{
    widget::{button, column, container, row, text, Space},
    Element, Length,
};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};

/// Tracks which help hints the user has dismissed
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HelpDismissals {
    /// Set of topic IDs that have been dismissed
    dismissed: HashSet<String>,
}

impl HelpDismissals {
    pub fn new() -> Self {
        Self {
            dismissed: HashSet::new(),
        }
    }

    /// Check if a topic has been dismissed
    pub fn is_dismissed(&self, topic_id: &str) -> bool {
        self.dismissed.contains(topic_id)
    }

    /// Mark a topic as dismissed
    pub fn dismiss(&mut self, topic_id: &str) {
        self.dismissed.insert(topic_id.to_string());
    }

    /// Un-dismiss a topic (show it again)
    pub fn restore(&mut self, topic_id: &str) {
        self.dismissed.remove(topic_id);
    }

    /// Clear all dismissals
    pub fn clear_all(&mut self) {
        self.dismissed.clear();
    }

    /// Get count of dismissed topics
    pub fn dismissed_count(&self) -> usize {
        self.dismissed.len()
    }
}

/// Help hint banner - shows once until dismissed
pub fn help_hint_banner<'a, Message: Clone + 'a>(
    topic_id: &'a str,
    icon: Bootstrap,
    title: &'a str,
    description: &'a str,
    on_dismiss: Message,
    on_toggle_expand: Option<Message>,
) -> Element<'a, Message> {
    let expand_icon = if on_toggle_expand.is_some() {
        Bootstrap::ChevronDown.to_string()
    } else {
        String::new()
    };

    let mut header_row = row![
        text(icon.to_string()).size(14)
            .font(BOOTSTRAP_FONT)
            .style(text_color(Colors::ACCENT_TEAL)),
        Space::with_width(6),
        text(title).size(12)
            .style(text_primary_color())
            .width(Length::Fill),
    ]
    .align_y(iced::Alignment::Center);

    if on_toggle_expand.is_some() {
        header_row = header_row.push(
            text(expand_icon).size(10)
                .font(BOOTSTRAP_FONT)
                .style(text_secondary_color()),
        );
    }

    header_row = header_row.push(Space::with_width(4));
    header_row = header_row.push(
        button(
            text(Bootstrap::X.to_string()).size(10)
                .font(BOOTSTRAP_FONT)
                .style(text_secondary_color()),
        )
        .on_press(on_dismiss)
        .padding(4)
        .style(secondary_button_style()),
    );

    let mut content = column![header_row];

    if let Some(toggle_msg) = on_toggle_expand {
        let expanded_text: Element<'_, Message> = text(description).size(11)
            .style(text_secondary_color())
            .width(Length::Fill)
            .into();

        let body = container(expanded_text)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    0.2, 0.2, 0.25, 0.3,
                ))),
                border: iced::border::rounded(4),
                ..Default::default()
            })
            .padding(8);

        content = content.push(Space::with_height(4));
        content = content.push(
            container(column![].push(
                button("Xem chi tiết").on_press(toggle_msg).padding(6).style(secondary_button_style()),
            ))
            .padding(iced::padding::Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 20.0,
            }),
        );
    }

    container(content)
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.2, 0.3, 0.4, 0.15))),
            border: iced::border::rounded(8),
            ..Default::default()
        })
        .padding(10)
        .width(Length::Fill)
        .into()
}

/// "Show Help" toggle button for settings
pub fn show_help_reset_button<'a, Message: Clone + 'a>(
    on_reset: Message,
    dismissed_count: usize,
) -> Element<'a, Message> {
    if dismissed_count == 0 {
        return container(
            text(t("Không có gợi ý nào bị ẩn", "No hidden hints")).size(12)
                .style(text_muted_color()),
        )
        .into();
    }

    let reset_label = format!(
        "{} ({})",
        t("Khôi phục tất cả gợi ý", "Restore all hints"),
        dismissed_count
    );

    container(
        button(text(reset_label.to_string()).size(12))
            .on_press(on_reset)
            .padding(8)
            .style(secondary_button_style()),
    )
    .into()
}
