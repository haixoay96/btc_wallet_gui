use crate::i18n::t;
use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Color, Element, Length,
};

use crate::theme::{popup_dialog_style, text_color,
    text_primary_color, text_secondary_color,
    Colors};

/// Platform-aware key modifier display (Cmd on macOS, Ctrl elsewhere)
pub fn ctrl_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    }
}

/// Keyboard shortcut definition
pub struct KeyboardShortcut {
    pub keys: Vec<&'static str>,
    pub description_vi: &'static str,
    pub description_en: &'static str,
}

impl KeyboardShortcut {
    pub fn all() -> Vec<Self> {
        let ctrl = ctrl_label();
        vec![
            // Navigation shortcuts
            Self {
                keys: vec![ctrl, "1"],
                description_vi: "Màn hình Tổng quan",
                description_en: "Dashboard screen",
            },
            Self {
                keys: vec![ctrl, "2"],
                description_vi: "Màn hình Quản lý ví",
                description_en: "Wallets screen",
            },
            Self {
                keys: vec![ctrl, "3"],
                description_vi: "Màn hình Gửi BTC",
                description_en: "Send screen",
            },
            Self {
                keys: vec![ctrl, "4"],
                description_vi: "Màn hình Nhận BTC",
                description_en: "Receive screen",
            },
            Self {
                keys: vec![ctrl, "5"],
                description_vi: "Màn hình Lịch sử giao dịch",
                description_en: "Transaction History screen",
            },
            Self {
                keys: vec![ctrl, "6"],
                description_vi: "Màn hình Cài đặt",
                description_en: "Settings screen",
            },
            // Clipboard shortcuts
            Self {
                keys: vec![ctrl, "C"],
                description_vi: "Sao chép địa chỉ / TxID đã chọn",
                description_en: "Copy selected address / TxID",
            },
            Self {
                keys: vec![ctrl, "V"],
                description_vi: "Dán địa chỉ vào ô nhận (màn Gửi)",
                description_en: "Paste address into Receive field (Send screen)",
            },
            // Form shortcuts
            Self {
                keys: vec![ctrl, "Enter"],
                description_vi: "Gửi form (Đăng nhập / Gửi BTC)",
                description_en: "Submit form (Login / Send BTC)",
            },
            Self {
                keys: vec![ctrl, "S"],
                description_vi: "Lưu trạng thái ví thủ công",
                description_en: "Save wallet state manually",
            },
            // Search shortcut
            Self {
                keys: vec![ctrl, "F"],
                description_vi: "Focus vào ô tìm kiếm (Lịch sử)",
                description_en: "Focus search box (History)",
            },
            // Modal/Popup shortcuts
            Self {
                keys: vec!["Esc"],
                description_vi: "Đóng popup / modal / thông báo",
                description_en: "Close popup / modal / notification",
            },
            // Help shortcuts
            Self {
                keys: vec![ctrl, "/"],
                description_vi: "Hiển thị trợ giúp phím tắt",
                description_en: "Show keyboard shortcuts help",
            },
            Self {
                keys: vec!["F1"],
                description_vi: "Hiển thị trợ giúp phím tắt",
                description_en: "Show keyboard shortcuts help",
            },
        ]
    }
}

fn key_badge(key: &str) -> Element<'static, ()> {
    let is_modifier = key == "Ctrl" || key == "⌘" || key == "Shift" || key == "Alt";
    let text_color_val = if is_modifier {
        Colors::TEXT_SECONDARY
    } else {
        Colors::TEXT_PRIMARY
    };

    let key_owned = key.to_string();
    container(text(key_owned).size(11).style(text_color(text_color_val)))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.2))),
            border: iced::border::rounded(4),
            ..Default::default()
        })
        .padding(6)
        .into()
}

fn shortcut_section_header(vi: &'static str, en: &'static str) -> Element<'static, ()> {
    container(text(t(vi, en)).size(12).style(text_color(Colors::ACCENT_TEAL)))
        .padding(iced::padding::Padding {
            top: 8.0,
            right: 0.0,
            bottom: 4.0,
            left: 0.0,
        })
        .into()
}

/// Keyboard shortcuts help popup with categorized sections
pub fn shortcuts_help_popup() -> Element<'static, ()> {
    let shortcuts = KeyboardShortcut::all();

    let mut content = column![].spacing(0).width(Length::Fill);

    // Navigation section
    content = content.push(shortcut_section_header(
        "Điều hướng",
        "Navigation",
    ));
    for shortcut in shortcuts.iter().take(6) {
        content = content.push(shortcut_row(shortcut));
    }

    // Clipboard section
    content = content.push(shortcut_section_header(
        "Sao chép & Dán",
        "Clipboard",
    ));
    for shortcut in shortcuts.iter().skip(6).take(2) {
        content = content.push(shortcut_row(shortcut));
    }

    // Form & Actions section
    content = content.push(shortcut_section_header(
        "Form & Thao tác",
        "Forms & Actions",
    ));
    for shortcut in shortcuts.iter().skip(8).take(2) {
        content = content.push(shortcut_row(shortcut));
    }

    // Search & Help section
    content = content.push(shortcut_section_header(
        "Tìm kiếm & Trợ giúp",
        "Search & Help",
    ));
    for shortcut in shortcuts.iter().skip(10) {
        content = content.push(shortcut_row(shortcut));
    }

    scrollable(
        column![
            Space::with_height(8),
            content,
            Space::with_height(8),
        ]
        .spacing(0)
        .width(Length::Fill),
    )
    .into()
}

fn shortcut_row(shortcut: &KeyboardShortcut) -> Element<'static, ()> {
    let keys_row = row(
        shortcut
            .keys
            .iter()
            .map(|key| key_badge(key))
            .collect::<Vec<_>>(),
    )
    .spacing(4)
    .align_y(iced::Alignment::Center);

    let item = row![
        keys_row,
        Space::with_width(16),
        text(t(shortcut.description_vi, shortcut.description_en))
            .size(12)
            .style(text_secondary_color()),
    ]
    .align_y(iced::Alignment::Center);

    container(item)
        .padding(iced::padding::Padding {
            top: 4.0,
            right: 8.0,
            bottom: 4.0,
            left: 8.0,
        })
        .into()
}
