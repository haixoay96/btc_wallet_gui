use std::{env, fmt, path::PathBuf};

use iced::{
    widget::{button, column, container, pick_list, scrollable, text, text_input, Space},
    Element, Length,
};

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

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    ToggleChangePassphrase,
    CurrentPassphraseChanged(String),
    NewPassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    SubmitPassphraseChange,
    ExportLocationChanged(BackupLocation),
    ExportPathChanged(String),
    ExportWallet,
    ToggleAbout,
    ToggleClearDataConfirm,
    ClearDataPassphraseChanged(String),
    ConfirmClearData,
    CancelClearData,
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
            BackupLocation::Home => "Home",
            BackupLocation::Desktop => "Desktop",
            BackupLocation::Documents => "Documents",
            BackupLocation::Downloads => "Downloads",
            BackupLocation::CurrentDirectory => "Current Folder",
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

    pub fn update(&mut self, message: SettingsMessage) -> Option<crate::app::AppMessage> {
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
                    self.error = Some("Vui lòng nhập passphrase hiện tại".to_string());
                    return None;
                }

                if self.new_passphrase.trim().is_empty() {
                    self.error = Some("Vui lòng nhập passphrase mới".to_string());
                    return None;
                }

                if self.new_passphrase != self.confirm_passphrase {
                    self.error = Some("Passphrase mới và xác nhận không khớp".to_string());
                    return None;
                }

                self.error = None;
                self.success = None;
                Some(crate::app::AppMessage::ChangePassphrase {
                    current: self.current_passphrase.clone(),
                    new_passphrase: self.new_passphrase.clone(),
                })
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
            SettingsMessage::ExportWallet => {
                let path = self.export_path.trim();
                if path.is_empty() {
                    self.error = Some("Vui lòng nhập đường dẫn export".to_string());
                    return None;
                }

                self.error = None;
                self.success = None;
                Some(crate::app::AppMessage::ExportWalletBackup(path.to_string()))
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
                    self.error = Some("Vui lòng nhập passphrase hiện tại để xác nhận".to_string());
                    return None;
                }

                self.show_clear_data_confirm = false;
                self.error = None;
                self.success = None;
                Some(crate::app::AppMessage::ClearAllData(
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
        let title = text("Settings")
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let mut content = column![title].spacing(20).padding(32);

        let change_passphrase_btn = button(text("Change Passphrase").size(16))
            .on_press(SettingsMessage::ToggleChangePassphrase)
            .padding(12)
            .width(Length::Fill)
            .style(secondary_button_style());

        content = content.push(
            container(column![
                text("Security")
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
                text("Current Passphrase")
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Enter current passphrase...", &self.current_passphrase)
                    .on_input(SettingsMessage::CurrentPassphraseChanged)
                    .secure(true)
                    .padding(10)
                    .size(14)
            ]
            .spacing(2);

            let new_input = column![
                text("New Passphrase")
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Enter new passphrase...", &self.new_passphrase)
                    .on_input(SettingsMessage::NewPassphraseChanged)
                    .secure(true)
                    .padding(10)
                    .size(14)
            ]
            .spacing(2);

            let confirm_input = column![
                text("Confirm New Passphrase")
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Confirm new passphrase...", &self.confirm_passphrase)
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
                    button(text("Update Passphrase").size(14))
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
            text("Export Backup")
                .size(18)
                .style(text_color(Colors::TEXT_PRIMARY)),
            Space::with_height(8),
            text("Backup sẽ được mã hóa bằng passphrase hiện tại")
                .size(12)
                .style(text_color(Colors::TEXT_SECONDARY)),
            text("Khuyến nghị: ưu tiên backup mnemonic cho từng wallet thay vì backup toàn app.")
                .size(12)
                .style(text_color(Colors::WARNING)),
            text("Import backup chỉ hỗ trợ ở màn hình khởi tạo khi app chưa có passphrase.")
                .size(12)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(10),
            text("Chọn thư mục lưu backup")
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
            text_input("Path to backup file...", &self.export_path)
                .on_input(SettingsMessage::ExportPathChanged)
                .padding(10)
                .size(14),
            Space::with_height(8),
            button(text("Export Wallet Backup").size(14))
                .on_press(SettingsMessage::ExportWallet)
                .padding(12)
                .style(secondary_button_style()),
        ])
        .style(card_style())
        .padding(16)
        .width(Length::Fill);

        content = content.push(export_section);

        let clear_data_button = button(text("Clear All Wallet Data").size(14))
            .on_press(SettingsMessage::ToggleClearDataConfirm)
            .padding(12)
            .style(secondary_button_style());

        let mut clear_data_col = column![
            text("Danger Zone")
                .size(18)
                .style(text_color(Colors::ERROR)),
            Space::with_height(8),
            text("Xóa toàn bộ ví và dữ liệu đã lưu trong ứng dụng")
                .size(12)
                .style(text_color(Colors::WARNING)),
            Space::with_height(10),
            clear_data_button,
        ]
        .spacing(6);

        if self.show_clear_data_confirm {
            clear_data_col = clear_data_col.push(
                column![
                    text("Xác nhận xóa toàn bộ dữ liệu?")
                        .size(13)
                        .style(text_color(Colors::ERROR)),
                    Space::with_height(8),
                    text_input("Nhập passphrase hiện tại...", &self.clear_data_passphrase)
                        .on_input(SettingsMessage::ClearDataPassphraseChanged)
                        .secure(true)
                        .padding(10)
                        .size(13),
                    Space::with_height(8),
                    button(text("Xóa toàn bộ ngay").size(13))
                        .on_press(SettingsMessage::ConfirmClearData)
                        .padding(10)
                        .style(primary_button_style()),
                    Space::with_height(6),
                    button(text("Hủy").size(13))
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

        let about_btn = button(text("About").size(16))
            .on_press(SettingsMessage::ToggleAbout)
            .padding(12)
            .width(Length::Fill)
            .style(secondary_button_style());

        let mut info_col = column![
            text("Information")
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
                    text("Built with iced.rs")
                        .size(12)
                        .style(text_color(Colors::TEXT_MUTED)),
                )
                .push(
                    text("Storage: encrypted backup (ChaCha20-Poly1305 + Argon2id)")
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
