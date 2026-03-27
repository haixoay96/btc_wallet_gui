use iced::{
    widget::{button, column, container, radio, row, scrollable, text, text_input, Space},
    Alignment, Element, Length,
};

use crate::theme::{card_style, primary_button_style, secondary_button_style, text_color, Colors};
use crate::wallet::{ChangeStrategy, FeeMode, InputSource, WalletEntry};

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
    success: Option<String>,
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
            success: None,
        }
    }

    pub fn set_estimated_fee(&mut self, fee_sat: u64) {
        self.estimated_fee = Some(fee_sat);
        self.error = None;
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error = Some(message.into());
    }

    pub fn set_success(&mut self, message: impl Into<String>) {
        self.success = Some(message.into());
        self.error = None;
    }

    pub fn update(&mut self, message: SendMessage) -> Option<crate::app::AppMessage> {
        match message {
            SendMessage::ToAddressChanged(addr) => {
                self.to_address = addr;
                self.error = None;
                None
            }
            SendMessage::AmountChanged(amount) => {
                self.amount = amount;
                self.error = None;
                self.estimated_fee = None;
                None
            }
            SendMessage::FeeModeChanged(mode) => {
                self.fee_mode = mode;
                self.error = None;
                None
            }
            SendMessage::FeeAmountChanged(fee) => {
                self.fee_amount = fee;
                self.error = None;
                None
            }
            SendMessage::UseAllFunds(use_all) => {
                self.use_all_funds = use_all;
                if use_all {
                    self.estimated_fee = None;
                }
                self.error = None;
                None
            }
            SendMessage::FromAddressChanged(addr) => {
                self.from_address = addr;
                self.error = None;
                self.estimated_fee = None;
                None
            }
            SendMessage::ChangeAddressChanged(addr) => {
                self.change_address = addr;
                self.error = None;
                None
            }
            SendMessage::BroadcastChanged(broadcast) => {
                self.broadcast = broadcast;
                None
            }
            SendMessage::EstimateFee => {
                if self.use_all_funds {
                    self.error =
                        Some("Send all funds không cần estimate fee trước".to_string());
                    return None;
                }

                let amount_sat = match parse_u64_required(&self.amount, "amount") {
                    Ok(value) => value,
                    Err(err) => {
                        self.error = Some(err);
                        return None;
                    }
                };

                let input_source = match parse_input_source(&self.from_address) {
                    Ok(value) => value,
                    Err(err) => {
                        self.error = Some(err);
                        return None;
                    }
                };

                self.error = None;
                self.success = None;
                Some(crate::app::AppMessage::EstimateSendFee {
                    amount_sat,
                    input_source,
                })
            }
            SendMessage::Send => {
                if self.to_address.trim().is_empty() {
                    self.error = Some("Vui lòng nhập địa chỉ nhận".to_string());
                    return None;
                }

                let input_source = match parse_input_source(&self.from_address) {
                    Ok(value) => value,
                    Err(err) => {
                        self.error = Some(err);
                        return None;
                    }
                };

                let change_strategy = match parse_change_strategy(&self.change_address) {
                    Ok(value) => value,
                    Err(err) => {
                        self.error = Some(err);
                        return None;
                    }
                };

                let fee_mode = match self.fee_mode {
                    SimpleFeeMode::Auto => FeeMode::Auto,
                    SimpleFeeMode::Fixed => match parse_u64_required(&self.fee_amount, "fee") {
                        Ok(value) => FeeMode::FixedSat(value),
                        Err(err) => {
                            self.error = Some(err);
                            return None;
                        }
                    },
                };

                let amount_sat = if self.use_all_funds {
                    None
                } else {
                    match parse_u64_required(&self.amount, "amount") {
                        Ok(value) => Some(value),
                        Err(err) => {
                            self.error = Some(err);
                            return None;
                        }
                    }
                };

                self.error = None;
                self.success = None;

                Some(crate::app::AppMessage::SendTransaction(
                    crate::app::SendRequest {
                        to_address: self.to_address.trim().to_string(),
                        amount_sat,
                        fee_mode,
                        use_all_funds: self.use_all_funds,
                        input_source,
                        change_strategy,
                        broadcast: self.broadcast,
                    },
                ))
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
                self.success = None;
                self.estimated_fee = None;
                None
            }
        }
    }

    pub fn view(&self, wallet: Option<&WalletEntry>) -> Element<'_, SendMessage> {
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
            text("To Address")
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            text_input("Enter recipient address...", &self.to_address)
                .on_input(SendMessage::ToAddressChanged)
                .padding(12)
                .size(14)
        ]
        .spacing(4);

        let use_all_toggle = row![
            text("Send All Funds")
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_width(Length::Fill),
            button(text(if self.use_all_funds { "ON" } else { "OFF" }).size(12))
                .on_press(SendMessage::UseAllFunds(!self.use_all_funds))
                .padding(6)
                .style(if self.use_all_funds {
                    primary_button_style()
                } else {
                    secondary_button_style()
                })
        ]
        .align_y(Alignment::Center);

        let amount_input = column![
            text("Amount (sat)")
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            text_input("Enter amount in satoshis...", &self.amount)
                .on_input(SendMessage::AmountChanged)
                .padding(12)
                .size(14)
        ]
        .spacing(4);

        let fee_mode_section = column![
            text("Fee Mode")
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            row![
                radio(
                    "Auto",
                    SimpleFeeMode::Auto,
                    Some(self.fee_mode),
                    SendMessage::FeeModeChanged
                )
                .size(14),
                Space::with_width(16),
                radio(
                    "Fixed",
                    SimpleFeeMode::Fixed,
                    Some(self.fee_mode),
                    SendMessage::FeeModeChanged
                )
                .size(14),
            ]
            .spacing(8)
        ]
        .spacing(4);

        let fee_input: Element<'_, SendMessage> = if self.fee_mode == SimpleFeeMode::Fixed {
            column![
                text("Fee Amount (sat)")
                    .size(14)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Enter fee in satoshis...", &self.fee_amount)
                    .on_input(SendMessage::FeeAmountChanged)
                    .padding(12)
                    .size(14)
            ]
            .spacing(4)
            .into()
        } else if let Some(fee) = self.estimated_fee {
            text(format!("Estimated fee: {} sat", fee))
                .size(14)
                .style(text_color(Colors::SUCCESS))
                .into()
        } else {
            text("Click 'Estimate Fee' for auto mode")
                .size(14)
                .style(text_color(Colors::TEXT_MUTED))
                .into()
        };

        let estimate_btn = button(text("Estimate Fee").size(14))
            .on_press(SendMessage::EstimateFee)
            .padding(10)
            .style(secondary_button_style());

        let advanced_section = column![
            text("Advanced Options (Optional)")
                .size(16)
                .style(text_color(Colors::TEXT_PRIMARY)),
            Space::with_height(8),
            column![
                text("From address indexes (comma separated)")
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Example: 0,1,4", &self.from_address)
                    .on_input(SendMessage::FromAddressChanged)
                    .padding(10)
                    .size(12)
            ]
            .spacing(2),
            Space::with_height(8),
            column![
                text("Change address index (empty = derive new)")
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input("Example: 2", &self.change_address)
                    .on_input(SendMessage::ChangeAddressChanged)
                    .padding(10)
                    .size(12)
            ]
            .spacing(2),
            Space::with_height(8),
            row![
                text("Broadcast Immediately")
                    .size(14)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_width(Length::Fill),
                button(text(if self.broadcast { "YES" } else { "NO" }).size(12))
                    .on_press(SendMessage::BroadcastChanged(!self.broadcast))
                    .padding(6)
                    .style(if self.broadcast {
                        primary_button_style()
                    } else {
                        secondary_button_style()
                    })
            ]
            .align_y(Alignment::Center)
        ]
        .spacing(8);

        let error_text: Element<'_, SendMessage> = if let Some(error) = &self.error {
            text(error.as_str())
                .size(14)
                .style(text_color(Colors::ERROR))
                .into()
        } else {
            Space::with_height(0).into()
        };

        let success_text: Element<'_, SendMessage> = if let Some(success) = &self.success {
            text(success.as_str())
                .size(14)
                .style(text_color(Colors::SUCCESS))
                .into()
        } else {
            Space::with_height(0).into()
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
            container(advanced_section).style(card_style()).padding(16),
            Space::with_height(24),
            error_text,
            success_text,
            Space::with_height(16),
            row![send_btn, Space::with_width(16), clear_btn].width(Length::Fill),
        ]
        .spacing(8)
        .padding(32);

        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn parse_u64_required(raw: &str, field: &str) -> Result<u64, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(format!("Vui lòng nhập {field}"));
    }

    let parsed = value
        .parse::<u64>()
        .map_err(|_| format!("{field} phải là số nguyên dương"))?;

    if parsed == 0 {
        return Err(format!("{field} phải lớn hơn 0"));
    }

    Ok(parsed)
}

fn parse_input_source(raw: &str) -> Result<InputSource, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(InputSource::All);
    }

    let mut indexes = Vec::new();
    for token in trimmed.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let index = token
            .parse::<u32>()
            .map_err(|_| "from indexes không hợp lệ (ví dụ: 0,1,2)".to_string())?;
        indexes.push(index);
    }

    if indexes.is_empty() {
        return Err("from indexes không được rỗng".to_string());
    }

    Ok(InputSource::AddressIndexes(indexes))
}

fn parse_change_strategy(raw: &str) -> Result<ChangeStrategy, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(ChangeStrategy::NewAddress);
    }

    let index = trimmed
        .parse::<u32>()
        .map_err(|_| "change index không hợp lệ".to_string())?;
    Ok(ChangeStrategy::ExistingIndex(index))
}
