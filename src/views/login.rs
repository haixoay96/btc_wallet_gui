use iced::{
    widget::{button, column, container, text, text_input, row, Space},
    Alignment, Element, Length, Padding,
};
use crate::theme::{Colors, card_style, input_style, primary_button_style, text_color};

#[derive(Debug, Clone)]
pub enum LoginMessage {
    PassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    Login,
    CreateWallet,
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
    error: Option<String>,
}

impl LoginView {
    pub fn new() -> Self {
        Self {
            passphrase: String::new(),
            confirm_passphrase: String::new(),
            mode: LoginMode::ExistingWallet,
            error: None,
        }
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
            LoginMessage::Login => {
                if self.passphrase.trim().is_empty() {
                    self.error = Some("Passphrase không được để trống".to_string());
                    return None;
                }
                Some(crate::app::AppMessage::Login(self.passphrase.clone()))
            }
            LoginMessage::CreateWallet => {
                if self.passphrase.trim().is_empty() {
                    self.error = Some("Passphrase không được để trống".to_string());
                    return None;
                }
                if self.passphrase != self.confirm_passphrase {
                    self.error = Some("Passphrase không khớp".to_string());
                    return None;
                }
                self.mode = LoginMode::ExistingWallet;
                self.confirm_passphrase.clear();
                Some(crate::app::AppMessage::Login(self.passphrase.clone()))
            }
        }
    }

    pub fn view(&self) -> Element<LoginMessage> {
        let title = text("Bitcoin Wallet")
            .size(36)
            .style(text_color(Colors::TEXT_PRIMARY));

        let subtitle = text("Exodus-style GUI with iced.rs")
            .size(16)
            .style(text_color(Colors::TEXT_SECONDARY));

        let passphrase_input = text_input("Nhập passphrase...", &self.passphrase)
            .on_input(LoginMessage::PassphraseChanged)
            .on_submit(LoginMessage::Login)
            .padding(12)
            .size(16)
            .style(input_style());

        let confirm_input = if self.mode == LoginMode::NewWallet {
            text_input("Xác nhận passphrase...", &self.confirm_passphrase)
                .on_input(LoginMessage::ConfirmPassphraseChanged)
                .on_submit(LoginMessage::CreateWallet)
                .padding(12)
                .size(16)
                .style(input_style())
        } else {
            text_input("", "")
                .padding(12)
                .size(16)
                .style(input_style())
        };

        let error_text = if let Some(error) = &self.error {
            text(error.as_str())
                .style(text_color(Colors::ERROR))
                .size(14)
        } else {
            text("")
        };

        let buttons = row![
            button(
                text(if self.mode == LoginMode::ExistingWallet { "Đăng nhập" } else { "Tạo ví mới" })
                    .size(16)
            )
            .on_press(if self.mode == LoginMode::ExistingWallet { LoginMessage::Login } else { LoginMessage::CreateWallet })
            .padding(12)
            .style(primary_button_style()),
            Space::with_width(12),
            button(
                text(if self.mode == LoginMode::ExistingWallet { "Tạo ví mới" } else { "Đăng nhập" })
                    .size(16)
            )
            .on_press(if self.mode == LoginMode::ExistingWallet { LoginMessage::CreateWallet } else { LoginMessage::Login })
            .padding(12)
            .style(primary_button_style())
        ]
        .spacing(12)
        .align_y(Alignment::Center);

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