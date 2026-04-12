use iced::{
    widget::{button, column, container, row, text, text_input, Space},
    Alignment, Element, Length, Padding,
};

use crate::ui::components::language_selector::LanguageSelector;
use crate::ui::components::{calculate_strength, strength_bar};
use crate::ui::i18n::t;
use crate::ui::theme::{
    input_style, muted_button_style, notice_style, primary_button_style, screen_background_style,
    secondary_button_style, selected_button_style, text_color, text_muted_color,
    text_primary_color, text_scaled, text_secondary_color, Colors, NoticeTone,
};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

use super::structure::*;

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
            show_passphrase: false,
            show_confirm_passphrase: false,
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

    pub fn clear_sensitive_inputs(&mut self) {
        self.passphrase.clear();
        self.confirm_passphrase.clear();
        self.show_passphrase = false;
        self.show_confirm_passphrase = false;
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
                    LoginMode::ExistingWallet => Some(LoginEvent::SubmitExisting {
                        passphrase: self.passphrase.clone(),
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

                        Some(LoginEvent::SubmitNew {
                            passphrase: self.passphrase.clone(),
                            nickname: self.nickname.trim().to_string(),
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

                        Some(LoginEvent::SubmitImport {
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
                    self.show_confirm_passphrase = false;
                }
                if self.mode != LoginMode::ImportBackup {
                    self.backup_path.clear();
                }
                if self.mode == LoginMode::ExistingWallet {
                    self.nickname.clear();
                }
                self.show_passphrase = false;
                self.error = None;
                None
            }
            LoginMessage::TogglePassphraseVisibility => {
                self.show_passphrase = !self.show_passphrase;
                None
            }
            LoginMessage::ToggleConfirmPassphraseVisibility => {
                self.show_confirm_passphrase = !self.show_confirm_passphrase;
                None
            }
        }
    }

    pub fn view(&self) -> Element<'_, LoginMessage> {
        let language_picker = self.language_selector.view(LoginMessage::LanguageChanged);

        let header_bar = container(
            row![Space::with_width(Length::Fill), language_picker,].align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(Padding::from([16, 24]));

        let logo = text_scaled("₿", 64).style(text_color(Colors::ACCENT_PURPLE));

        let logo_container = container(logo).center_x(Length::Fill);

        let title = container(
            text_scaled(t("Mở Ví Bitcoin", "Open Bitcoin Wallet"), 36).style(text_primary_color()),
        )
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center);

        let subtitle = container(
            text_scaled(
                match self.mode {
                    LoginMode::ExistingWallet => t(
                        "Đăng nhập để truy cập dữ liệu ví đã lưu trên máy này.",
                        "Log in to access wallet data already stored on this device.",
                    ),
                    LoginMode::NewWallet => t(
                        "Tạo bộ dữ liệu ví mới và đặt passphrase bảo vệ ứng dụng.",
                        "Create fresh wallet data and protect the app with a passphrase.",
                    ),
                    LoginMode::ImportBackup => t(
                        "Khôi phục toàn bộ dữ liệu ứng dụng từ file backup mã hóa.",
                        "Restore the full app data from an encrypted backup file.",
                    ),
                },
                16,
            )
            .style(text_secondary_color()),
        )
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center);

        let mode_switcher_content: Element<'_, LoginMessage> = if self.can_create_new_passphrase {
            container(
                row![
                    mode_button(
                        t("Tạo passphrase mới", "Create new passphrase").to_string(),
                        self.mode == LoginMode::NewWallet,
                        true,
                    )
                    .on_press(LoginMessage::SetMode(LoginMode::NewWallet)),
                    mode_button(
                        t("Import backup", "Import backup").to_string(),
                        self.mode == LoginMode::ImportBackup,
                        true,
                    )
                    .on_press(LoginMessage::SetMode(LoginMode::ImportBackup)),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .into()
        } else {
            column![
                row![
                    mode_button(
                        t("Đăng nhập", "Login").to_string(),
                        true,
                        true,
                    ),
                    mode_button(
                        t("Tạo dữ liệu mới", "Create new data").to_string(),
                        false,
                        false,
                    ),
                    mode_button(
                        t("Import app backup", "Import app backup").to_string(),
                        false,
                        false,
                    ),
                ]
                .spacing(10),
                text_scaled(t(
                    "Để khôi phục thêm ví riêng lẻ, vào Wallets > Import sau khi đăng nhập.",
                    "To restore additional individual wallets, use Wallets > Import after login.",
                ), 12)
                .style(text_secondary_color()),
            ]
            .spacing(10)
            .align_x(Alignment::Center)
            .into()
        };

        let input_width = Length::Fixed(570.0);

        let nickname_input: Element<'_, LoginMessage> = if self.mode == LoginMode::NewWallet {
            column![
                text_scaled(t("Tên hiển thị", "Display name"), 12).style(text_secondary_color()),
                Space::with_height(4),
                text_input(t("Nhập nickname...", "Enter nickname..."), &self.nickname)
                    .on_input(LoginMessage::NicknameChanged)
                    .padding(12)
                    .size(16)
                    .style(input_style()),
            ]
            .spacing(0)
            .width(input_width)
            .into()
        } else {
            Space::with_height(0).into()
        };

        let passphrase_input = column![
            text_scaled(t("Passphrase", "Passphrase"), 12).style(text_secondary_color()),
            Space::with_height(4),
            row![
                text_input(
                    t("Nhập passphrase...", "Enter passphrase..."),
                    &self.passphrase,
                )
                .on_input(LoginMessage::PassphraseChanged)
                .on_submit(LoginMessage::Submit)
                .secure(!self.show_passphrase)
                .padding(12)
                .size(16)
                .style(input_style()),
                button(
                    text_scaled(
                        if self.show_passphrase {
                            Bootstrap::EyeSlash.to_string()
                        } else {
                            Bootstrap::Eye.to_string()
                        },
                        16
                    )
                    .font(BOOTSTRAP_FONT)
                    .style(text_muted_color()),
                )
                .on_press(LoginMessage::TogglePassphraseVisibility)
                .padding(10)
                .style(secondary_button_style()),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .width(Length::Fill),
        ]
        .spacing(0)
        .width(input_width);

        let confirm_input: Element<'_, LoginMessage> = if self.mode == LoginMode::NewWallet {
            let strength = calculate_strength(&self.passphrase);
            let strength_widget = strength_bar(strength, true).map(|_| LoginMessage::Submit);

            column![
                text_scaled(t("Xác nhận passphrase", "Confirm passphrase"), 12)
                    .style(text_secondary_color()),
                Space::with_height(4),
                row![
                    text_input(
                        t("Xác nhận passphrase...", "Confirm passphrase..."),
                        &self.confirm_passphrase,
                    )
                    .on_input(LoginMessage::ConfirmPassphraseChanged)
                    .on_submit(LoginMessage::Submit)
                    .secure(!self.show_confirm_passphrase)
                    .padding(12)
                    .size(16)
                    .style(input_style()),
                    button(
                        text_scaled(
                            if self.show_confirm_passphrase {
                                Bootstrap::EyeSlash.to_string()
                            } else {
                                Bootstrap::Eye.to_string()
                            },
                            16
                        )
                        .font(BOOTSTRAP_FONT)
                        .style(text_muted_color()),
                    )
                    .on_press(LoginMessage::ToggleConfirmPassphraseVisibility)
                    .padding(10)
                    .style(secondary_button_style()),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .width(Length::Fill),
                Space::with_height(8),
                strength_widget,
            ]
            .spacing(0)
            .width(input_width)
            .into()
        } else {
            Space::with_height(0).into()
        };

        let backup_path_input: Element<'_, LoginMessage> = if self.mode == LoginMode::ImportBackup {
            let mut col = column![
                text_scaled(t("File backup ứng dụng", "App backup file"), 12)
                    .style(text_secondary_color()),
                Space::with_height(4),
                button(text_scaled(t("Chọn file backup", "Choose backup file"), 14))
                    .on_press(LoginMessage::BrowseBackupPath)
                    .padding(12)
                    .style(secondary_button_style()),
            ]
            .spacing(8)
            .width(input_width);

            if !self.backup_path.trim().is_empty() {
                col = col
                    .push(text_scaled(self.backup_path.as_str(), 13).style(text_primary_color()));
            }

            col.into()
        } else {
            Space::with_height(0).into()
        };

        let action_label = match self.mode {
            LoginMode::ExistingWallet => t("Đăng nhập", "Login"),
            LoginMode::NewWallet => t("Tạo dữ liệu mới", "Create new data"),
            LoginMode::ImportBackup => t("Khôi phục từ backup", "Restore from backup"),
        };

        let error_text: Element<'_, LoginMessage> = if let Some(error) = &self.error {
            container(
                container(text(error.as_str()).style(text_primary_color()).size(14))
                    .style(notice_style(NoticeTone::Error))
                    .padding(12),
            )
            .width(input_width)
            .into()
        } else {
            Space::with_height(0).into()
        };

        let input_section = column![
            mode_switcher_content,
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
            Space::with_height(20),
            button(
                container(text_scaled(action_label, 16))
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .on_press(LoginMessage::Submit)
            .padding(12)
            .width(Length::Fill)
            .style(primary_button_style()),
        ]
        .spacing(0)
        .width(input_width)
        .align_x(iced::alignment::Horizontal::Center);

        let login_content = column![
            logo_container,
            Space::with_height(4),
            title,
            Space::with_height(8),
            subtitle,
            Space::with_height(20),
            container(input_section)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center),
            Space::with_height(4),
        ]
        .spacing(0)
        .width(Length::Fill);

        let login_container = container(login_content)
            .width(Length::Fixed(700.0))
            .center_x(Length::Fill)
            .padding(Padding::from(28));

        let main_layout = column![
            header_bar,
            Space::with_height(Length::Fill),
            login_container,
            Space::with_height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        container(main_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(screen_background_style())
            .into()
    }
}

fn mode_button(
    label: String,
    active: bool,
    enabled: bool,
) -> iced::widget::Button<'static, LoginMessage> {
    button(text_scaled(label, 13))
        .padding(10)
        .style(if !enabled {
            muted_button_style()
        } else if active {
            selected_button_style()
        } else {
            secondary_button_style()
        })
}
