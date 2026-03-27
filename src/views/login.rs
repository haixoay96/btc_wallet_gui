use iced::{
    widget::{button, column, container, text, text_input, row, Space},
    Alignment, Element, Length, Padding,
};
use crate::theme::{Colors, card_style, input_style, primary_button_style, text_color};

#[derive(Debug, Clone)]
pub enum LoginMessage {
    PassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    Submit,
    ToggleMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginMode {
    ExistingWallet,
    NewWallet,
}

pub struct LoginView {
    passphrase: String,
    confirm_passphrase: String,
    mode: LoginMode,
    can_create_new_passphrase: bool,
    error: Option<String>,
}

impl LoginView {
    pub fn new() -> Self {
        Self {
            passphrase: String::new(),
            confirm_passphrase: String::new(),
            mode: LoginMode::ExistingWallet,
            can_create_new_passphrase: true,
            error: None,
        }
    }

    pub fn set_can_create_new_passphrase(&mut self, can_create: bool) {
        self.can_create_new_passphrase = can_create;
        if !can_create {
            self.mode = LoginMode::ExistingWallet;
            self.confirm_passphrase.clear();
        }
    }

    pub fn set_mode(&mut self, mode: LoginMode) {
        self.mode = if !self.can_create_new_passphrase && mode == LoginMode::NewWallet {
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

    pub fn update(&mut self, message: LoginMessage) -> Option<crate::app::AppMessage> {
        match message {
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
            LoginMessage::Submit => {
                if self.passphrase.trim().is_empty() {
                    self.error = Some("Passphrase không được để trống".to_string());
                    return None;
                }

                if self.mode == LoginMode::NewWallet {
                    if self.confirm_passphrase.trim().is_empty() {
                        self.error = Some("Vui lòng xác nhận passphrase".to_string());
                        return None;
                    }

                    if self.passphrase != self.confirm_passphrase {
                        self.error = Some("Passphrase không khớp".to_string());
                        return None;
                    }
                }

                Some(crate::app::AppMessage::Login(self.passphrase.clone()))
            }
            LoginMessage::ToggleMode => {
                if !self.can_create_new_passphrase {
                    return None;
                }

                self.mode = match self.mode {
                    LoginMode::ExistingWallet => LoginMode::NewWallet,
                    LoginMode::NewWallet => LoginMode::ExistingWallet,
                };
                self.confirm_passphrase.clear();
                self.error = None;
                None
            }
        }
    }

    pub fn view(&self) -> Element<'_, LoginMessage> {
        let is_existing_mode = self.mode == LoginMode::ExistingWallet;

        let title = text("Bitcoin Wallet")
            .size(36)
            .style(text_color(Colors::TEXT_PRIMARY));

        let subtitle = text(if !self.can_create_new_passphrase {
            "Đăng nhập bằng passphrase hiện tại"
        } else if is_existing_mode {
            "Đăng nhập bằng passphrase"
        } else {
            "Tạo bộ dữ liệu ví mới bằng passphrase"
        })
            .size(16)
            .style(text_color(Colors::TEXT_SECONDARY));

        let passphrase_input = text_input("Nhập passphrase...", &self.passphrase)
            .on_input(LoginMessage::PassphraseChanged)
            .on_submit(LoginMessage::Submit)
            .secure(true)
            .padding(12)
            .size(16)
            .style(input_style());

        let confirm_input: Element<'_, LoginMessage> = if self.mode == LoginMode::NewWallet {
            text_input("Xác nhận passphrase...", &self.confirm_passphrase)
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

        let error_text = if let Some(error) = &self.error {
            text(error.as_str())
                .style(text_color(Colors::ERROR))
                .size(14)
        } else {
            text("")
        };

        let mut buttons = row![
            button(
                text(if is_existing_mode { "Đăng nhập" } else { "Khởi tạo dữ liệu mới" })
                    .size(16)
            )
            .on_press(LoginMessage::Submit)
            .padding(12)
            .style(primary_button_style()),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        if self.can_create_new_passphrase {
            buttons = buttons.push(Space::with_width(12)).push(
                button(
                    text(if is_existing_mode { "Chuyển sang tạo mới" } else { "Chuyển sang đăng nhập" })
                        .size(16)
                )
                .on_press(LoginMessage::ToggleMode)
                .padding(12)
                .style(primary_button_style()),
            );
        }

        let content = column![
            Space::with_height(40),
            title,
            Space::with_height(8),
            subtitle,
            Space::with_height(40),
            passphrase_input,
            Space::with_height(12),
            confirm_input,
            Space::with_height(16),
            error_text,
            Space::with_height(24),
            buttons,
            Space::with_height(40),
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
