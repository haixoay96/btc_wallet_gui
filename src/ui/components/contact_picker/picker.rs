use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Element, Length,
};

use crate::i18n::t;

use super::structure::*;
use crate::storage::address_book::{AddressBook, ContactEntry};
use crate::ui::theme::{
    get_theme_colors, input_style, primary_button_style, secondary_button_style, text_color,
    text_muted_color, text_primary_color, text_secondary_color, Colors,
};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

/// Contact picker for Send screen
/// Shows a list of contacts that can be selected to fill the address field
pub fn contact_picker_view<'a, Message: Clone + 'a>(
    address_book: &'a AddressBook,
    search_query: &'a str,
    on_search_changed: impl Fn(String) -> Message + 'a,
    on_select_contact: impl Fn(&ContactEntry) -> Message + 'a,
    on_delete_contact: impl Fn(String) -> Message + 'a,
    on_edit_contact: impl Fn(String) -> Message + 'a,
    on_add_new_contact: Message,
) -> Element<'a, Message> {
    let contacts = address_book.search(search_query);

    let search_box = container(
        row![
            text(Bootstrap::Search.to_string())
                .size(14)
                .font(BOOTSTRAP_FONT)
                .style(text_secondary_color()),
            Space::with_width(6),
            text_input(t("Tìm kiếm contact...", "Search contacts..."), search_query)
                .on_input(on_search_changed)
                .padding(8)
                .size(12)
                .width(Length::Fill)
                .style(input_style()),
        ]
        .align_y(iced::Alignment::Center),
    )
    .style(|theme: &iced::Theme| {
        let colors = get_theme_colors(theme);
        iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                colors.text_muted.r,
                colors.text_muted.g,
                colors.text_muted.b,
                0.15,
            ))),
            border: iced::border::rounded(8),
            ..Default::default()
        }
    })
    .padding(iced::padding::Padding {
        top: 6.0,
        right: 8.0,
        bottom: 6.0,
        left: 8.0,
    });

    let mut content = column![
        row![
            text(Bootstrap::Person.to_string())
                .size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_color(Colors::ACCENT_TEAL)),
            Space::with_width(6),
            text(t("Contact", "Contact"))
                .size(14)
                .style(text_primary_color()),
            Space::with_width(Length::Fill),
            button(
                text(Bootstrap::Plus.to_string())
                    .size(14)
                    .font(BOOTSTRAP_FONT)
                    .style(text_primary_color()),
            )
            .on_press(on_add_new_contact)
            .padding([4, 8])
            .style(primary_button_style()),
        ]
        .align_y(iced::Alignment::Center),
        Space::with_height(8),
        search_box,
    ]
    .spacing(8);

    if contacts.is_empty() {
        if search_query.is_empty() {
            content = content.push(
                container(
                    text(t(
                        "Chưa có contact nào. Thêm contact mới!",
                        "No contacts yet. Add a new one!",
                    ))
                    .size(12)
                    .style(text_muted_color()),
                )
                .padding(16)
                .center_x(Length::Fill),
            );
        } else {
            content = content.push(
                container(
                    text(t("Không tìm thấy contact nào", "No contacts found"))
                        .size(12)
                        .style(text_muted_color()),
                )
                .padding(16)
                .center_x(Length::Fill),
            );
        }
    } else {
        let mut contact_list = column![];
        for contact in contacts {
            let contact_item = row![
                text(Bootstrap::PersonFill.to_string())
                    .size(14)
                    .font(BOOTSTRAP_FONT)
                    .style(text_color(Colors::ACCENT_PURPLE)),
                Space::with_width(8),
                column![
                    text(&contact.name).size(13).style(text_primary_color()),
                    text(&contact.address)
                        .size(10)
                        .style(text_secondary_color()),
                ]
                .spacing(2),
                Space::with_width(Length::Fill),
                if !contact.note.is_empty() {
                    container(text(&contact.note).size(9).style(text_muted_color()))
                        .style(|theme: &iced::Theme| {
                            let colors = get_theme_colors(theme);
                            iced::widget::container::Style {
                                background: Some(iced::Background::Color(iced::Color::from_rgba(
                                    colors.text_muted.r,
                                    colors.text_muted.g,
                                    colors.text_muted.b,
                                    0.15,
                                ))),
                                border: iced::border::rounded(8),
                                ..Default::default()
                            }
                        })
                        .padding([2, 6])
                } else {
                    container(Space::with_width(0))
                },
                Space::with_width(4),
                button(
                    text(Bootstrap::Pencil.to_string())
                        .size(10)
                        .font(BOOTSTRAP_FONT)
                        .style(text_color(Colors::ACCENT_TEAL)),
                )
                .on_press(on_edit_contact(contact.id.clone()))
                .padding([4, 6])
                .style(secondary_button_style()),
                button(
                    text(Bootstrap::Trash.to_string())
                        .size(11)
                        .font(BOOTSTRAP_FONT)
                        .style(text_color(Colors::ERROR)),
                )
                .on_press(on_delete_contact(contact.id.clone()))
                .padding([4, 6])
                .style(secondary_button_style()),
            ]
            .align_y(iced::Alignment::Center)
            .padding(8);

            contact_list = contact_list.push(
                button(container(contact_item).width(Length::Fill))
                    .on_press(on_select_contact(contact))
                    .padding(0)
                    .style(secondary_button_style())
                    .width(Length::Fill),
            );
            contact_list = contact_list.push(Space::with_height(4));
        }

        content = content.push(scrollable(contact_list).height(Length::Fill));
    }

    container(content)
        .style(|theme: &iced::Theme| {
            let colors = get_theme_colors(theme);
            iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    colors.bg_secondary.r,
                    colors.bg_secondary.g,
                    colors.bg_secondary.b,
                    0.95,
                ))),
                border: iced::border::rounded(12),
                ..Default::default()
            }
        })
        .padding(12)
        .width(Length::Fill)
        .height(Length::Shrink)
        .into()
}

/// Contact form for adding/editing contacts
pub fn contact_form_view<'a, Message: Clone + 'a>(
    name: &'a str,
    address: &'a str,
    note: &'a str,
    is_editing: bool,
    address_error: Option<&'a str>,
    on_name_changed: impl Fn(String) -> Message + 'a,
    on_address_changed: impl Fn(String) -> Message + 'a,
    on_note_changed: impl Fn(String) -> Message + 'a,
    on_save: Message,
    on_cancel: Message,
    on_delete: Option<Message>,
) -> Element<'a, Message> {
    let title = if is_editing {
        t("Sửa Contact", "Edit Contact")
    } else {
        t("Thêm Contact Mới", "Add New Contact")
    };

    let save_label = if is_editing {
        t("Cập nhật", "Update")
    } else {
        t("Thêm Contact", "Add Contact")
    };

    // Validate address is not empty and looks like a valid BTC address
    let is_address_valid = !address.trim().is_empty() && address_error.is_none();
    let is_name_valid = !name.trim().is_empty();
    let can_save = is_address_valid && is_name_valid;

    let save_button = if can_save {
        button(text(save_label).size(12))
            .on_press(on_save)
            .padding(10)
            .style(primary_button_style())
    } else {
        button(text(save_label).size(12))
            .padding(10)
            .style(primary_button_style())
    };

    let delete_button = if is_editing {
        if let Some(delete_msg) = on_delete {
            Some(
                button(
                    text(Bootstrap::Trash.to_string())
                        .size(12)
                        .font(BOOTSTRAP_FONT)
                        .style(text_color(Colors::ERROR)),
                )
                .on_press(delete_msg)
                .padding([6, 10])
                .style(secondary_button_style()),
            )
        } else {
            None
        }
    } else {
        None
    };

    let mut buttons = row![
        save_button,
        Space::with_width(8),
        button(text(t("Hủy", "Cancel")).size(12))
            .on_press(on_cancel)
            .padding(10)
            .style(secondary_button_style()),
    ];

    if let Some(del_btn) = delete_button {
        buttons = buttons.push(Space::with_width(8));
        buttons = buttons.push(del_btn);
    }

    buttons = buttons.align_y(iced::Alignment::Center);

    container(
        column![
            row![
                text(Bootstrap::PersonPlus.to_string())
                    .size(16)
                    .font(BOOTSTRAP_FONT)
                    .style(text_color(Colors::ACCENT_TEAL)),
                Space::with_width(6),
                text(title).size(14).style(text_primary_color()),
            ]
            .align_y(iced::Alignment::Center),
            Space::with_height(12),
            column![
                row![
                    text(t("Tên", "Name"))
                        .size(12)
                        .style(text_secondary_color()),
                    Space::with_width(Length::Fill),
                    text(t(
                        "(Ctrl+Shift+V để dán tiếng Việt)",
                        "(Ctrl+Shift+V to paste Vietnamese)"
                    ))
                    .size(9)
                    .style(text_muted_color()),
                ]
                .align_y(iced::Alignment::Center),
                Space::with_height(4),
                text_input(
                    t("VD: Ví cá nhân, Binance...", "e.g. Alice, Binance..."),
                    name
                )
                .on_input(on_name_changed)
                .padding(8)
                .size(12)
                .style(input_style()),
            ]
            .spacing(4),
            Space::with_height(8),
            column![
                row![
                    text(t("Địa chỉ BTC", "BTC Address"))
                        .size(12)
                        .style(text_secondary_color()),
                    Space::with_width(Length::Fill),
                    if address_error.is_some() && !address.trim().is_empty() {
                        text(Bootstrap::ExclamationTriangle.to_string())
                            .size(10)
                            .font(BOOTSTRAP_FONT)
                            .style(text_color(Colors::ERROR))
                    } else if is_address_valid {
                        text(Bootstrap::CheckCircle.to_string())
                            .size(10)
                            .font(BOOTSTRAP_FONT)
                            .style(text_color(Colors::SUCCESS))
                    } else {
                        text("").size(10)
                    },
                ]
                .align_y(iced::Alignment::Center),
                Space::with_height(4),
                text_input(t("VD: bc1q...", "e.g. bc1q..."), address)
                    .on_input(on_address_changed)
                    .padding(8)
                    .size(12)
                    .style(input_style()),
                if address_error.is_some() && !address.trim().is_empty() {
                    text(address_error.unwrap())
                        .size(11)
                        .style(text_color(Colors::ERROR))
                } else {
                    text("").size(11)
                },
            ]
            .spacing(4),
            Space::with_height(8),
            column![
                row![
                    text(t("Ghi chú (tùy chọn)", "Note (optional)"))
                        .size(12)
                        .style(text_secondary_color()),
                    Space::with_width(Length::Fill),
                    text(t(
                        "(Ctrl+Shift+V để dán tiếng Việt)",
                        "(Ctrl+Shift+V to paste Vietnamese)"
                    ))
                    .size(9)
                    .style(text_muted_color()),
                ]
                .align_y(iced::Alignment::Center),
                Space::with_height(4),
                text_input(
                    t(
                        "VD: Ví cá nhân, Exchange...",
                        "e.g. Personal wallet, Exchange..."
                    ),
                    note
                )
                .on_input(on_note_changed)
                .padding(8)
                .size(12)
                .style(input_style()),
            ]
            .spacing(4),
            Space::with_height(12),
            buttons,
        ]
        .spacing(4),
    )
    .style(|theme: &iced::Theme| {
        let colors = get_theme_colors(theme);
        iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                colors.bg_card.r,
                colors.bg_card.g,
                colors.bg_card.b,
                0.98,
            ))),
            border: iced::border::rounded(12),
            ..Default::default()
        }
    })
    .padding(16)
    .width(Length::Fill)
    .into()
}
