use crate::i18n::t;
use iced::{
    widget::{button, column, container, row, text, Space},
    Color, Element, Length,
};

use crate::theme::{popup_dialog_style, text_color, Colors};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};

/// Keyboard shortcut definition
pub struct KeyboardShortcut {
    pub keys: Vec<&'static str>,
    pub description_vi: &'static str,
    pub description_en: &'static str,
}

impl KeyboardShortcut {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                keys: vec!["Ctrl", "1"],
                description_vi: "Dashboard",
                description_en: "Dashboard",
            },
            Self {
                keys: vec!["Ctrl", "2"],
                description_vi: "Ví",
                description_en: "Wallets",
            },
            Self {
                keys: vec!["Ctrl", "3"],
                description_vi: "Gửi",
                description_en: "Send",
            },
            Self {
                keys: vec!["Ctrl", "4"],
                description_vi: "Nhận",
                description_en: "Receive",
            },
            Self {
                keys: vec!["Ctrl", "5"],
                description_vi: "Lịch sử",
                description_en: "History",
            },
            Self {
                keys: vec!["Ctrl", "6"],
                description_vi: "Cài đặt",
                description_en: "Settings",
            },
            Self {
                keys: vec!["Ctrl", "C"],
                description_vi: "Sao chép địa chỉ",
                description_en: "Copy address",
            },
            Self {
                keys: vec!["Ctrl", "Enter"],
                description_vi: "Gửi form",
                description_en: "Submit form",
            },
            Self {
                keys: vec!["Esc"],
                description_vi: "Đóng popup",
                description_en: "Close popup",
            },
        ]
    }
}

fn key_badge(key: &str) -> Element<'static, ()> {
    let key_owned = key.to_string();
    container(text(key_owned).size(11).style(text_color(Colors::TEXT_PRIMARY)))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.2))),
            border: iced::border::rounded(4),
            ..Default::default()
        })
        .padding([2, 6])
        .into()
}

/// Keyboard shortcuts help content (không wrap container full screen)
pub fn shortcuts_help_popup() -> Element<'static, ()> {
    let shortcuts = KeyboardShortcut::all();

    let mut items = column![];
    for shortcut in shortcuts {
        let keys_row = row(
            shortcut
                .keys
                .into_iter()
                .map(|key| key_badge(key))
                .collect::<Vec<_>>(),
        )
        .spacing(4)
        .align_y(iced::Alignment::Center);

        let item = row![
            keys_row,
            Space::with_width(12),
            text(t(shortcut.description_vi, shortcut.description_en))
                .size(12)
                .style(text_color(Colors::TEXT_SECONDARY)),
        ]
        .align_y(iced::Alignment::Center);

        items = items.push(item);
        items = items.push(Space::with_height(6));
    }

    column![
        Space::with_height(12),
        items,
    ]
    .spacing(0)
    .width(Length::Fill)
    .into()
}
