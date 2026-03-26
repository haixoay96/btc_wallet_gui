use iced::{
    widget::{button, column, container, row, text, text_input, Space},
    Alignment, Element, Length,
};
use crate::theme::{Colors, card_style, primary_button_style, secondary_button_style, text_color, danger_button_style};

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    ChangePassphrase,
    CurrentPassphraseChanged(String),
    NewPassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    ExportWallet,
    ImportWallet,
    ShowAbout,
}

pub struct SettingsView {
    show_change_passphrase: bool,
    current_passphrase: String,
    new_passphrase: String,
    confirm_passphrase: String,
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
            error: None,
            success: None,
        }
    }

    pub fn update(&mut self, message: SettingsMessage) -> Option<crate::app::AppMessage> {
        match message {
            SettingsMessage::ChangePassphrase => {
                self.show_change_passphrase = !self.show_change_passphrase;
                self.error = None;
                self.success = None;
                None
            }
            SettingsMessage::CurrentPassphraseChanged(p) => {
                self.current_passphrase = p;
                None
            }
            SettingsMessage::NewPassphraseChanged(p) => {
                self.new_passphrase = p;
                None
            }
            SettingsMessage::ConfirmPassphraseChanged(p) => {
                self.confirm_passphrase = p;
                None
            }
            SettingsMessage::ExportWallet => {
                None
            }
            SettingsMessage::ImportWallet => {
                None
            }
            SettingsMessage::ShowAbout => {
                None
            }
        }
    }

    pub fn view(&self) -> Element<SettingsMessage> {
        let title = text("Settings")
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let mut content = column![title].spacing(20).padding(32);

        // Change Passphrase Section
        let change_passphrase_btn = button(text("🔐 Change Passphrase").size(16))
            .on_press(SettingsMessage::ChangePassphrase)
            .padding(12)
            .width(Length::Fill)
            .style(secondary_button_style());

        content = content.push(
            container(
                column![
                    text("Security").size(18).style(text_color(Colors::TEXT_PRIMARY)),
                    Space::with_height(12),
                    change_passphrase_btn,
                ]
            )
            .style(card_style())
            .padding(16)
            .width(Length::Fill)
        );

        if self.show_change_passphrase {
            let current_input = column![
                text("Current Passphrase").size(12).style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Enter current passphrase...", &self.current_passphrase)
                    .on_input(SettingsMessage::CurrentPassphraseChanged)
                    .padding(10)
                    .size(14)
            ].spacing(2);

            let new_input = column![
                text("New Passphrase").size(12).style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Enter new passphrase...", &self.new_passphrase)
                    .on_input(SettingsMessage::NewPassphraseChanged)
                    .padding(10)
                    .size(14)
            ].spacing(2);

            let confirm_input = column![
                text("Confirm New Passphrase").size(12).style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Confirm new passphrase...", &self.confirm_passphrase)
                    .on_input(SettingsMessage::ConfirmPassphraseChanged)
                    .padding(10)
                    .size(14)
            ].spacing(2);

            let error_text = if let Some(err) = &self.error {
                text(err.as_str()).size(12).style(text_color(Colors::ERROR))
            } else {
                text("")
            };

            let success_text = if let Some(succ) = &self.success {
                text(succ.as_str()).size(12).style(text_color(Colors::SUCCESS))
            } else {
                text("")
            };

            content = content.push(
                container(
                    column![
                        current_input,
                        Space::with_height(12),
                        new_input,
                        Space::with_height(12),
                        confirm_input,
                        Space::with_height(16),
                        error_text,
                        success_text,
                        Space::with_height(12),
                        button(text("Update Passphrase").size(14))
                            .padding(12)
                            .style(primary_button_style()),
                    ]
                )
                .style(card_style())
                .padding(16)
                .width(Length::Fill)
            );
        }

        // Wallet Management Section
        let export_btn = button(text("📤 Export Wallet").size(16))
            .on_press(SettingsMessage::ExportWallet)
            .padding(12)
            .width(Length::Fill)
            .style(secondary_button_style());

        let import_btn = button(text("📥 Import Wallet").size(16))
            .on_press(SettingsMessage::ImportWallet)
            .padding(12)
            .width(Length::Fill)
            .style(secondary_button_style());

        content = content.push(
            container(
                column![
                    text("Wallet Management").size(18).style(text_color(Colors::TEXT_PRIMARY)),
                    Space::with_height(12),
                    export_btn,
                    Space::with_height(8),
                    import_btn,
                ]
            )
            .style(card_style())
            .padding(16)
            .width(Length::Fill)
        );

        // About Section
        let about_btn = button(text("ℹ️ About").size(16))
            .on_press(SettingsMessage::ShowAbout)
            .padding(12)
            .width(Length::Fill)
            .style(secondary_button_style());

        content = content.push(
            container(
                column![
                    text("Information").size(18).style(text_color(Colors::TEXT_PRIMARY)),
                    Space::with_height(12),
                    about_btn,
                    Space::with_height(8),
                    text("Bitcoin Wallet GUI v0.1.0").size(12).style(text_color(Colors::TEXT_MUTED)),
                    text("Built with iced.rs").size(12).style(text_color(Colors::TEXT_MUTED)),
                    text("Exodus-style design").size(12).style(text_color(Colors::TEXT_MUTED)),
                ]
            )
            .style(card_style())
            .padding(16)
            .width(Length::Fill)
        );

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}