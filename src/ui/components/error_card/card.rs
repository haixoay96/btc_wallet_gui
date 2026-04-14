use iced::{
    Element, Length,
    widget::{Space, button, column, container, row, text},
};

use crate::ui::i18n::t;

use crate::ui::theme::text_color;
use crate::ui::theme::text_error_color;
use crate::ui::theme::text_primary_color;
use crate::ui::theme::text_secondary_color;
use iced_fonts::{BOOTSTRAP_FONT, bootstrap::advanced_text};

/// Error card với retry button
pub fn error_card<Message: 'static + Clone>(
    title: String,
    message: String,
    on_retry: Option<Message>,
) -> Element<'static, Message> {
    let mut content = column![
        row![
            text(advanced_text::x_circle().0)
                .size(24)
                .font(BOOTSTRAP_FONT)
                .style(text_error_color()),
            Space::new().width(12),
            text(title).size(16).style(text_primary_color())
        ]
        .align_y(iced::Alignment::Center),
        Space::new().height(8),
        text(message).size(13).style(text_secondary_color()),
    ]
    .spacing(0);

    if let Some(retry_msg) = on_retry {
        content = content.push(Space::new().height(12));
        content = content.push(
            container(
                button(
                    text(format!(
                        "{} {}",
                        advanced_text::arrow_repeat,
                        t("Thử lại", "Retry")
                    ))
                    .size(13)
                    .font(BOOTSTRAP_FONT)
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
