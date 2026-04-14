use crate::ui::components::backup_reminder::structure::BackupReminderMessage;
use crate::ui::i18n::t;
use crate::ui::theme::{
    notice_style, text_primary_color, text_scaled, text_secondary_color, NoticeTone,
};
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};

/// Render backup reminder banner
pub fn backup_reminder_banner(
    wallets_needing_backup: usize,
) -> Element<'static, BackupReminderMessage> {
    let message = if wallets_needing_backup == 1 {
        t(
            "Có 1 ví chưa được backup seed phrase.",
            "1 wallet has not been backed up.",
        )
    } else {
        &format!(
            "{}",
            t(
                &format!(
                    "Có {} ví chưa được backup seed phrase.",
                    wallets_needing_backup
                ),
                &format!(
                    "{} wallets have not been backed up.",
                    wallets_needing_backup
                ),
            )
        )
    };

    let detail = t(
        "Mất seed phrase = mất ví vĩnh viễn. Hãy backup ngay!",
        "Losing seed phrase means losing your wallet forever. Backup now!",
    );

    let banner = container(
        column![
            row![
                text("⚠️").size(18),
                text_scaled(&message, 13).style(text_primary_color()),
            ]
            .align_y(Alignment::Center)
            .spacing(8),
            Space::new().height(4),
            text_scaled(detail, 12).style(text_secondary_color()),
            Space::new().height(8),
            row![
                button(text_scaled(t("Backup ngay", "Backup Now"), 12))
                    .on_press(BackupReminderMessage::NavigateToWallets)
                    .padding([6, 12])
                    .style(crate::ui::theme::primary_button_style()),
                Space::new().width(8),
                button(text_scaled(t("Nhắc tôi sau", "Remind me later"), 12))
                    .on_press(BackupReminderMessage::DismissReminder)
                    .padding([6, 12])
                    .style(crate::ui::theme::secondary_button_style()),
            ],
        ]
        .spacing(2),
    )
    .style(notice_style(NoticeTone::Warning))
    .padding(12)
    .width(Length::Fill);

    container(banner).width(Length::Fill).into()
}
