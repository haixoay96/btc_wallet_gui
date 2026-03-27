use iced::{
    widget::{button, column, container, row, text, text_input, Space},
    Alignment, Element, Length, Padding,
};

use crate::i18n::t;
use crate::theme::{
    card_style, input_style, primary_button_style, secondary_button_style, text_color, Colors,
};

#[derive(Debug, Clone)]
pub enum LoginMessage {
    NicknameChanged(String),
    PassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    BackupPathChanged(String),
    BrowseBackupPath,
    Submit,
    SetMode(LoginMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginMode {
    ExistingWallet,
    NewWallet,
    ImportBackup,
}

pub struct LoginView {
    nickname: String,
    passphrase: String,
    confirm_passphrase: String,
    backup_path: String,
    mode: LoginMode,
    can_create_new_passphrase: bool,
    error: Option<String>,
}

impl LoginView {
    pub fn new() -> Self {
        Self {
            nickname: String::new(),
            passphrase: String::new(),
            confirm_passphrase: String::new(),
            backup_path: String::new(),
            mode: LoginMode::ExistingWallet,
            can_create_new_passphrase: true,
            error: None,
        }
    }

    pub fn set_can_create_new_passphrase(&mut self, can_create: bool) {
        self.can_create_new_passphrase = can_create;
        if !can_create {
            self.mode = LoginMode::ExistingWallet;
            self.nickname.clear();
            self.confirm_passphrase.clear();
            self.backup_path.clear();
        }
    }

    pub fn set_mode(&mut self, mode: LoginMode) {
        self.mode = if !self.can_create_new_passphrase && mode != LoginMode::ExistingWallet {
            LoginMode::ExistingWallet
        } else {
            mode
        };
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error = Some(message.into());
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn set_backup_path(&mut self, path: String) {
        self.backup_path = path;
        self.error = None;
    }

    pub fn update(&mut self, message: LoginMessage) -> Option<crate::app::AppMessage> {
        match message {
            LoginMessage::NicknameChanged(value) => {
                self.nickname = value;
                self.error = None;
                None
            }
            LoginMessage::PassphraseChanged(value) => {
                self.passphrase = value;
                self.error = None;
                None
            }
            LoginMessage::ConfirmPassphraseChanged(value) => {
                self.confirm_passphrase = value;
                self.error = None;
                None
            }
            LoginMessage::BackupPathChanged(value) => {
                self.backup_path = value;
                self.error = None;
                None
            }
            LoginMessage::BrowseBackupPath => Some(crate::app::AppMessage::PickImportBackupPath),
            LoginMessage::Submit => {
                if self.passphrase.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Passphrase không được để trống",
                            "Passphrase cannot be empty",
                        )
                        .to_string(),
                    );
                    return None;
                }

                match self.mode {
                    LoginMode::ExistingWallet => Some(crate::app::AppMessage::Login {
                        passphrase: self.passphrase.clone(),
                        nickname: None,
                        creating_new: false,
                    }),
                    LoginMode::NewWallet => {
                        if self.nickname.trim().is_empty() {
                            self.error = Some(
                                t("Vui lòng nhập nickname", "Please enter nickname").to_string(),
                            );
                            return None;
                        }

                        if self.confirm_passphrase.trim().is_empty() {
                            self.error = Some(
                                t("Vui lòng xác nhận passphrase", "Please confirm passphrase")
                                    .to_string(),
                            );
                            return None;
                        }

                        if self.passphrase != self.confirm_passphrase {
                            self.error = Some(
                                t("Passphrase không khớp", "Passphrases do not match").to_string(),
                            );
                            return None;
                        }

                        Some(crate::app::AppMessage::Login {
                            passphrase: self.passphrase.clone(),
                            nickname: Some(self.nickname.trim().to_string()),
                            creating_new: true,
                        })
                    }
                    LoginMode::ImportBackup => {
                        if self.backup_path.trim().is_empty() {
                            self.error = Some(
                                t(
                                    "Vui lòng nhập đường dẫn file backup",
                                    "Please enter backup file path",
                                )
                                .to_string(),
                            );
                            return None;
                        }

                        Some(crate::app::AppMessage::InitialImportBackup {
                            backup_path: self.backup_path.trim().to_string(),
                            passphrase: self.passphrase.clone(),
                        })
                    }
                }
            }
            LoginMessage::SetMode(mode) => {
                self.set_mode(mode);
                if self.mode != LoginMode::NewWallet {
                    self.confirm_passphrase.clear();
                }
                if self.mode != LoginMode::ImportBackup {
                    self.backup_path.clear();
                }
                if self.mode == LoginMode::ExistingWallet {
                    self.nickname.clear();
                }
                self.error = None;
                None
            }
        }
    }

    pub fn view(&self) -> Element<'_, LoginMessage> {
        let title = text(t("Ví Bitcoin", "Bitcoin Wallet"))
            .size(36)
            .style(text_color(Colors::TEXT_PRIMARY));

        let subtitle = text(match self.mode {
            LoginMode::ExistingWallet => t("Đăng nhập bằng passphrase", "Login with passphrase"),
            LoginMode::NewWallet => t(
                "Tạo bộ dữ liệu ví mới bằng passphrase",
                "Create new wallet data with passphrase",
            ),
            LoginMode::ImportBackup => t(
                "Import backup khi app chưa có dữ liệu, sau đó đăng nhập bằng passphrase backup",
                "Import backup when app has no data yet, then login with backup passphrase",
            ),
        })
        .size(16)
        .style(text_color(Colors::TEXT_SECONDARY));

        let mode_switcher: Element<'_, LoginMessage> = if self.can_create_new_passphrase {
            row![
                mode_button(
                    t("Đăng nhập", "Login").to_string(),
                    self.mode == LoginMode::ExistingWallet
                )
                .on_press(LoginMessage::SetMode(LoginMode::ExistingWallet)),
                mode_button(
                    t("Tạo passphrase mới", "Create new passphrase").to_string(),
                    self.mode == LoginMode::NewWallet
                )
                .on_press(LoginMessage::SetMode(LoginMode::NewWallet)),
                mode_button(
                    t("Import backup", "Import backup").to_string(),
                    self.mode == LoginMode::ImportBackup
                )
                .on_press(LoginMessage::SetMode(LoginMode::ImportBackup)),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
        } else {
            Space::with_height(0).into()
        };

        let nickname_input: Element<'_, LoginMessage> = if self.mode == LoginMode::NewWallet {
            text_input(t("Nhập nickname...", "Enter nickname..."), &self.nickname)
                .on_input(LoginMessage::NicknameChanged)
                .padding(12)
                .size(16)
                .style(input_style())
                .into()
        } else {
            Space::with_height(0).into()
        };

        let passphrase_input = text_input(
            t("Nhập passphrase...", "Enter passphrase..."),
            &self.passphrase,
        )
        .on_input(LoginMessage::PassphraseChanged)
        .on_submit(LoginMessage::Submit)
        .secure(true)
        .padding(12)
        .size(16)
        .style(input_style());

        let confirm_input: Element<'_, LoginMessage> = if self.mode == LoginMode::NewWallet {
            text_input(
                t("Xác nhận passphrase...", "Confirm passphrase..."),
                &self.confirm_passphrase,
            )
            .on_input(LoginMessage::ConfirmPassphraseChanged)
            .on_submit(LoginMessage::Submit)
            .secure(true)
            .padding(12)
            .size(16)
            .style(input_style())
            .into()
        } else {
            Space::with_height(0).into()
        };

        let backup_path_input: Element<'_, LoginMessage> = if self.mode == LoginMode::ImportBackup {
            column![
                text(t("Đường dẫn file backup", "Backup File Path"))
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                row![
                    text_input(
                        t(
                            "Ví dụ: ~/Downloads/wallet_backup.enc",
                            "Example: ~/Downloads/wallet_backup.enc"
                        ),
                        &self.backup_path
                    )
                    .on_input(LoginMessage::BackupPathChanged)
                    .on_submit(LoginMessage::Submit)
                    .padding(12)
                    .size(16)
                    .style(input_style())
                    .width(Length::Fill),
                    Space::with_width(8),
                    button(text(t("Chọn file", "Choose file")).size(14))
                        .on_press(LoginMessage::BrowseBackupPath)
                        .padding(12)
                        .style(secondary_button_style()),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(2)
            .into()
        } else {
            Space::with_height(0).into()
        };

        let action_label = match self.mode {
            LoginMode::ExistingWallet => t("Đăng nhập", "Login"),
            LoginMode::NewWallet => t("Khởi tạo dữ liệu mới", "Initialize new data"),
            LoginMode::ImportBackup => t("Import backup và đăng nhập", "Import backup and login"),
        };

        let error_text = if let Some(error) = &self.error {
            text(error.as_str())
                .style(text_color(Colors::ERROR))
                .size(14)
        } else {
            text("")
        };

        let content = column![
            Space::with_height(24),
            title,
            Space::with_height(8),
            subtitle,
            Space::with_height(20),
            mode_switcher,
            Space::with_height(20),
            nickname_input,
            Space::with_height(12),
            passphrase_input,
            Space::with_height(12),
            confirm_input,
            Space::with_height(12),
            backup_path_input,
            Space::with_height(16),
            error_text,
            Space::with_height(24),
            button(text(action_label).size(16))
                .on_press(LoginMessage::Submit)
                .padding(12)
                .style(primary_button_style()),
            Space::with_height(24),
        ]
        .spacing(0)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(card_style())
            .padding(Padding::from(40))
            .into()
    }
}

fn mode_button(label: String, active: bool) -> iced::widget::Button<'static, LoginMessage> {
    button(text(label).size(13)).padding(10).style(if active {
        primary_button_style()
    } else {
        secondary_button_style()
    })
}
