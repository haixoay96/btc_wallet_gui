use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Alignment, Element, Length,
};

use crate::components::modal;
use crate::i18n::t;
use crate::theme::{
    card_style, danger_button_style, info_style, notice_style, primary_button_style,
    secondary_button_style, text_color, warning_style, Colors, NoticeTone,
};

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    ToggleChangePassphrase,
    CurrentPassphraseChanged(String),
    NewPassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    SubmitPassphraseChange,
    ExportWallet,
    ToggleAbout,
    ToggleClearDataConfirm,
    ClearDataPassphraseChanged(String),
    ConfirmClearData,
    CancelClearData,
}

#[derive(Debug, Clone)]
pub enum SettingsEvent {
    ChangePassphrase {
        current: String,
        new_passphrase: String,
    },
    ExportWallet,
    ClearAllData(String),
}

pub struct SettingsView {
    show_change_passphrase: bool,
    current_passphrase: String,
    new_passphrase: String,
    confirm_passphrase: String,
    show_about: bool,
    pub show_clear_data_confirm: bool,
    clear_data_passphrase: String,
    error: Option<String>,
    success: Option<String>,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            show_change_passphrase: false,
            current_passphrase: String::new(),
            new_passphrase: String::new(),
            confirm_passphrase: String::new(),
            show_about: false,
            show_clear_data_confirm: false,
            clear_data_passphrase: String::new(),
            error: None,
            success: None,
        }
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error = Some(message.into());
        self.success = None;
    }

    pub fn set_success(&mut self, message: impl Into<String>) {
        self.success = Some(message.into());
        self.error = None;
    }

    pub fn clear_sensitive_inputs(&mut self) {
        self.current_passphrase.clear();
        self.new_passphrase.clear();
        self.confirm_passphrase.clear();
        self.clear_data_passphrase.clear();
    }

    pub fn update(&mut self, message: SettingsMessage) -> Option<SettingsEvent> {
        match message {
            SettingsMessage::ToggleChangePassphrase => {
                self.show_change_passphrase = !self.show_change_passphrase;
                self.error = None;
                self.success = None;
                None
            }
            SettingsMessage::CurrentPassphraseChanged(p) => {
                self.current_passphrase = p;
                self.error = None;
                None
            }
            SettingsMessage::NewPassphraseChanged(p) => {
                self.new_passphrase = p;
                self.error = None;
                None
            }
            SettingsMessage::ConfirmPassphraseChanged(p) => {
                self.confirm_passphrase = p;
                self.error = None;
                None
            }
            SettingsMessage::SubmitPassphraseChange => {
                if self.current_passphrase.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập passphrase hiện tại",
                            "Please enter your current passphrase",
                        )
                        .to_string(),
                    );
                    return None;
                }

                if self.new_passphrase.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập passphrase mới",
                            "Please enter a new passphrase",
                        )
                        .to_string(),
                    );
                    return None;
                }

                if self.new_passphrase != self.confirm_passphrase {
                    self.error = Some(
                        t(
                            "Passphrase mới và xác nhận không khớp",
                            "New passphrase and confirmation do not match",
                        )
                        .to_string(),
                    );
                    return None;
                }

                self.error = None;
                self.success = None;
                Some(SettingsEvent::ChangePassphrase {
                    current: self.current_passphrase.clone(),
                    new_passphrase: self.new_passphrase.clone(),
                })
            }
            SettingsMessage::ExportWallet => {
                self.error = None;
                self.success = None;
                Some(SettingsEvent::ExportWallet)
            }
            SettingsMessage::ToggleAbout => {
                self.show_about = !self.show_about;
                None
            }
            SettingsMessage::ToggleClearDataConfirm => {
                self.show_clear_data_confirm = !self.show_clear_data_confirm;
                if !self.show_clear_data_confirm {
                    self.clear_data_passphrase.clear();
                }
                self.error = None;
                self.success = None;
                None
            }
            SettingsMessage::ClearDataPassphraseChanged(value) => {
                self.clear_data_passphrase = value;
                self.error = None;
                None
            }
            SettingsMessage::ConfirmClearData => {
                if self.clear_data_passphrase.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập passphrase hiện tại để xác nhận",
                            "Please enter your current passphrase to confirm",
                        )
                        .to_string(),
                    );
                    return None;
                }

                self.show_clear_data_confirm = false;
                self.error = None;
                self.success = None;
                Some(SettingsEvent::ClearAllData(
                    self.clear_data_passphrase.clone(),
                ))
            }
            SettingsMessage::CancelClearData => {
                self.show_clear_data_confirm = false;
                self.clear_data_passphrase.clear();
                None
            }
        }
    }

    pub fn view(&self) -> Element<'_, SettingsMessage> {
        let title = text(t("Cài đặt", "Settings"))
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let mut content = column![title].spacing(20).padding(32);

        let change_passphrase_btn = button(text(t("Đổi passphrase", "Change Passphrase")).size(16))
            .on_press(SettingsMessage::ToggleChangePassphrase)
            .padding(12)
            .style(secondary_button_style());

        content = content.push(
            container(column![
                text(t("Bảo mật", "Security"))
                    .size(18)
                    .style(text_color(Colors::TEXT_PRIMARY)),
                Space::with_height(12),
                change_passphrase_btn,
            ])
            .style(card_style())
            .padding(16)
            .width(Length::Fill),
        );

        if self.show_change_passphrase {
            let current_input = column![
                text(t("Passphrase hiện tại", "Current Passphrase"))
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input(
                    t("Nhập passphrase hiện tại...", "Enter current passphrase..."),
                    &self.current_passphrase
                )
                .on_input(SettingsMessage::CurrentPassphraseChanged)
                .secure(true)
                .padding(10)
                .size(14)
            ]
            .spacing(2);

            let new_input = column![
                text(t("Passphrase mới", "New Passphrase"))
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input(
                    t("Nhập passphrase mới...", "Enter new passphrase..."),
                    &self.new_passphrase
                )
                .on_input(SettingsMessage::NewPassphraseChanged)
                .secure(true)
                .padding(10)
                .size(14)
            ]
            .spacing(2);

            let confirm_input = column![
                text(t("Xác nhận passphrase mới", "Confirm New Passphrase"))
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input(
                    t("Xác nhận passphrase mới...", "Confirm new passphrase..."),
                    &self.confirm_passphrase
                )
                .on_input(SettingsMessage::ConfirmPassphraseChanged)
                .secure(true)
                .padding(10)
                .size(14)
            ]
            .spacing(2);

            content = content.push(
                container(column![
                    current_input,
                    Space::with_height(12),
                    new_input,
                    Space::with_height(12),
                    confirm_input,
                    Space::with_height(12),
                    button(text(t("Cập nhật passphrase", "Update Passphrase")).size(14))
                        .on_press(SettingsMessage::SubmitPassphraseChange)
                        .padding(12)
                        .style(primary_button_style()),
                ])
                .style(card_style())
                .padding(16)
                .width(Length::Fill),
            );
        }

        let export_section = container(column![
            text(t("Xuất backup", "Export Backup"))
                .size(18)
                .style(text_color(Colors::TEXT_PRIMARY)),
            Space::with_height(8),
            text(t(
                "Backup sẽ được mã hóa bằng passphrase hiện tại",
                "Backup will be encrypted with the current passphrase"
            ))
                .size(12)
                .style(text_color(Colors::TEXT_SECONDARY)),
            text(t(
                "Khuyến nghị: ưu tiên backup mnemonic cho từng wallet thay vì backup toàn app.",
                "Recommended: backup each wallet mnemonic instead of full app backup."
            ))
                .size(12)
                .style(text_color(Colors::WARNING)),
            text(t(
                "Khôi phục app backup chỉ dùng ở màn hình khởi tạo. Khôi phục ví .enc riêng lẻ dùng tại Wallets > Import.",
                "Full app backup restore is only for the startup screen. Individual .enc wallet restore is available in Wallets > Import."
            ))
                .size(12)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(10),
            button(text(t("Xuất backup ví", "Export Wallet Backup")).size(14))
                .on_press(SettingsMessage::ExportWallet)
                .padding(12)
                .style(secondary_button_style()),
        ])
        .style(card_style())
        .padding(16)
        .width(Length::Fill);

        content = content.push(export_section);

        let clear_data_button =
            button(text(t("Xóa toàn bộ dữ liệu ví", "Clear All Wallet Data")).size(14))
                .on_press(SettingsMessage::ToggleClearDataConfirm)
                .padding(12)
                .style(warning_style());

        let mut clear_data_col = column![
            text(t("Vùng nguy hiểm", "Danger Zone"))
                .size(18)
                .style(text_color(Colors::ERROR)),
            Space::with_height(8),
            text(t(
                "Xóa toàn bộ ví và dữ liệu đã lưu trong ứng dụng",
                "Delete all wallets and saved app data"
            ))
            .size(12)
            .style(text_color(Colors::WARNING)),
            Space::with_height(10),
            clear_data_button,
        ]
        .spacing(6);

        if self.show_clear_data_confirm {
            clear_data_col = clear_data_col.push(
                column![
                    text(t(
                        "Thao tác này sẽ xóa toàn bộ ví khỏi máy hiện tại.",
                        "This action will remove every wallet from the current device."
                    ))
                    .size(13)
                    .style(text_color(Colors::ERROR)),
                    text(t(
                        "Bạn sẽ cần app backup hoặc các secret backup riêng của từng ví để khôi phục lại sau này.",
                        "You will need the app backup or each wallet's own secret backup to restore later.",
                    ))
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                ]
                .spacing(6),
            );
        }

        content = content.push(
            container(clear_data_col)
                .style(card_style())
                .padding(16)
                .width(Length::Fill),
        );

        let about_btn = button(text(t("Giới thiệu", "About")).size(16))
            .on_press(SettingsMessage::ToggleAbout)
            .padding(12)
            .style(info_style());

        let mut info_col = column![
            text(t("Thông tin", "Information"))
                .size(18)
                .style(text_color(Colors::TEXT_PRIMARY)),
            Space::with_height(12),
            about_btn,
        ]
        .spacing(8);

        if self.show_about {
            info_col = info_col
                .push(
                    text("Bitcoin Wallet GUI v0.1.0")
                        .size(12)
                        .style(text_color(Colors::TEXT_MUTED)),
                )
                .push(
                    text(t("Xây dựng với iced.rs", "Built with iced.rs"))
                        .size(12)
                        .style(text_color(Colors::TEXT_MUTED)),
                )
                .push(
                    text(t(
                        "Lưu trữ: backup mã hóa (ChaCha20-Poly1305 + Argon2id)",
                        "Storage: encrypted backup (ChaCha20-Poly1305 + Argon2id)",
                    ))
                    .size(12)
                    .style(text_color(Colors::TEXT_MUTED)),
                );
        }

        content = content.push(
            container(info_col)
                .style(card_style())
                .padding(16)
                .width(Length::Fill),
        );

        if let Some(err) = &self.error {
            content = content.push(
                container(
                    text(err.as_str())
                        .size(13)
                        .style(text_color(Colors::TEXT_PRIMARY)),
                )
                .style(notice_style(NoticeTone::Error))
                .padding(12)
                .width(Length::Fill),
            );
        }

        if let Some(succ) = &self.success {
            content = content.push(
                container(
                    text(succ.as_str())
                        .size(13)
                        .style(text_color(Colors::TEXT_PRIMARY)),
                )
                .style(notice_style(NoticeTone::Success))
                .padding(12)
                .width(Length::Fill),
            );
        }

        let base: Element<'_, SettingsMessage> = scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        if self.show_clear_data_confirm {
            let clear_content = column![
                text(t(
                    "Dữ liệu ví trên máy này sẽ bị xóa vĩnh viễn.",
                    "Wallet data on this device will be permanently deleted.",
                ))
                .size(14)
                .style(text_color(Colors::TEXT_PRIMARY)),
                Space::with_height(8),
                text_input(
                    t("Nhập passphrase hiện tại...", "Enter current passphrase..."),
                    &self.clear_data_passphrase
                )
                .on_input(SettingsMessage::ClearDataPassphraseChanged)
                .secure(true)
                .padding(12)
                .size(14),
                Space::with_height(12),
                container(
                    row![
                        button(text(t("Hủy", "Cancel")).size(14))
                            .on_press(SettingsMessage::CancelClearData)
                            .padding(10)
                            .style(secondary_button_style()),
                        Space::with_width(10),
                        button(text(t("Xóa toàn bộ ngay", "Delete Everything")).size(14))
                            .on_press(SettingsMessage::ConfirmClearData)
                            .padding(10)
                            .style(danger_button_style()),
                    ]
                    .spacing(8),
                )
                .width(Length::Fill)
                .align_x(Alignment::Center),
            ]
            .padding(0)
            .spacing(0);

            return modal(
                base.into(),
                t("Xác nhận xóa toàn bộ", "Confirm Full Data Deletion"),
                clear_content.into(),
                SettingsMessage::CancelClearData,
            );
        }

        base
    }
}
