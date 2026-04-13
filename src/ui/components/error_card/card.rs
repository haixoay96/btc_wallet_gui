use iced::{
    widget::{button, column, container, row, text, Space},
    Element, Length,
};

use crate::ui::i18n::t;

use crate::ui::theme::text_color;
use crate::ui::theme::text_error_color;
use crate::ui::theme::text_primary_color;
use crate::ui::theme::text_secondary_color;
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

/// Error card với retry button
pub fn error_card<Message: 'static + Clone>(
    title: String,
    message: String,
    on_retry: Option<Message>,
) -> Element<'static, Message> {
    let mut content = column![
        row![
            text(Bootstrap::XCircle.to_string())
                .size(24)
                .font(BOOTSTRAP_FONT)
                .style(text_error_color()),
            Space::with_width(12),
            text(title).size(16).style(text_primary_color())
        ]
        .align_y(iced::Alignment::Center),
        Space::with_height(8),
        text(message).size(13).style(text_secondary_color()),
    ]
    .spacing(0);

    if let Some(retry_msg) = on_retry {
        content = content.push(Space::with_height(12));
        content = content.push(
            container(
                button(
                    text(format!(
                        "{} {}",
                        Bootstrap::ArrowRepeat,
                        t("Thử lại", "Retry")
                    ))
                    .size(13)
                    .style(text_error_color()),
                )
                .on_press(retry_msg)
                .padding(8),
            )
            .width(Length::Fill)
            .align_x(iced::Alignment::Center),
        );
    }

    container(content).padding(16).width(Length::Fill).into()
}
