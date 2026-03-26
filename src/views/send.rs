use iced::{
    widget::{button, column, container, row, text, text_input, Space, radio, scrollable},
    Alignment, Element, Length, Padding,
};
use crate::theme::{Colors, card_style, primary_button_style, secondary_button_style, text_color, danger_button_style};
use crate::wallet::{WalletEntry, FeeMode, TxBuildOptions, InputSource, ChangeStrategy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleFeeMode {
    Auto,
    Fixed,
}

#[derive(Debug, Clone)]
pub enum SendMessage {
    ToAddressChanged(String),
    AmountChanged(String),
    FeeModeChanged(SimpleFeeMode),
    FeeAmountChanged(String),
    UseAllFunds(bool),
    FromAddressChanged(String),
    ChangeAddressChanged(String),
    BroadcastChanged(bool),
    EstimateFee,
    Send,
    ClearForm,
}

pub struct SendView {
    to_address: String,
    amount: String,
    fee_mode: SimpleFeeMode,
    fee_amount: String,
    use_all_funds: bool,
    from_address: String,
    change_address: String,
    broadcast: bool,
    estimated_fee: Option<u64>,
    error: Option<String>,
}

impl SendView {
    pub fn new() -> Self {
        Self {
            to_address: String::new(),
            amount: String::new(),
            fee_mode: SimpleFeeMode::Auto,
            fee_amount: String::new(),
            use_all_funds: false,
            from_address: String::new(),
            change_address: String::new(),
            broadcast: false,
            estimated_fee: None,
            error: None,
        }
    }

    pub fn update(&mut self, message: SendMessage) -> Option<crate::app::AppMessage> {
        match message {
            SendMessage::ToAddressChanged(addr) => {
                self.to_address = addr;
                None
            }
            SendMessage::AmountChanged(amount) => {
                self.amount = amount;
                None
            }
            SendMessage::FeeModeChanged(mode) => {
                self.fee_mode = mode;
                None
            }
            SendMessage::FeeAmountChanged(fee) => {
                self.fee_amount = fee;
                None
            }
            SendMessage::UseAllFunds(use_all) => {
                self.use_all_funds = use_all;
                None
            }
            SendMessage::FromAddressChanged(addr) => {
                self.from_address = addr;
                None
            }
            SendMessage::ChangeAddressChanged(addr) => {
                self.change_address = addr;
                None
            }
            SendMessage::BroadcastChanged(broadcast) => {
                self.broadcast = broadcast;
                None
            }
            SendMessage::EstimateFee => {
                // TODO: Implement fee estimation
                None
            }
            SendMessage::Send => {
                if self.to_address.trim().is_empty() {
                    self.error = Some("Please enter recipient address".to_string());
                    return None;
                }
                if self.amount.trim().is_empty() && !self.use_all_funds {
                    self.error = Some("Please enter amount".to_string());
                    return None;
                }
                self.error = None;
                // TODO: Implement actual send
                None
            }
            SendMessage::ClearForm => {
                self.to_address.clear();
                self.amount.clear();
                self.fee_amount.clear();
                self.from_address.clear();
                self.change_address.clear();
                self.broadcast = false;
                self.use_all_funds = false;
                self.error = None;
                self.estimated_fee = None;
                None
            }
        }
    }

    pub fn view(&self, wallet: Option<&WalletEntry>) -> Element<SendMessage> {
        let title = text("Send BTC")
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let balance_text = if let Some(wallet) = wallet {
            let balance: i64 = wallet.history.iter().map(|tx| tx.amount_sat).sum();
            let balance_btc = balance as f64 / 100_000_000.0;
            text(format!("Available: {:.8} BTC", balance_btc))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY))
        } else {
            text("No wallet selected")
                .size(14)
                .style(text_color(Colors::ERROR))
        };

        let to_input = column![
            text("To Address").size(14).style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            text_input("Enter recipient address...", &self.to_address)
                .on_input(SendMessage::ToAddressChanged)
                .padding(12)
                .size(14)
        ].spacing(4);

        let use_all_toggle = row![
            text("Send All Funds").size(14).style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_width(Length::Fill),
            button(
                text(if self.use_all_funds { "✓ ON" } else { "OFF" }).size(12)
            )
            .on_press(SendMessage::UseAllFunds(!self.use_all_funds))
            .padding(6)
            .style(if self.use_all_funds { primary_button_style() } else { secondary_button_style() })
        ].align_y(Alignment::Center);

        let amount_input = column![
            text("Amount (sat)").size(14).style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            text_input("Enter amount in satoshis...", &self.amount)
                .on_input(SendMessage::AmountChanged)
                .padding(12)
                .size(14)
        ].spacing(4);

        let fee_mode_section = column![
            text("Fee Mode").size(14).style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            row![
                radio("Auto", SimpleFeeMode::Auto, Some(self.fee_mode), SendMessage::FeeModeChanged)
                    .size(14),
                Space::with_width(16),
                radio("Fixed", SimpleFeeMode::Fixed, Some(self.fee_mode), SendMessage::FeeModeChanged)
                    .size(14),
            ].spacing(8)
        ].spacing(4);

        let fee_input = if self.fee_mode == SimpleFeeMode::Fixed {
            column![
                text("Fee Amount (sat)").size(14).style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Enter fee in satoshis...", &self.fee_amount)
                    .on_input(SendMessage::FeeAmountChanged)
                    .padding(12)
                    .size(14)
            ].spacing(4)
        } else {
            column![
                if let Some(fee) = self.estimated_fee {
                    text(format!("Estimated fee: {} sat", fee))
                        .size(14)
                        .style(text_color(Colors::SUCCESS))
                } else {
                    text("Click 'Estimate Fee' to calculate")
                        .size(14)
                        .style(text_color(Colors::TEXT_MUTED))
                }
            ]
        };

        let estimate_btn = button(text("Estimate Fee").size(14))
            .on_press(SendMessage::EstimateFee)
            .padding(10)
            .style(secondary_button_style());

        let advanced_section = column![
            text("Advanced Options (Optional)").size(16).style(text_color(Colors::TEXT_PRIMARY)),
            Space::with_height(8),
            column![
                text("From Address (leave empty for all)").size(12).style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Specific address to spend from...", &self.from_address)
                    .on_input(SendMessage::FromAddressChanged)
                    .padding(10)
                    .size(12)
            ].spacing(2),
            Space::with_height(8),
            column![
                text("Change Address (leave empty for new)").size(12).style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Address to receive change...", &self.change_address)
                    .on_input(SendMessage::ChangeAddressChanged)
                    .padding(10)
                    .size(12)
            ].spacing(2),
            Space::with_height(8),
            row![
                text("Broadcast Immediately").size(14).style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_width(Length::Fill),
                button(
                    text(if self.broadcast { "✓ YES" } else { "NO" }).size(12)
                )
                .on_press(SendMessage::BroadcastChanged(!self.broadcast))
                .padding(6)
                .style(if self.broadcast { primary_button_style() } else { secondary_button_style() })
            ].align_y(Alignment::Center)
        ].spacing(8);

        let error_text = if let Some(error) = &self.error {
            text(error.as_str())
                .size(14)
                .style(text_color(Colors::ERROR))
        } else {
            text("")
        };

        let send_btn = button(text("Send Transaction").size(16))
            .on_press(SendMessage::Send)
            .padding(14)
            .width(Length::Fill)
            .style(primary_button_style());

        let clear_btn = button(text("Clear Form").size(14))
            .on_press(SendMessage::ClearForm)
            .padding(10)
            .style(secondary_button_style());

        let content = column![
            title,
            Space::with_height(8),
            balance_text,
            Space::with_height(24),
            to_input,
            Space::with_height(16),
            use_all_toggle,
            Space::with_height(16),
            amount_input,
            Space::with_height(16),
            fee_mode_section,
            Space::with_height(8),
            fee_input,
            Space::with_height(8),
            estimate_btn,
            Space::with_height(24),
            container(advanced_section)
                .style(card_style())
                .padding(16),
            Space::with_height(24),
            error_text,
            Space::with_height(16),
            row![send_btn, Space::with_width(16), clear_btn]
                .width(Length::Fill),
        ]
        .spacing(8)
        .padding(32);

        scrollable(
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .into()
    }
}