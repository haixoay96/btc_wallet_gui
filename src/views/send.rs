use iced::{
    widget::{
        button, column, container, pick_list, row, scrollable, stack, text, text_input, Space,
    },
    Alignment, Element, Length,
};

use crate::i18n::t;
use crate::components::{BtcUnit, format_amount_with_unit, help_icon, info_box, modal};
use crate::theme::{
    card_style, notice_style, pick_list_menu_style, pick_list_style, popup_dialog_style,
    popup_overlay_style, primary_button_style, secondary_button_style, selected_button_style,
    text_color, Colors, NoticeTone,
};
use crate::views::wallet_picker::{selected_wallet_choice, wallet_choices};
use crate::wallet::{validate_bitcoin_address, ChangeStrategy, InputSource, Wallet};
use crate::utils::{format_btc_with_spaces, format_number_with_spaces};
use iced_fonts::Bootstrap;

fn format_btc_and_sat(amount_sat: u64) -> String {
    let formatted_btc = format_btc_with_spaces(amount_sat);
    let formatted_sat = format_number_with_spaces(amount_sat, 3);
    format!("{} BTC ({} sat)", formatted_btc, formatted_sat)
}

fn parse_btc_to_sat(raw: &str, field_vi: &str, field_en: &str) -> Result<u64, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(format!(
            "{} {}",
            t("Vui lòng nhập", "Please enter"),
            t(field_vi, field_en)
        ));
    }

    let parsed = value.parse::<f64>().map_err(|_| {
        format!(
            "{} {}",
            t(field_vi, field_en),
            t("phải là số hợp lệ", "must be a valid number")
        )
    })?;

    if parsed <= 0.0 {
        return Err(format!(
            "{} {}",
            t(field_vi, field_en),
            t("phải lớn hơn 0", "must be greater than 0")
        ));
    }

    let amount_sat = (parsed * 100_000_000.0).round() as u64;
    if amount_sat == 0 {
        return Err(format!(
            "{} {}",
            t(field_vi, field_en),
            t(
                "quá nhỏ, phải >= 0.00000001 BTC",
                "too small, must be >= 0.00000001 BTC"
            )
        ));
    }

    Ok(amount_sat)
}

#[derive(Debug, Clone)]
pub enum SendMessage {
    SelectWallet(usize),
    ToAddressChanged(String),
    AmountChanged(String),
    FeeAmountChanged(String),
    MaxAmount,
    FromAddressChanged(String),
    ChangeAddressChanged(String),
    ToggleAdvanced,
    EstimateFee,
    Send,
    ConfirmSend,
    CancelSend,
    ChangeUnit(BtcUnit),
    ShowAmountHelp,
    ShowFeeHelp,
    ToggleAddressHelp,
    PassphraseConfirmChanged(String),
}

#[derive(Debug, Clone)]
pub struct SendRequest {
    pub to_address: String,
    pub amount_sat: Option<u64>,
    pub fee_sat: Option<u64>,
    pub input_source: InputSource,
    pub change_strategy: ChangeStrategy,
}

#[derive(Debug, Clone)]
pub enum SendEvent {
    SelectWallet(usize),
    EstimateSendFee {
        amount_sat: u64,
        input_source: crate::wallet::InputSource,
    },
    MaxAmount {
        input_source: crate::wallet::InputSource,
    },
    SendTransaction(SendRequest),
}

pub struct SendView {
    to_address: String,
    amount: String,
    fee_amount: String,
    from_address: String,
    change_address: String,
    broadcast: bool,
    estimated_fee: Option<u64>,
    to_address_error: Option<String>,
    amount_error: Option<String>,
    fee_error: Option<String>,
    error: Option<String>,
    success: Option<String>,
    pub show_confirm: bool,
    show_advanced: bool,
    unit: BtcUnit,
    show_amount_help: bool,
    show_fee_help: bool,
    show_address_help: bool,
    show_passphrase_confirm: bool,
    passphrase_confirm: String,
}

impl SendView {
    pub fn new() -> Self {
        Self {
            to_address: String::new(),
            amount: String::new(),
            fee_amount: String::new(),
            from_address: String::new(),
            change_address: String::new(),
            broadcast: false,
            estimated_fee: None,
            to_address_error: None,
            amount_error: None,
            fee_error: None,
            error: None,
            success: None,
            show_confirm: false,
            show_advanced: false,
            unit: BtcUnit::default(),
            show_amount_help: false,
            show_fee_help: false,
            show_address_help: false,
            show_passphrase_confirm: false,
            passphrase_confirm: String::new(),
        }
    }

    pub fn set_fee_amount(&mut self, fee_sat: u64) {
        let fee_btc = fee_sat as f64 / 100_000_000.0;
        self.fee_amount = format!("{:.8}", fee_btc)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
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

    pub fn clear_form(&mut self) {
        self.to_address.clear();
        self.amount.clear();
        self.fee_amount.clear();
        self.from_address.clear();
        self.change_address.clear();
        self.broadcast = false;
        self.error = None;
        self.success = None;
        self.estimated_fee = None;
        self.to_address_error = None;
        self.amount_error = None;
        self.fee_error = None;
        self.show_confirm = false;
        self.show_advanced = false;
    }

    pub fn set_max_amount(&mut self, amount_sat: u64) {
        let amount_btc = amount_sat as f64 / 100_000_000.0;
        self.amount = format!("{:.8}", amount_btc)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
        self.error = None;
        self.estimated_fee = None;
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
                self.to_address = addr.clone();
                if !addr.trim().is_empty() {
                    if let Err(err) = validate_bitcoin_address(&addr) {
                        self.to_address_error = Some(err);
                    } else {
                        self.to_address_error = None;
                    }
                } else {
                    self.to_address_error = None;
                }
                None
            }
            SendMessage::AmountChanged(amount) => {
                self.amount = amount.clone();
                if !amount.trim().is_empty() {
                    if let Err(err) = parse_btc_to_sat(&amount, "số lượng", "amount") {
                        self.amount_error = Some(err);
                    } else {
                        self.amount_error = None;
                    }
                } else {
                    self.amount_error = None;
                }
                self.estimated_fee = None;
                None
            }
            SendMessage::MaxAmount => {
                self.error = None;
                self.success = None;
                Some(SendEvent::MaxAmount {
                    input_source: match parse_input_source(&self.from_address) {
                        Ok(value) => value,
                        Err(_) => crate::wallet::InputSource::All,
                    },
                })
            }
            SendMessage::FeeAmountChanged(fee) => {
                self.fee_amount = fee.clone();
                if !fee.trim().is_empty() {
                    if let Err(err) = parse_btc_to_sat(&fee, "phí", "fee") {
                        self.fee_error = Some(err);
                    } else {
                        self.fee_error = None;
                    }
                } else {
                    self.fee_error = None;
                }
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
            SendMessage::ToggleAdvanced => {
                self.show_advanced = !self.show_advanced;
                None
            }
            SendMessage::EstimateFee => {
                let amount_sat = match parse_btc_to_sat(&self.amount, "số lượng", "amount") {
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
                Some(SendEvent::EstimateSendFee {
                    amount_sat,
                    input_source,
                })
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

                let _amount_sat = match parse_btc_to_sat(&self.amount, "số lượng", "amount") {
                    Ok(value) => value,
                    Err(err) => {
                        self.error = Some(err);
                        return None;
                    }
                };

                let _fee_sat = if !self.fee_amount.trim().is_empty() {
                    match parse_btc_to_sat(&self.fee_amount, "phí", "fee") {
                        Ok(value) => Some(value),
                        Err(err) => {
                            self.error = Some(err);
                            return None;
                        }
                    }
                } else {
                    None
                };

                // Show confirmation popup instead of sending directly
                self.show_confirm = true;
                self.error = None;
                None
            }
            SendMessage::ConfirmSend => {
                // Actually send the transaction
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

                let amount_sat = match parse_btc_to_sat(&self.amount, "số lượng", "amount") {
                    Ok(value) => value,
                    Err(err) => {
                        self.error = Some(err);
                        return None;
                    }
                };

                let fee_sat = if !self.fee_amount.trim().is_empty() {
                    match parse_btc_to_sat(&self.fee_amount, "phí", "fee") {
                        Ok(value) => Some(value),
                        Err(err) => {
                            self.error = Some(err);
                            return None;
                        }
                    }
                } else {
                    None
                };

                self.show_confirm = false;

                Some(SendEvent::SendTransaction(SendRequest {
                    to_address: self.to_address.trim().to_string(),
                    amount_sat: Some(amount_sat),
                    fee_sat,
                    input_source,
                    change_strategy,
                }))
            }
            SendMessage::CancelSend => {
                // Cancel confirmation
                self.show_confirm = false;
                None
            }
            SendMessage::ChangeUnit(unit) => {
                self.unit = unit;
                None
            }
            SendMessage::ShowAmountHelp => {
                self.show_amount_help = !self.show_amount_help;
                self.show_fee_help = false;
                None
            }
            SendMessage::ShowFeeHelp => {
                self.show_fee_help = !self.show_fee_help;
                self.show_amount_help = false;
                None
            }
            SendMessage::ToggleAddressHelp => {
                self.show_address_help = !self.show_address_help;
                None
            }
            SendMessage::PassphraseConfirmChanged(p) => {
                self.passphrase_confirm = p;
                None
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        wallets: &'a [Wallet],
        selected_wallet: usize,
        is_estimating_fee: bool,
        is_calculating_max: bool,
        is_sending: bool,
    ) -> Element<'a, SendMessage> {
        let wallet_options = wallet_choices(wallets);
        let selected_wallet_option = selected_wallet_choice(wallets, selected_wallet);
        let wallet = wallets.get(selected_wallet);
        let is_any_busy = is_estimating_fee || is_calculating_max || is_sending;

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
            let balance_sat = wallet.balance();
            let balance_btc = balance_sat as f64 / 100_000_000.0;
            container(
                row![
                    text(format!(
                        "{}: {:.8} BTC",
                        t("Sẵn có", "Available"),
                        balance_btc
                    ))
                    .size(14)
                    .style(text_color(Colors::TEXT_PRIMARY)),
                    Space::with_width(Length::Fill),
                    text(format!(
                        "{}: {}",
                        t("Mạng", "Network"),
                        wallet.network.as_str()
                    ))
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                ]
                .align_y(Alignment::Center),
            )
            .style(card_style())
            .padding(14)
        } else {
            container(
                text(t("Chưa chọn ví", "No wallet selected"))
                    .size(14)
                    .style(text_color(Colors::TEXT_PRIMARY)),
            )
            .style(notice_style(NoticeTone::Warning))
            .padding(12)
        };

        let to_label = row![
            text(t("Địa chỉ nhận", "To Address"))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_width(8),
            button(text(Bootstrap::QuestionCircle.to_string()).size(12).font(iced_fonts::BOOTSTRAP_FONT).style(text_color(Colors::TEXT_MUTED)))
                .on_press(SendMessage::ToggleAddressHelp)
                .padding(4)
                .style(secondary_button_style()),
        ]
        .align_y(Alignment::Center);

        let to_input = column![
            to_label,
            Space::with_height(4),
            text_input(
                t("Nhập địa chỉ nhận...", "Enter recipient address..."),
                &self.to_address
            )
            .on_input(SendMessage::ToAddressChanged)
            .padding(12)
            .size(14),
            if let Some(error) = &self.to_address_error {
                text(error.as_str())
                    .size(12)
                    .style(text_color(Colors::ERROR))
            } else {
                text("")
            }
        ]
        .spacing(4);

        // Address help box - i18n support
        let address_help: Element<'_, SendMessage> = if self.show_address_help {
            info_box(
                t("Địa chỉ nhận hợp lệ", "Valid recipient address"),
                t("Hỗ trợ BTC mainnet (bc1..., 1..., 3...) và testnet (tb1..., m/n/2...). Đảm bảo địa chỉ đúng network với ví nguồn.",
                  "Supports BTC mainnet (bc1..., 1..., 3...) and testnet (tb1..., m/n/2...). Make sure the address matches the wallet network.")
            ).map(|_| SendMessage::ToggleAddressHelp)
        } else {
            Space::with_height(0).into()
        };

        let max_label = if is_calculating_max {
            t("Đang tính...", "Calculating...")
        } else {
            t("Dùng tối đa", "Use max")
        };
        let mut max_button = button(text(max_label).size(12))
            .padding(6)
            .style(primary_button_style());
        if !is_calculating_max {
            max_button = max_button.on_press(SendMessage::MaxAmount);
        }

        let amount_label = row![
            text(t("Số lượng (BTC)", "Amount (BTC)"))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_width(8),
            button(text(Bootstrap::QuestionCircle.to_string()).size(12).font(iced_fonts::BOOTSTRAP_FONT).style(text_color(Colors::TEXT_MUTED)))
                .on_press(SendMessage::ShowAmountHelp)
                .padding(4)
                .style(secondary_button_style()),
        ]
        .align_y(Alignment::Center);

        let amount_input_row = row![
            text_input(
                t("Nhập số BTC...", "Enter amount in BTC..."),
                &self.amount,
            )
            .on_input(SendMessage::AmountChanged)
            .padding(12)
            .size(14),
            Space::with_width(8),
            max_button,
        ]
        .align_y(Alignment::Center);

        let amount_info: Element<'_, SendMessage> = if let Some(error) = &self.amount_error {
            text(error.as_str())
                .size(12)
                .style(text_color(Colors::ERROR))
                .into()
        } else if let Ok(amount_btc) = self.amount.trim().parse::<f64>() {
            if amount_btc > 0.0 {
                let amount_sat = (amount_btc * 100_000_000.0).round() as u64;
                let display = match self.unit {
                    BtcUnit::Btc => format_btc_with_spaces(amount_sat),
                    BtcUnit::Satoshi => format_number_with_spaces(amount_sat, 3),
                    BtcUnit::MilliBtc => format!("{:.5}", amount_sat as f64 / 100_000.0),
                };
                row![
                    text(format!("≈ {} {}", display, self.unit.symbol()))
                        .size(12)
                        .style(text_color(Colors::TEXT_MUTED)),
                    Space::with_width(8),
                    pick_list(BtcUnit::all(), Some(self.unit), SendMessage::ChangeUnit)
                        .width(Length::Fixed(80.0))
                        .padding(4)
                        .style(pick_list_style())
                        .menu_style(pick_list_menu_style()),
                ]
                .align_y(Alignment::Center)
                .into()
            } else {
                Space::with_height(0).into()
            }
        } else {
            Space::with_height(0).into()
        };

        // Amount help box - i18n support
        let amount_help: Element<'_, SendMessage> = if self.show_amount_help {
            info_box(
                t("Hướng dẫn nhập số lượng", "How to enter amount"),
                t("Nhập số BTC bạn muốn gửi. Ví dụ: 0.001 BTC = 100,000 satoshi. Phí giao dịch sẽ được trừ riêng.",
                  "Enter the BTC amount you want to send. Example: 0.001 BTC = 100,000 satoshis. Transaction fee will be deducted separately.")
            ).map(|_| SendMessage::AmountChanged(self.amount.clone()))
        } else {
            Space::with_height(0).into()
        };

        let estimate_label = if is_estimating_fee {
            t("Đang ước tính...", "Estimating...")
        } else {
            t("Ước tính tự động", "Auto estimate")
        };

        // Amount help box
        let amount_help: Element<'_, SendMessage> = if self.show_amount_help {
            info_box(
                t("Hướng dẫn nhập số lượng", "How to enter amount"),
                t("Nhập số BTC bạn muốn gửi. Ví dụ: 0.001 BTC = 100,000 satoshi. Phí giao dịch sẽ được trừ riêng.",
                  "Enter the BTC amount you want to send. Example: 0.001 BTC = 100,000 satoshis. Transaction fee will be deducted separately.")
            ).map(|_| SendMessage::AmountChanged(self.amount.clone()))
        } else {
            Space::with_height(0).into()
        };

        let estimate_label = if is_estimating_fee {
            t("Đang ước tính...", "Estimating...")
        } else {
            t("Ước tính tự động", "Auto estimate")
        };
        let mut estimate_btn = button(text(estimate_label).size(14))
            .padding(6)
            .style(primary_button_style());
        if !is_estimating_fee {
            estimate_btn = estimate_btn.on_press(SendMessage::EstimateFee);
        }

        let fee_label = row![
            text(t("Phí (BTC)", "Fee (BTC)"))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_width(8),
            button(text(Bootstrap::QuestionCircle.to_string()).size(12).font(iced_fonts::BOOTSTRAP_FONT).style(text_color(Colors::TEXT_MUTED)))
                .on_press(SendMessage::ShowFeeHelp)
                .padding(4)
                .style(secondary_button_style()),
        ]
        .align_y(Alignment::Center);

        let fee_row = row![
            text_input(
                t("Nhập phí BTC...", "Enter fee in BTC..."),
                &self.fee_amount
            )
            .on_input(SendMessage::FeeAmountChanged)
            .padding(12)
            .size(14),
            Space::with_width(8),
            estimate_btn,
        ]
        .align_y(Alignment::Center);

        let fee_info: Element<'_, SendMessage> = if let Some(error) = &self.fee_error {
            text(error.as_str())
                .size(12)
                .style(text_color(Colors::ERROR))
                .into()
        } else if let Ok(fee_btc) = self.fee_amount.trim().parse::<f64>() {
            if fee_btc > 0.0 {
                let fee_sat = (fee_btc * 100_000_000.0).round() as u64;
                let display = match self.unit {
                    BtcUnit::Btc => format_btc_with_spaces(fee_sat),
                    BtcUnit::Satoshi => format_number_with_spaces(fee_sat, 3),
                    BtcUnit::MilliBtc => format!("{:.5}", fee_sat as f64 / 100_000.0),
                };
                row![
                    text(format!("{} {}", display, self.unit.symbol()))
                        .size(12)
                        .style(text_color(Colors::TEXT_MUTED)),
                    Space::with_width(8),
                    pick_list(BtcUnit::all(), Some(self.unit), SendMessage::ChangeUnit)
                        .width(Length::Fixed(80.0))
                        .padding(4)
                        .style(pick_list_style())
                        .menu_style(pick_list_menu_style()),
                ]
                .align_y(Alignment::Center)
                .into()
            } else {
                Space::with_height(0).into()
            }
        } else {
            Space::with_height(0).into()
        };

        // Fee help box - i18n support
        let fee_help: Element<'_, SendMessage> = if self.show_fee_help {
            info_box(
                t("Hướng dẫn nhập phí", "How to enter fee"),
                t("Phí giao dịch tính bằng BTC. Phí cao hơn = giao dịch được xác nhận nhanh hơn. Bấm 'Ước tính tự động' để lấy phí hiện tại.",
                  "Transaction fee in BTC. Higher fee = faster confirmation. Click 'Auto estimate' to get current fee rates.")
            ).map(|_| SendMessage::FeeAmountChanged(self.fee_amount.clone()))
        } else {
            Space::with_height(0).into()
        };

        let advanced_toggle = button(
            text(if self.show_advanced {
                t("Ẩn tùy chọn nâng cao", "Hide advanced options")
            } else {
                t("Hiện tùy chọn nâng cao", "Show advanced options")
            })
            .size(13),
        )
        .on_press(SendMessage::ToggleAdvanced)
        .padding(10)
        .style(if self.show_advanced {
            selected_button_style()
        } else {
            secondary_button_style()
        });

        let advanced_section: Element<'_, SendMessage> = if self.show_advanced {
            container(
                column![
                    text(t(
                        "Tùy chọn nâng cao chỉ cần khi bạn muốn kiểm soát input/change cụ thể.",
                        "Advanced options are only needed when you want to control specific inputs/change.",
                    ))
                    .size(12)
                    .style(text_color(Colors::TEXT_SECONDARY)),
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
                ]
                .spacing(8),
            )
            .style(card_style())
            .padding(16)
            .width(Length::Fill)
            .into()
        } else {
            Space::with_height(0).into()
        };

        let error_text: Element<'_, SendMessage> = if let Some(error) = &self.error {
            container(
                text(error.as_str())
                    .size(14)
                    .style(text_color(Colors::TEXT_PRIMARY)),
            )
            .style(notice_style(NoticeTone::Error))
            .padding(12)
            .width(Length::Fill)
            .into()
        } else {
            Space::with_height(0).into()
        };

        let success_text: Element<'_, SendMessage> = if let Some(success) = &self.success {
            container(
                text(success.as_str())
                    .size(14)
                    .style(text_color(Colors::TEXT_PRIMARY)),
            )
            .style(notice_style(NoticeTone::Success))
            .padding(12)
            .width(Length::Fill)
            .into()
        } else {
            Space::with_height(0).into()
        };

        let send_label = if is_sending {
            t("Đang gửi giao dịch...", "Sending transaction...")
        } else {
            t("Gửi giao dịch", "Send Transaction")
        };
        let mut send_btn = button(text(send_label).size(16))
            .padding(14)
            .width(Length::Fill)
            .style(primary_button_style());
        if !is_any_busy {
            send_btn = send_btn.on_press(SendMessage::Send);
        }

        let content = column![
            title,
            Space::with_height(8),
            wallet_selector,
            Space::with_height(8),
            balance_text,
            Space::with_height(24),
            to_input,
            Space::with_height(4),
            address_help,
            Space::with_height(16),
            amount_label,
            Space::with_height(4),
            amount_input_row,
            amount_info,
            Space::with_height(4),
            amount_help,
            Space::with_height(16),
            fee_label,
            Space::with_height(4),
            fee_row,
            fee_info,
            Space::with_height(4),
            fee_help,
            Space::with_height(24),
            advanced_toggle,
            advanced_section,
            Space::with_height(24),
            error_text,
            success_text,
            Space::with_height(16),
            send_btn,
        ]
        .spacing(8)
        .padding(32);

        let base_content: Element<'_, SendMessage> = scrollable(content).width(Length::Fill).height(Length::Fill).into();

        if self.show_confirm {
            let amount_sat = parse_btc_to_sat(&self.amount, "số lượng", "amount").ok();
            let fee_sat = if self.fee_amount.trim().is_empty() {
                None
            } else {
                parse_btc_to_sat(&self.fee_amount, "phí", "fee").ok()
            };
            let balance_sat = wallet.map(Wallet::balance).unwrap_or_default() as u64;
            let remaining_sat = amount_sat.and_then(|amount| {
                let total_spend = amount.saturating_add(fee_sat.unwrap_or_default());
                balance_sat.checked_sub(total_spend)
            });
            let source_label = wallet
                .map(|value| format!("{} ({})", value.name, value.network.as_str()))
                .unwrap_or_else(|| t("Chưa chọn ví", "No wallet selected").to_string());

            let confirm_content = column![
                text(t(
                    "Kiểm tra kỹ thông tin trước khi broadcast lên mạng.",
                    "Review the details carefully before broadcasting to the network.",
                ))
                .size(13)
                .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(16),
                summary_row(t("Từ ví", "From Wallet"), source_label),
                summary_row(t("Đến", "To"), self.to_address.clone()),
                summary_row(
                    t("Số lượng", "Amount"),
                    amount_sat
                        .map(format_btc_and_sat)
                        .unwrap_or_else(|| self.amount.clone()),
                ),
                summary_row(
                    t("Phí", "Fee"),
                    fee_sat.map(format_btc_and_sat).unwrap_or_else(|| {
                        if self.fee_amount.trim().is_empty() {
                            t("Tự động hoặc chưa nhập", "Auto or not entered").to_string()
                        } else {
                            self.fee_amount.clone()
                        }
                    }),
                ),
                summary_row(
                    t("Địa chỉ trả lại", "Change destination"),
                    if self.change_address.trim().is_empty() {
                        t("Tạo địa chỉ mới", "Derive a new address").to_string()
                    } else {
                        format!("#{}", self.change_address.trim())
                    },
                ),
                if let Some(remaining) = remaining_sat {
                    container(summary_row(
                        t("Số dư còn lại", "Remaining balance"),
                        format_btc_and_sat(remaining),
                    ))
                    .style(notice_style(NoticeTone::Info))
                    .padding(10)
                    .width(Length::Fill)
                } else {
                    container(Space::with_height(0))
                },
                Space::with_height(16),
                text_input(
                    t("Nhập passphrase để xác nhận...", "Enter passphrase to confirm..."),
                    &self.passphrase_confirm,
                )
                .on_input(SendMessage::PassphraseConfirmChanged)
                .on_submit(SendMessage::ConfirmSend)
                .secure(true)
                .padding(12)
                .size(14),
                Space::with_height(16),
                container(
                    row![
                        button(text(t("Hủy", "Cancel")).size(14))
                            .on_press(SendMessage::CancelSend)
                            .padding(10)
                            .style(secondary_button_style()),
                        Space::with_width(10),
                        button(text(t("Broadcast giao dịch", "Broadcast transaction")).size(14))
                            .on_press(SendMessage::ConfirmSend)
                            .padding(10)
                            .style(primary_button_style()),
                    ]
                    .spacing(8),
                )
                .width(Length::Fill)
                .align_x(Alignment::Center),
            ]
            .spacing(8);

            modal(
                base_content.into(),
                t("Xác nhận giao dịch", "Confirm Transaction"),
                confirm_content.into(),
                SendMessage::CancelSend,
            )
        } else {
            base_content.into()
        }
    }
}

fn summary_row<'a>(label: &'a str, value: String) -> Element<'a, SendMessage> {
    row![
        text(label)
            .size(13)
            .style(text_color(Colors::TEXT_SECONDARY)),
        Space::with_width(Length::Fill),
        text(value).size(13).style(text_color(Colors::TEXT_PRIMARY)),
    ]
    .align_y(Alignment::Center)
    .into()
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

#[cfg(test)]
mod tests {
    use super::parse_input_source;
    use crate::wallet::InputSource;

    #[test]
    fn parse_input_source_accepts_empty_and_csv_indexes() {
        assert!(matches!(parse_input_source(""), Ok(InputSource::All)));
        assert!(matches!(
            parse_input_source("0, 2,5"),
            Ok(InputSource::AddressIndexes(indexes)) if indexes == vec![0, 2, 5]
        ));
    }

    #[test]
    fn parse_input_source_rejects_invalid_tokens() {
        assert!(parse_input_source("x,1").is_err());
        assert!(parse_input_source(",,,").is_err());
    }
}
