use super::structure::*;
use crate::ui::i18n::t;
use crate::ui::theme::{text_muted_color, text_primary_color, text_secondary_color};
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length};

/// Get color for a specific tag based on index
fn get_tag_color(_tag: &str, color_index: usize) -> Color {
    TAG_COLORS[color_index % TAG_COLORS.len()]
}

/// Render a single tag badge
fn tag_badge(tag: String, index: usize, can_remove: bool) -> Element<'static, TagMessage> {
    let bg_color = get_tag_color(&tag, index);
    let display_tag = tag.clone();

    let mut content = row![
        container(Space::new().width(8))
            .width(Length::Fixed(8.0))
            .height(Length::Fixed(16.0))
            .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(bg_color)),
                border: iced::Border {
                    radius: 4.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                snap: false,
                ..Default::default()
            }),
        text(display_tag).size(12).style(text_primary_color()),
    ]
    .align_y(Alignment::Center);

    if can_remove {
        let tag_for_btn = tag.clone();
        content = content.push(
            button(text("×").size(14).style(text_muted_color()))
                .on_press(TagMessage::RemoveTag(tag_for_btn))
                .padding([0, 4])
                .style(crate::ui::theme::secondary_button_style()),
        );
    }

    container(content)
        .style(move |_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color {
                r: bg_color.r,
                g: bg_color.g,
                b: bg_color.b,
                a: 0.15,
            })),
            border: iced::Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            snap: false,
            ..Default::default()
        })
        .padding(4)
        .into()
}

/// Render the full tag picker component
pub fn tag_picker(current_tags: &[String], input_value: &str) -> Element<'static, TagMessage> {
    let mut content = column![].spacing(8);

    // Current tags
    if !current_tags.is_empty() {
        let mut tags_row = row![].spacing(6).align_y(Alignment::Center);

        for (i, tag) in current_tags.iter().enumerate() {
            tags_row = tags_row.push(tag_badge(tag.clone(), i, true));
        }

        // Wrap tags in horizontal scrollable
        content = content.push(
            container(
                scrollable(tags_row)
                    .direction(scrollable::Direction::Horizontal(
                        scrollable::Scrollbar::default(),
                    ))
                    .height(Length::Fixed(60.0)), // Chiều cao đủ lớn để không bị cụt
            )
            .width(Length::Fill)
            .padding([4, 8]), // Padding cho vùng scroll
        );
    } else {
        content = content.push(
            text(t("Chưa có tag nào", "No tags yet"))
                .size(12)
                .style(text_muted_color()),
        );
    }

    // Suggested tags
    let mut suggested_row = row![].spacing(6).align_y(Alignment::Center);

    let available: Vec<&&str> = COMMON_TAGS
        .iter()
        .filter(|t| !current_tags.iter().any(|ct| ct.eq_ignore_ascii_case(t)))
        .collect();

    for (_i, tag) in available.iter().take(5).enumerate() {
        suggested_row = suggested_row.push(
            button(text(tag.to_string()).size(11).style(text_secondary_color()))
                .on_press(TagMessage::SelectTag(tag.to_string()))
                .padding([4, 8])
                .style(crate::ui::theme::secondary_button_style()),
        );
    }

    content = content.push(
        column![
            text(t("Gợi ý:", "Suggestions:"))
                .size(11)
                .style(text_muted_color()),
            suggested_row,
        ]
        .spacing(4),
    );

    // Input for new tag
    content = content.push(
        text_input(t("Thêm tag mới...", "Add new tag..."), input_value)
            .on_input(TagMessage::InputChanged)
            .on_submit(TagMessage::CreateTag(input_value.to_string()))
            .padding(8)
            .size(12)
            .style(crate::ui::theme::input_style()),
    );

    container(content)
        .style(crate::ui::theme::card_style())
        .padding(12)
        .width(Length::Fill)
        .into()
}
