use super::structure::KeyboardShortcut;

use crate::ui::i18n::t;
use iced::{
    widget::{column, container, row, scrollable, text, Space},
    Color, Element, Length, Theme,
};

use crate::ui::theme::{get_theme_colors, text_scaled, text_secondary_color};

/// Platform-aware key modifier display (Cmd on macOS, Ctrl elsewhere)
pub fn ctrl_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    }
}

/// Keyboard shortcut definition

impl KeyboardShortcut {
    pub fn all() -> Vec<Self> {
        let ctrl = ctrl_label();
        vec![
            // Navigation shortcuts (Direct access)
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
            // Sidebar Navigation (Sequential)
            Self {
                keys: vec!["Up", "Down"],
                description_vi: "Chuyển mục trong Sidebar",
                description_en: "Navigate Sidebar items",
            },
            // Form actions
            Self {
                keys: vec!["Enter", "Space"],
                description_vi: "Xác nhận / Gửi form",
                description_en: "Confirm / Submit form",
            },
            Self {
                keys: vec!["Esc"],
                description_vi: "Đóng popup / Hủy bỏ",
                description_en: "Close popup / Cancel",
            },
            // Clipboard shortcuts
            Self {
                keys: vec![ctrl, "C"],
                description_vi: "Sao chép địa chỉ / TxID",
                description_en: "Copy address / TxID",
            },
            Self {
                keys: vec![ctrl, "V"],
                description_vi: "Dán địa chỉ vào ô nhập",
                description_en: "Paste address into input",
            },
            // Search & Help
            Self {
                keys: vec![ctrl, "F"],
                description_vi: "Focus vào ô tìm kiếm",
                description_en: "Focus search box",
            },
            Self {
                keys: vec![ctrl, "S"],
                description_vi: "Lưu trạng thái ví",
                description_en: "Save wallet state",
            },
            Self {
                keys: vec!["F1"],
                description_vi: "Xem trợ giúp phím tắt",
                description_en: "Show keyboard shortcuts help",
            },
        ]
    }
}

fn key_badge(key: &str) -> Element<'static, ()> {
    let is_modifier = key == "Ctrl" || key == "⌘" || key == "Shift" || key == "Alt";
    let key_owned = key.to_string();
    container(text_scaled(key_owned, 11).style(move |theme: &Theme| {
        let colors = get_theme_colors(theme);
        let text_color_val = if is_modifier {
            colors.text_secondary
        } else {
            colors.text_primary
        };
        text::Style {
            color: Some(text_color_val),
        }
    }))
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.5, 0.5, 0.5, 0.2,
        ))),
        border: iced::border::rounded(4),
        ..Default::default()
    })
    .padding(6)
    .into()
}

fn shortcut_section_header(vi: &'static str, en: &'static str) -> Element<'static, ()> {
    container(text_scaled(t(vi, en), 12).style(move |theme: &Theme| {
        let colors = get_theme_colors(theme);
        text::Style {
            color: Some(colors.accent_teal),
        }
    }))
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

    // Navigation section (Items 0-6)
    content = content.push(shortcut_section_header(
        "Điều hướng (Navigation)",
        "Navigation",
    ));
    for shortcut in shortcuts.iter().take(7) {
        content = content.push(shortcut_row(shortcut));
    }

    // Actions section (Items 7-10)
    content = content.push(shortcut_section_header("Thao tác (Actions)", "Actions"));
    for shortcut in shortcuts.iter().skip(7).take(4) {
        content = content.push(shortcut_row(shortcut));
    }

    // Search & Help section (Items 11-13)
    content = content.push(shortcut_section_header(
        "Tìm kiếm & Trợ giúp",
        "Search & Help",
    ));
    for shortcut in shortcuts.iter().skip(11) {
        content = content.push(shortcut_row(shortcut));
    }

    scrollable(
        column![Space::with_height(8), content, Space::with_height(8),]
            .spacing(0)
            .width(Length::Fill),
    )
    .into()
}

fn shortcut_row(shortcut: &KeyboardShortcut) -> Element<'static, ()> {
    let keys_row = row(shortcut
        .keys
        .iter()
        .map(|key| key_badge(key))
        .collect::<Vec<_>>())
    .spacing(4)
    .align_y(iced::Alignment::Center);

    let item = row![
        keys_row,
        Space::with_width(16),
        text_scaled(t(shortcut.description_vi, shortcut.description_en), 12)
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
