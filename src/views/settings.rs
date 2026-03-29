use std::{env, fmt, path::PathBuf};

use iced::{
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, Space},
    Alignment, Element, Length,
};

use crate::i18n::{current_language, t, AppLanguage};
use crate::theme::{
    card_style, pick_list_menu_style, pick_list_style, primary_button_style,
    secondary_button_style, text_color, Colors,
};

const BACKUP_LOCATIONS: [BackupLocation; 5] = [
    BackupLocation::Desktop,
    BackupLocation::Downloads,
    BackupLocation::Documents,
    BackupLocation::Home,
    BackupLocation::CurrentDirectory,
];
const APP_LANGUAGES: [AppLanguage; 2] = AppLanguage::ALL;

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    LanguageChanged(AppLanguage),
    ToggleChangePassphrase,
    CurrentPassphraseChanged(String),
    NewPassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    SubmitPassphraseChange,
    ExportLocationChanged(BackupLocation),
    ExportPathChanged(String),
    BrowseExportPath,
    ExportWallet,
    ToggleAbout,
    ToggleClearDataConfirm,
    ClearDataPassphraseChanged(String),
    ConfirmClearData,
    CancelClearData,
}

#[derive(Debug, Clone)]
pub enum SettingsEvent {
    ChangeLanguage(AppLanguage),
    BrowseExportPath,
    ChangePassphrase { current: String, new_passphrase: String },
    ExportWallet(String),
    ClearAllData(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupLocation {
    Home,
    Desktop,
    Documents,
    Downloads,
    CurrentDirectory,
}

impl fmt::Display for BackupLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            BackupLocation::Home => t("Thư mục Home", "Home"),
            BackupLocation::Desktop => t("Desktop", "Desktop"),
            BackupLocation::Documents => t("Documents", "Documents"),
            BackupLocation::Downloads => t("Downloads", "Downloads"),
            BackupLocation::CurrentDirectory => t("Thư mục hiện tại", "Current Folder"),
        };
        f.write_str(label)
    }
}

pub struct SettingsView {
    show_change_passphrase: bool,
    current_passphrase: String,
    new_passphrase: String,
    confirm_passphrase: String,
    export_location: BackupLocation,
    export_path: String,
    show_about: bool,
    show_clear_data_confirm: bool,
    clear_data_passphrase: String,
    error: Option<String>,
    success: Option<String>,
}

impl SettingsView {
    pub fn new() -> Self {
        let export_location = BackupLocation::Desktop;
        Self {
            show_change_passphrase: false,
            current_passphrase: String::new(),
            new_passphrase: String::new(),
            confirm_passphrase: String::new(),
            export_location,
            export_path: default_export_path(export_location),
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
    }

    pub fn set_export_path(&mut self, path: String) {
        self.export_path = path;
        self.error = None;
    }

    pub fn update(&mut self, message: SettingsMessage) -> Option<SettingsEvent> {
        match message {
            SettingsMessage::LanguageChanged(language) => {
                self.error = None;
                self.success = None;
                Some(SettingsEvent::ChangeLanguage(language))
            }
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
                Some(SettingsEvent::ChangePassphrase { current: self.current_passphrase.clone(), new_passphrase: self.new_passphrase.clone() })
            }
            SettingsMessage::ExportLocationChanged(location) => {
                self.export_location = location;
                self.export_path = default_export_path(location);
                self.error = None;
                None
            }
            SettingsMessage::ExportPathChanged(path) => {
                self.export_path = path;
                self.error = None;
                None
            }
            SettingsMessage::BrowseExportPath => Some(SettingsEvent::BrowseExportPath),
            SettingsMessage::ExportWallet => {
                let path = self.export_path.trim();
                if path.is_empty() {
                    self.error = Some(
                        t("Vui lòng nhập đường dẫn export", "Please enter export path").to_string(),
                    );
                    return None;
                }

                self.error = None;
                self.success = None;
                Some(SettingsEvent::ExportWallet(path.to_string()))
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
                Some(SettingsEvent::ClearAllData(self.clear_data_passphrase.clone()))
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

        let language_section = container(
            column![
                text(t("Ngôn ngữ", "Language"))
                    .size(18)
                    .style(text_color(Colors::TEXT_PRIMARY)),
                Space::with_height(8),
                pick_list(
                    APP_LANGUAGES,
                    Some(current_language()),
                    SettingsMessage::LanguageChanged
                )
                .width(Length::Fill)
                .padding(10)
                .style(pick_list_style())
                .menu_style(pick_list_menu_style()),
            ]
            .spacing(8),
        )
        .style(card_style())
        .padding(16)
        .width(Length::Fill);

        content = content.push(language_section);

        let change_passphrase_btn = button(text(t("Đổi passphrase", "Change Passphrase")).size(16))
            .on_press(SettingsMessage::ToggleChangePassphrase)
            .padding(12)
            .width(Length::Fill)
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
                "Import backup chỉ hỗ trợ ở màn hình khởi tạo khi app chưa có passphrase.",
                "Backup import is only supported on the initial screen when app has no passphrase yet."
            ))
                .size(12)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(10),
            text(t("Chọn thư mục lưu backup", "Choose backup directory"))
                .size(12)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            pick_list(
                BACKUP_LOCATIONS,
                Some(self.export_location),
                SettingsMessage::ExportLocationChanged
            )
            .width(Length::Fill)
            .padding(10)
            .style(pick_list_style())
            .menu_style(pick_list_menu_style()),
            Space::with_height(8),
            row![
                text_input(
                    t("Đường dẫn file backup...", "Path to backup file..."),
                    &self.export_path
                )
                    .on_input(SettingsMessage::ExportPathChanged)
                    .padding(10)
                    .size(14)
                    .width(Length::Fill),
                Space::with_width(8),
                button(text(t("Chọn nơi lưu", "Browse")).size(14))
                    .on_press(SettingsMessage::BrowseExportPath)
                    .padding(10)
                    .style(secondary_button_style()),
            ]
            .align_y(Alignment::Center),
            Space::with_height(8),
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
                .style(secondary_button_style());

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
                        "Xác nhận xóa toàn bộ dữ liệu?",
                        "Confirm deleting all data?"
                    ))
                    .size(13)
                    .style(text_color(Colors::ERROR)),
                    Space::with_height(8),
                    text_input(
                        t("Nhập passphrase hiện tại...", "Enter current passphrase..."),
                        &self.clear_data_passphrase
                    )
                    .on_input(SettingsMessage::ClearDataPassphraseChanged)
                    .secure(true)
                    .padding(10)
                    .size(13),
                    Space::with_height(8),
                    button(text(t("Xóa toàn bộ ngay", "Delete Everything")).size(13))
                        .on_press(SettingsMessage::ConfirmClearData)
                        .padding(10)
                        .style(primary_button_style()),
                    Space::with_height(6),
                    button(text(t("Hủy", "Cancel")).size(13))
                        .on_press(SettingsMessage::CancelClearData)
                        .padding(10)
                        .style(secondary_button_style()),
                ]
                .spacing(4),
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
            .width(Length::Fill)
            .style(secondary_button_style());

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
            content = content.push(text(err.as_str()).size(13).style(text_color(Colors::ERROR)));
        }

        if let Some(succ) = &self.success {
            content = content.push(
                text(succ.as_str())
                    .size(13)
                    .style(text_color(Colors::SUCCESS)),
            );
        }

        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn default_export_path(location: BackupLocation) -> String {
    location_dir(location)
        .join("wallet_backup.enc")
        .to_string_lossy()
        .to_string()
}

fn location_dir(location: BackupLocation) -> PathBuf {
    match location {
        BackupLocation::Home => home_dir(),
        BackupLocation::Desktop => home_dir().join("Desktop"),
        BackupLocation::Documents => home_dir().join("Documents"),
        BackupLocation::Downloads => home_dir().join("Downloads"),
        BackupLocation::CurrentDirectory => {
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        }
    }
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
