use iced::{
    widget::{
        button, column, container, pick_list, radio, row, scrollable, text, text_input, Space,
    },
    Alignment, Element, Length,
};
use std::fmt;

use crate::i18n::t;
use crate::theme::{
    card_style, pick_list_menu_style, pick_list_style, primary_button_style,
    secondary_button_style, text_color, Colors,
};
use crate::wallet::{ChangeStrategy, FeeMode, InputSource, WalletEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleFeeMode {
    Auto,
    Fixed,
}

#[derive(Debug, Clone)]
pub enum SendMessage {
    SelectWallet(usize),
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

#[derive(Debug, Clone)]
pub enum SendEvent {
    SelectWallet(usize),
    EstimateSendFee { amount_sat: u64, input_source: crate::wallet::InputSource },
    SendTransaction(crate::app::SendRequest),
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

    pub fn update(&mut self, message: SendMessage) -> Option<SendEvent> {
        match message {
            SendMessage::SelectWallet(index) => {
                self.error = None;
                self.success = None;
                self.estimated_fee = None;
                Some(SendEvent::SelectWallet(index))
            }
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
                    self.error = Some(
                        t(
                            "Send all funds không cần estimate fee trước",
                            "Send-all does not require fee estimation",
                        )
                        .to_string(),
                    );
                    return None;
                }

                let amount_sat = match parse_u64_required(&self.amount, "số lượng", "amount") {
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
                Some(SendEvent::EstimateSendFee { amount_sat, input_source })
            }
            SendMessage::Send => {
                if self.to_address.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập địa chỉ nhận",
                            "Please enter recipient address",
                        )
                        .to_string(),
                    );
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
                    SimpleFeeMode::Fixed => {
                        match parse_u64_required(&self.fee_amount, "phí", "fee") {
                            Ok(value) => FeeMode::FixedSat(value),
                            Err(err) => {
                                self.error = Some(err);
                                return None;
                            }
                        }
                    }
                };

                let amount_sat = if self.use_all_funds {
                    None
                } else {
                    match parse_u64_required(&self.amount, "số lượng", "amount") {
                        Ok(value) => Some(value),
                        Err(err) => {
                            self.error = Some(err);
                            return None;
                        }
                    }
                };

                self.error = None;
                self.success = None;

                Some(SendEvent::SendTransaction(crate::app::SendRequest {
                    to_address: self.to_address.trim().to_string(),
                    amount_sat,
                    fee_mode,
                    use_all_funds: self.use_all_funds,
                    input_source,
                    change_strategy,
                    broadcast: self.broadcast,
                }))
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

    pub fn view<'a>(
        &'a self,
        wallets: &'a [WalletEntry],
        selected_wallet: usize,
    ) -> Element<'a, SendMessage> {
        let wallet_options = wallet_choices(wallets);
        let selected_wallet_option = selected_wallet_choice(wallets, selected_wallet);
        let wallet = wallets.get(selected_wallet);

        let title = text(t("Gửi BTC", "Send BTC"))
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let wallet_selector = column![
            text(t("Từ ví", "From Wallet"))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            pick_list(wallet_options, selected_wallet_option, |choice| {
                SendMessage::SelectWallet(choice.index)
            })
            .placeholder(t("Chọn ví để gửi BTC...", "Select wallet to send BTC..."))
            .width(Length::Fill)
            .padding(12)
            .style(pick_list_style())
            .menu_style(pick_list_menu_style()),
        ]
        .spacing(4);

        let balance_text = if let Some(wallet) = wallet {
            let balance: i64 = wallet.history.iter().map(|tx| tx.amount_sat).sum();
            let balance_btc = balance as f64 / 100_000_000.0;
            text(format!(
                "{}: {:.8} BTC",
                t("Sẵn có", "Available"),
                balance_btc
            ))
            .size(14)
            .style(text_color(Colors::TEXT_SECONDARY))
        } else {
            text(t("Chưa chọn ví", "No wallet selected"))
                .size(14)
                .style(text_color(Colors::ERROR))
        };

        let to_input = column![
            text(t("Địa chỉ nhận", "To Address"))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            text_input(
                t("Nhập địa chỉ nhận...", "Enter recipient address..."),
                &self.to_address
            )
            .on_input(SendMessage::ToAddressChanged)
            .padding(12)
            .size(14)
        ]
        .spacing(4);

        let use_all_toggle = row![
            text(t("Gửi toàn bộ số dư", "Send All Funds"))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_width(Length::Fill),
            button(
                text(if self.use_all_funds {
                    t("BẬT", "ON")
                } else {
                    t("TẮT", "OFF")
                })
                .size(12)
            )
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
            text(t("Số lượng (sat)", "Amount (sat)"))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            text_input(
                t("Nhập số satoshi...", "Enter amount in satoshis..."),
                &self.amount
            )
            .on_input(SendMessage::AmountChanged)
            .padding(12)
            .size(14)
        ]
        .spacing(4);

        let fee_mode_section = column![
            text(t("Chế độ phí", "Fee Mode"))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            row![
                radio(
                    t("Tự động", "Auto"),
                    SimpleFeeMode::Auto,
                    Some(self.fee_mode),
                    SendMessage::FeeModeChanged
                )
                .size(14),
                Space::with_width(16),
                radio(
                    t("Cố định", "Fixed"),
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
                text(t("Phí (sat)", "Fee Amount (sat)"))
                    .size(14)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input(
                    t("Nhập phí satoshi...", "Enter fee in satoshis..."),
                    &self.fee_amount
                )
                .on_input(SendMessage::FeeAmountChanged)
                .padding(12)
                .size(14)
            ]
            .spacing(4)
            .into()
        } else if let Some(fee) = self.estimated_fee {
            text(format!(
                "{}: {} sat",
                t("Ước tính phí", "Estimated fee"),
                fee
            ))
            .size(14)
            .style(text_color(Colors::SUCCESS))
            .into()
        } else {
            text(t(
                "Bấm 'Ước tính phí' khi dùng chế độ tự động",
                "Click 'Estimate Fee' for auto mode",
            ))
            .size(14)
            .style(text_color(Colors::TEXT_MUTED))
            .into()
        };

        let estimate_btn = button(text(t("Ước tính phí", "Estimate Fee")).size(14))
            .on_press(SendMessage::EstimateFee)
            .padding(10)
            .style(secondary_button_style());

        let advanced_section = column![
            text(t(
                "Tùy chọn nâng cao (không bắt buộc)",
                "Advanced Options (Optional)"
            ))
            .size(16)
            .style(text_color(Colors::TEXT_PRIMARY)),
            Space::with_height(8),
            column![
                text(t(
                    "Chỉ số địa chỉ nguồn (phân tách bởi dấu phẩy)",
                    "From address indexes (comma separated)",
                ))
                .size(12)
                .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input(t("Ví dụ: 0,1,4", "Example: 0,1,4"), &self.from_address)
                    .on_input(SendMessage::FromAddressChanged)
                    .padding(10)
                    .size(12)
            ]
            .spacing(2),
            Space::with_height(8),
            column![
                text(t(
                    "Chỉ số địa chỉ trả lại (để trống = tạo mới)",
                    "Change address index (empty = derive new)",
                ))
                .size(12)
                .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(4),
                text_input(t("Ví dụ: 2", "Example: 2"), &self.change_address)
                    .on_input(SendMessage::ChangeAddressChanged)
                    .padding(10)
                    .size(12)
            ]
            .spacing(2),
            Space::with_height(8),
            row![
                text(t("Broadcast ngay", "Broadcast Immediately"))
                    .size(14)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_width(Length::Fill),
                button(
                    text(if self.broadcast {
                        t("CÓ", "YES")
                    } else {
                        t("KHÔNG", "NO")
                    })
                    .size(12)
                )
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

        let send_btn = button(text(t("Gửi giao dịch", "Send Transaction")).size(16))
            .on_press(SendMessage::Send)
            .padding(14)
            .width(Length::Fill)
            .style(primary_button_style());

        let clear_btn = button(text(t("Xóa form", "Clear Form")).size(14))
            .on_press(SendMessage::ClearForm)
            .padding(10)
            .style(secondary_button_style());

        let content = column![
            title,
            Space::with_height(8),
            wallet_selector,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct WalletChoice {
    index: usize,
    label: String,
}

impl fmt::Display for WalletChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

fn wallet_choices(wallets: &[WalletEntry]) -> Vec<WalletChoice> {
    wallets
        .iter()
        .enumerate()
        .map(|(index, wallet)| WalletChoice {
            index,
            label: format!("{} ({})", wallet.name, wallet.network.as_str()),
        })
        .collect()
}

fn selected_wallet_choice(wallets: &[WalletEntry], selected_wallet: usize) -> Option<WalletChoice> {
    wallets.get(selected_wallet).map(|wallet| WalletChoice {
        index: selected_wallet,
        label: format!("{} ({})", wallet.name, wallet.network.as_str()),
    })
}

fn parse_u64_required(raw: &str, field_vi: &str, field_en: &str) -> Result<u64, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(format!(
            "{} {}",
            t("Vui lòng nhập", "Please enter"),
            t(field_vi, field_en)
        ));
    }

    let parsed = value.parse::<u64>().map_err(|_| {
        format!(
            "{} {}",
            t(field_vi, field_en),
            t("phải là số nguyên dương", "must be a positive integer")
        )
    })?;

    if parsed == 0 {
        return Err(format!(
            "{} {}",
            t(field_vi, field_en),
            t("phải lớn hơn 0", "must be greater than 0")
        ));
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
        let index = token.parse::<u32>().map_err(|_| {
            t(
                "from indexes không hợp lệ (ví dụ: 0,1,2)",
                "Invalid from indexes (example: 0,1,2)",
            )
            .to_string()
        })?;
        indexes.push(index);
    }

    if indexes.is_empty() {
        return Err(t(
            "from indexes không được rỗng",
            "from indexes cannot be empty",
        )
        .to_string());
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
        .map_err(|_| t("change index không hợp lệ", "Invalid change index").to_string())?;
    Ok(ChangeStrategy::ExistingIndex(index))
}
