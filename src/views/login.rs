use iced::{
    widget::{button, column, container, row, text, text_input, Space},
    Alignment, Element, Length, Padding,
};

use crate::i18n::{t, AppLanguage};
use crate::theme::{
    card_style, gradient_button_style, input_style, primary_button_style, secondary_button_style,
    text_color, Colors,
};
use crate::views::language_selector::LanguageSelector;

#[derive(Debug, Clone)]
pub enum LoginMessage {
    LanguageChanged(AppLanguage),
    NicknameChanged(String),
    PassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    BrowseBackupPath,
    Submit,
    SetMode(LoginMode),
}

#[derive(Debug, Clone)]
pub enum LoginEvent {
    ChangeLanguage(AppLanguage),
    BrowseBackupPath,
    SubmitExisting { passphrase: String },
    SubmitNew { passphrase: String, nickname: String },
    SubmitImport { backup_path: String, passphrase: String },
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
    language_selector: LanguageSelector,
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
            language_selector: LanguageSelector::new(),
        }
    }

    pub fn set_can_create_new_passphrase(&mut self, can_create: bool) {
        self.can_create_new_passphrase = can_create;
        if can_create {
            // No passphrase exists yet, default to NewWallet mode
            self.mode = LoginMode::NewWallet;
        } else {
            // Passphrase exists, use ExistingWallet mode
            self.mode = LoginMode::ExistingWallet;
        }
        self.nickname.clear();
        self.confirm_passphrase.clear();
        self.backup_path.clear();
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

    pub fn update(&mut self, message: LoginMessage) -> Option<LoginEvent> {
        match message {
            LoginMessage::LanguageChanged(language) => {
                self.error = None;
                Some(LoginEvent::ChangeLanguage(language))
            }
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
            LoginMessage::BrowseBackupPath => Some(LoginEvent::BrowseBackupPath),
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
                    LoginMode::ExistingWallet => Some(LoginEvent::SubmitExisting { passphrase: self.passphrase.clone() }),
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

                        Some(LoginEvent::SubmitNew { passphrase: self.passphrase.clone(), nickname: self.nickname.trim().to_string() })
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

                        Some(LoginEvent::SubmitImport { backup_path: self.backup_path.trim().to_string(), passphrase: self.passphrase.clone() })
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
        let language_picker = self
            .language_selector
            .view(LoginMessage::LanguageChanged);

        // Header bar with language selector on top-right (similar to main view)
        let header_bar = container(
            row![
                Space::with_width(Length::Fill),
                language_picker,
            ]
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(Padding::from([8, 16]));

        let logo = text("₿")
            .size(64)
            .style(text_color(Colors::ACCENT_PURPLE));

        let logo_container = container(logo)
            .center_x(Length::Fill);

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
            // When no passphrase exists, show only NewWallet and ImportBackup options
            row![
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
            // When passphrase exists, show only Login option
            row![
                mode_button(
                    t("Đăng nhập", "Login").to_string(),
                    self.mode == LoginMode::ExistingWallet
                )
                .on_press(LoginMessage::SetMode(LoginMode::ExistingWallet)),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
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
            let mut col = column![
                button(text(t("Chọn file backup", "Choose backup file")).size(14))
                    .on_press(LoginMessage::BrowseBackupPath)
                    .padding(12)
                    .style(secondary_button_style()),
            ]
            .spacing(8)
            .align_x(Alignment::Center);

            // Show file path if selected
            if !self.backup_path.trim().is_empty() {
                col = col.push(
                    text(format!("📄 {}", self.backup_path))
                        .size(13)
                        .style(text_color(Colors::TEXT_SECONDARY))
                );
            }

            col.into()
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

        let login_content = column![
            logo_container,
            Space::with_height(8),
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
                .style(gradient_button_style()),
            Space::with_height(24),
        ]
        .spacing(0)
        .align_x(Alignment::Center);

        let login_container = container(login_content)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(Padding::from(40));

        // Main layout: header bar on top, login content centered below
        let main_layout = column![
            header_bar,
            Space::with_height(Length::Fill),
            login_container,
            Space::with_height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        // Apply card_style to entire login screen
        container(main_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(card_style())
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
