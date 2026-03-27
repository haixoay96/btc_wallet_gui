use iced::{
    widget::{button, column, container, scrollable, text, text_input, Space},
    Element, Length,
};

use crate::theme::{card_style, primary_button_style, secondary_button_style, text_color, Colors};

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    ToggleChangePassphrase,
    CurrentPassphraseChanged(String),
    NewPassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    SubmitPassphraseChange,
    ExportPathChanged(String),
    ImportPathChanged(String),
    ExportWallet,
    ImportWallet,
    ToggleAbout,
}

pub struct SettingsView {
    show_change_passphrase: bool,
    current_passphrase: String,
    new_passphrase: String,
    confirm_passphrase: String,
    export_path: String,
    import_path: String,
    show_about: bool,
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
            export_path: "./wallet_backup.enc".to_string(),
            import_path: "./wallet_backup.enc".to_string(),
            show_about: false,
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
            SettingsMessage::ExportPathChanged(path) => {
                self.export_path = path;
                None
            }
            SettingsMessage::ImportPathChanged(path) => {
                self.import_path = path;
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
            SettingsMessage::ImportWallet => {
                let path = self.import_path.trim();
                if path.is_empty() {
                    self.error = Some("Vui lòng nhập đường dẫn import".to_string());
                    return None;
                }

                self.error = None;
                self.success = None;
                Some(crate::app::AppMessage::ImportWalletBackup(path.to_string()))
            }
            SettingsMessage::ToggleAbout => {
                self.show_about = !self.show_about;
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
            container(
                column![
                    text("Security")
                        .size(18)
                        .style(text_color(Colors::TEXT_PRIMARY)),
                    Space::with_height(12),
                    change_passphrase_btn,
                ],
            )
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
                    .padding(10)
                    .size(14)
            ]
            .spacing(2);

            content = content.push(
                container(
                    column![
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
                    ],
                )
                .style(card_style())
                .padding(16)
                .width(Length::Fill),
            );
        }

        let export_section = container(
            column![
                text("Export Backup")
                    .size(18)
                    .style(text_color(Colors::TEXT_PRIMARY)),
                Space::with_height(8),
                text("Backup sẽ được mã hóa bằng passphrase hiện tại")
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
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
            ],
        )
        .style(card_style())
        .padding(16)
        .width(Length::Fill);

        content = content.push(export_section);

        let import_section = container(
            column![
                text("Import Backup")
                    .size(18)
                    .style(text_color(Colors::TEXT_PRIMARY)),
                Space::with_height(8),
                text("Import sẽ ghi đè danh sách wallet hiện tại")
                    .size(12)
                    .style(text_color(Colors::WARNING)),
                Space::with_height(8),
                text_input("Path to backup file...", &self.import_path)
                    .on_input(SettingsMessage::ImportPathChanged)
                    .padding(10)
                    .size(14),
                Space::with_height(8),
                button(text("Import Wallet Backup").size(14))
                    .on_press(SettingsMessage::ImportWallet)
                    .padding(12)
                    .style(secondary_button_style()),
            ],
        )
        .style(card_style())
        .padding(16)
        .width(Length::Fill);

        content = content.push(import_section);

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
                .push(text("Bitcoin Wallet GUI v0.1.0").size(12).style(text_color(Colors::TEXT_MUTED)))
                .push(text("Built with iced.rs").size(12).style(text_color(Colors::TEXT_MUTED)))
                .push(text("Storage: encrypted backup (ChaCha20-Poly1305 + Argon2id)").size(12).style(text_color(Colors::TEXT_MUTED)));
        }

        content = content.push(
            container(info_col)
                .style(card_style())
                .padding(16)
                .width(Length::Fill),
        );

        if let Some(err) = &self.error {
            content = content.push(
                text(err.as_str())
                    .size(13)
                    .style(text_color(Colors::ERROR)),
            );
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
