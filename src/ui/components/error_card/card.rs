use iced::{
    widget::{button, column, container, row, text, Space},
    Element, Length,
};

use crate::i18n::t;
use crate::ui::theme::text_primary_color;
use crate::ui::theme::text_secondary_color;
use crate::ui::theme::text_color;
use crate::ui::theme::Colors;
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};

/// Error card với retry button
pub fn error_card<Message: 'static + Clone>(
    title: String,
    message: String,
    on_retry: Option<Message>,
) -> Element<'static, Message> {
    let mut content = column![
        row![
            text(Bootstrap::XCircle.to_string()).size(24)
                .font(BOOTSTRAP_FONT)
                .style(text_color(Colors::ERROR)),
            Space::with_width(12),
            text(title).size(16)
                .style(text_primary_color())
        ]
        .align_y(iced::Alignment::Center),
        Space::with_height(8),
        text(message).size(13)
            .style(text_secondary_color()),
    ]
    .spacing(0);

    if let Some(retry_msg) = on_retry {
        content = content.push(Space::with_height(12));
        content = content.push(
            container(
                button(
                    text(format!("{} {}", Bootstrap::ArrowRepeat.to_string(), t("Thử lại", "Retry")))
                        .size(13)
                        .style(text_color(Colors::ERROR)),
                )
                .on_press(retry_msg)
                .padding(8),
            )
            .width(Length::Fill)
            .align_x(iced::Alignment::Center),
        );
    }

    container(content)
        .padding(16)
        .width(Length::Fill)
        .into()
}
