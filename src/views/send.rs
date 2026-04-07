use iced::{
    widget::{
        button, column, container, pick_list, row, scrollable, text, text_input, Space,
    },
    Alignment, Element, Length,
};

use crate::i18n::t;
use crate::components::{BtcUnit, info_box, modal, help_topic_panel, contact_picker_view, contact_form_view};
use crate::components::help_content::send_screen_topics;
use crate::storage::address_book::{AddressBook, ContactEntry};
use crate::theme::{text_scaled,
    card_style, input_style, notice_style, pick_list_menu_style, pick_list_style, primary_button_style,
    secondary_button_style, selected_button_style, text_color,
    text_primary_color, text_secondary_color, text_muted_color,
    Colors, NoticeTone,
};
use crate::views::wallet_picker::{selected_wallet_choice, wallet_choices};
use crate::wallet::{validate_bitcoin_address, ChangeStrategy, InputSource, Wallet};
use crate::utils::{format_btc_with_spaces, format_number_with_spaces};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

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
    ToggleHelpTopic(String),
    // Contact Book messages
    ToggleContactPicker,
    ContactSearchChanged(String),
    SelectContact(String),
    ShowContactForm,
    ContactFormNameChanged(String),
    ContactFormAddressChanged(String),
    ContactFormNoteChanged(String),
    SaveContact,
    CancelContactForm,
    HideContactPicker,
    DeleteContact(String),
    EditContact(String),
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
    pub to_address: String,
    amount: String,
    fee_amount: String,
    from_address: String,
    change_address: String,
    broadcast: bool,
    estimated_fee: Option<u64>,
    pub to_address_error: Option<String>,
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
    passphrase_confirm: String,
    
    // Help topics expansion state
    expanded_help_topics: std::collections::HashSet<String>,
    
    // Contact Book state
    pub show_contact_picker: bool,
    pub contact_search_query: String,
    pub show_contact_form: bool,
    pub contact_form_name: String,
    pub contact_form_address: String,
    pub contact_form_note: String,
    pub contact_form_address_error: Option<String>,
    pub editing_contact_id: Option<String>,
    pub contact_wallet_network: crate::wallet::WalletNetwork,
    
    // Matched contact label display
    pub matched_contact_name: Option<String>,
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
            passphrase_confirm: String::new(),
            expanded_help_topics: std::collections::HashSet::new(),
            show_contact_picker: false,
            contact_search_query: String::new(),
            show_contact_form: false,
            contact_form_name: String::new(),
            contact_form_address: String::new(),
            contact_form_note: String::new(),
            contact_form_address_error: None,
            editing_contact_id: None,
            contact_wallet_network: crate::wallet::WalletNetwork::Mainnet,
            matched_contact_name: None,
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
            SendMessage::ToggleHelpTopic(topic_id) => {
                if self.expanded_help_topics.contains(&topic_id) {
                    self.expanded_help_topics.remove(&topic_id);
                } else {
                    self.expanded_help_topics.insert(topic_id);
                }
                None
            }
            // Contact Book message handlers
            SendMessage::ToggleContactPicker => {
                self.show_contact_picker = !self.show_contact_picker;
                self.show_contact_form = false;
                self.contact_search_query.clear();
                None
            }
            SendMessage::ContactSearchChanged(query) => {
                self.contact_search_query = query;
                None
            }
            SendMessage::SelectContact(address) => {
                self.to_address = address.clone();
                self.to_address_error = None;
                self.show_contact_picker = false;
                // Trigger validation
                let _ = crate::wallet::validate_address_for_network(&address, crate::wallet::WalletNetwork::Mainnet);
                Some(SendEvent::SelectWallet(0)) // Dummy event, actual wallet selection handled elsewhere
            }
            SendMessage::ShowContactForm => {
                self.show_contact_form = true;
                self.contact_form_name.clear();
                self.contact_form_address = self.to_address.clone();
                self.contact_form_note.clear();
                self.editing_contact_id = None;
                // Validate pre-filled address from To Address
                if self.contact_form_address.trim().is_empty() {
                    self.contact_form_address_error = None;
                } else {
                    self.contact_form_address_error = validate_bitcoin_address(&self.contact_form_address).err();
                }
                None
            }
            SendMessage::ContactFormNameChanged(name) => {
                self.contact_form_name = name;
                None
            }
            SendMessage::ContactFormAddressChanged(address) => {
                self.contact_form_address = address;
                // Validate address on change
                if self.contact_form_address.trim().is_empty() {
                    self.contact_form_address_error = None;
                } else {
                    self.contact_form_address_error = validate_bitcoin_address(&self.contact_form_address).err();
                }
                None
            }
            SendMessage::ContactFormNoteChanged(note) => {
                self.contact_form_note = note;
                None
            }
            SendMessage::SaveContact => {
                // Will be handled by App
                None
            }
            SendMessage::CancelContactForm => {
                self.show_contact_form = false;
                self.editing_contact_id = None;
                None
            }
            SendMessage::HideContactPicker => {
                self.show_contact_picker = false;
                None
            }
            SendMessage::DeleteContact(id) => {
                // Will be handled by App
                None
            }
            SendMessage::EditContact(id) => {
                self.editing_contact_id = Some(id.clone());
                // Contact data will be loaded in App handler
                self.show_contact_form = true;
                self.show_contact_picker = false;
                None
            }
        }
    }

    /// Check if the send form is valid and can be submitted via keyboard shortcut
    pub fn can_submit(&self) -> bool {
        !self.to_address.trim().is_empty()
            && !self.amount.trim().is_empty()
            && self.to_address_error.is_none()
            && self.amount_error.is_none()
            && !self.show_confirm // Not already showing confirm
    }

    pub fn view<'a>(
        &'a self,
        wallets: &'a [Wallet],
        selected_wallet: usize,
        is_estimating_fee: bool,
        is_calculating_max: bool,
        is_sending: bool,
        address_book: &'a AddressBook,
    ) -> Element<'a, SendMessage> {
        let wallet_options = wallet_choices(wallets);
        let selected_wallet_option = selected_wallet_choice(wallets, selected_wallet);
        let wallet = wallets.get(selected_wallet);
        let is_any_busy = is_estimating_fee || is_calculating_max || is_sending;

        let title = text_scaled(t("Gửi BTC", "Send BTC"), 32)
            .style(text_primary_color());

        let wallet_selector = column![
            text_scaled(t("Từ ví", "From Wallet"), 14)
                .style(text_secondary_color()),
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
                    .style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    text(format!(
                        "{}: {}",
                        t("Mạng", "Network"),
                        wallet.network.as_str()
                    ))
                    .size(12)
                    .style(text_secondary_color()),
                ]
                .align_y(Alignment::Center),
            )
            .style(card_style())
            .padding(14)
        } else {
            container(
                text_scaled(t("Chưa chọn ví", "No wallet selected"), 14)
                    .style(text_primary_color()),
            )
            .style(notice_style(NoticeTone::Warning))
            .padding(12)
        };

        let to_label = row![
            text_scaled(t("Địa chỉ nhận", "To Address"), 14)
                .style(text_secondary_color()),
            Space::with_width(8),
            button(text_scaled(Bootstrap::QuestionCircle.to_string(), 12).font(iced_fonts::BOOTSTRAP_FONT).style(text_muted_color()))
                .on_press(SendMessage::ToggleAddressHelp)
                .padding(4)
                .style(secondary_button_style()),
        ]
        .align_y(Alignment::Center);

        let to_input = column![
            row![
                to_label,
                Space::with_width(Length::Fill),
                button(
                    row![
                        text_scaled(Bootstrap::Person.to_string(), 12)
                            .font(BOOTSTRAP_FONT)
                            .style(text_color(Colors::ACCENT_TEAL)),
                        Space::with_width(4),
                        text_scaled(t("Contact", "Contact"), 11)
                            .style(text_primary_color()),
                    ]
                    .align_y(Alignment::Center),
                )
                .on_press(SendMessage::ToggleContactPicker)
                .padding([6, 10])
                .style(if self.show_contact_picker { selected_button_style() } else { secondary_button_style() }),
            ]
            .align_y(Alignment::Center),
            Space::with_height(4),
            text_input(
                t("Nhập địa chỉ nhận...", "Enter recipient address..."),
                &self.to_address
            )
            .on_input(SendMessage::ToAddressChanged)
            .padding(12)
            .size(14)
            .style(input_style()),
            // Show matched contact label if exists
            if let Some(contact_name) = &self.matched_contact_name {
                row![
                    text_scaled(Bootstrap::PersonFill.to_string(), 10)
                        .font(BOOTSTRAP_FONT)
                        .style(text_color(Colors::ACCENT_PURPLE)),
                    Space::with_width(4),
                    text(format!("{}: {}", t("Contact", "Contact"), contact_name))
                        .size(11)
                        .style(text_color(Colors::ACCENT_TEAL)),
                ]
                .align_y(Alignment::Center)
            } else {
                row![]
            },
            if let Some(error) = &self.to_address_error {
                text_scaled(error.as_str(), 12)
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
        let mut max_button = button(text_scaled(max_label, 12))
            .padding(6)
            .style(primary_button_style());
        if !is_calculating_max {
            max_button = max_button.on_press(SendMessage::MaxAmount);
        }

        let amount_label = row![
            text(t("Số lượng (BTC)", "Amount (BTC)"))
                .size(14)
                .style(text_secondary_color()),
            Space::with_width(8),
            button(text_scaled(Bootstrap::QuestionCircle.to_string(), 12).font(iced_fonts::BOOTSTRAP_FONT).style(text_muted_color()))
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
            .size(14)
            .style(input_style()),
            Space::with_width(8),
            max_button,
        ]
        .align_y(Alignment::Center);

        let amount_info: Element<'_, SendMessage> = if let Some(error) = &self.amount_error {
            text_scaled(error.as_str(), 12)
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
                        .style(text_muted_color()),
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
        let mut estimate_btn = button(text_scaled(estimate_label, 14))
            .padding(6)
            .style(primary_button_style());
        if !is_estimating_fee {
            estimate_btn = estimate_btn.on_press(SendMessage::EstimateFee);
        }

        let fee_label = row![
            text(t("Phí (BTC)", "Fee (BTC)"))
                .size(14)
                .style(text_secondary_color()),
            Space::with_width(8),
            button(text_scaled(Bootstrap::QuestionCircle.to_string(), 12).font(iced_fonts::BOOTSTRAP_FONT).style(text_muted_color()))
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
            .size(14)
            .style(input_style()),
            Space::with_width(8),
            estimate_btn,
        ]
        .align_y(Alignment::Center);

        let fee_info: Element<'_, SendMessage> = if let Some(error) = &self.fee_error {
            text_scaled(error.as_str(), 12)
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
                        .style(text_muted_color()),
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
            text_scaled(if self.show_advanced {
                t("Ẩn tùy chọn nâng cao", "Hide advanced options")
            } else {
                t("Hiện tùy chọn nâng cao", "Show advanced options")
            }, 13),
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
                    text_scaled(t(
                        "Tùy chọn nâng cao chỉ cần khi bạn muốn kiểm soát input/change cụ thể.",
                        "Advanced options are only needed when you want to control specific inputs/change.",
                    ), 12)
                    .style(text_secondary_color()),
                    Space::with_height(8),
                    column![
                        text(t(
                            "Chỉ số địa chỉ nguồn (phân tách bởi dấu phẩy)",
                            "From address indexes (comma separated)",
                        ))
                        .size(12)
                        .style(text_secondary_color()),
                        Space::with_height(4),
                        text_input(t("Ví dụ: 0,1,4", "Example: 0,1,4"), &self.from_address)
                            .on_input(SendMessage::FromAddressChanged)
                            .padding(10)
                            .size(12)
                            .style(input_style())
                    ]
                    .spacing(2),
                    Space::with_height(8),
                    column![
                        text(t(
                            "Chỉ số địa chỉ trả lại (để trống = tạo mới)",
                            "Change address index (empty = derive new)",
                        ))
                        .size(12)
                        .style(text_secondary_color()),
                        Space::with_height(4),
                        text_input(t("Ví dụ: 2", "Example: 2"), &self.change_address)
                            .on_input(SendMessage::ChangeAddressChanged)
                            .padding(10)
                            .size(12)
                            .style(input_style())
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
                text_scaled(error.as_str(), 14)
                    .style(text_primary_color()),
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
                text_scaled(success.as_str(), 14)
                    .style(text_primary_color()),
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
        let mut send_btn = button(text_scaled(send_label, 16))
            .padding(14)
            .width(Length::Fill)
            .style(primary_button_style());
        if !is_any_busy {
            send_btn = send_btn.on_press(SendMessage::Send);
        }

        let mut content = column![
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
            Space::with_height(24),
            // Help topics section
            text_scaled(t("Trợ giúp", "Help"), 14)
                .style(text_primary_color()),
            Space::with_height(8),
        ]
        .spacing(8)
        .padding(32);

        // Add help topics
        let help_topics = send_screen_topics();
        let lang = crate::i18n::current_language();
        for topic in help_topics {
            let is_expanded = self.expanded_help_topics.contains(topic.id);
            let title = match lang {
                crate::i18n::AppLanguage::Vietnamese => topic.title_vi,
                crate::i18n::AppLanguage::English => topic.title_en,
            };
            let desc = match lang {
                crate::i18n::AppLanguage::Vietnamese => topic.description_vi,
                crate::i18n::AppLanguage::English => topic.description_en,
            };
            let detail: Option<&'static str> = match lang {
                crate::i18n::AppLanguage::Vietnamese => topic.detail_vi,
                crate::i18n::AppLanguage::English => topic.detail_en,
            };
            let panel: Element<'_, SendMessage> = help_topic_panel(
                topic.id,
                topic.icon,
                title,
                desc,
                detail,
                is_expanded,
                SendMessage::ToggleHelpTopic(topic.id.to_string()),
            );
            content = content.push(panel);
            content = content.push(Space::with_height(8));
        }

        let content = column![
            content
        ];

        let mut base_content: Element<'_, SendMessage> = scrollable(content).width(Length::Fill).height(Length::Fill).into();

        // Contact Picker Modal
        if self.show_contact_picker || self.show_contact_form {
            let contact_ui = if self.show_contact_form {
                contact_form_view(
                    &self.contact_form_name,
                    &self.contact_form_address,
                    &self.contact_form_note,
                    self.editing_contact_id.is_some(),
                    self.contact_form_address_error.as_deref(),
                    SendMessage::ContactFormNameChanged,
                    SendMessage::ContactFormAddressChanged,
                    SendMessage::ContactFormNoteChanged,
                    SendMessage::SaveContact,
                    SendMessage::CancelContactForm,
                    if self.editing_contact_id.is_some() {
                        Some(SendMessage::DeleteContact(self.editing_contact_id.clone().unwrap()))
                    } else {
                        None
                    },
                )
            } else {
                contact_picker_view(
                    address_book,
                    &self.contact_search_query,
                    SendMessage::ContactSearchChanged,
                    |contact| SendMessage::SelectContact(contact.address.clone()),
                    SendMessage::DeleteContact,
                    SendMessage::EditContact,
                    SendMessage::ShowContactForm,
                )
            };
            
            base_content = modal(
                base_content,
                if self.show_contact_form {
                    if self.editing_contact_id.is_some() {
                        t("Sửa Contact", "Edit Contact")
                    } else {
                        t("Thêm Contact", "Add Contact")
                    }
                } else {
                    t("Contact của tôi", "My Contacts")
                },
                contact_ui,
                SendMessage::HideContactPicker,
            );
        }

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
                text_scaled(t(
                    "Kiểm tra kỹ thông tin trước khi broadcast lên mạng.",
                    "Review the details carefully before broadcasting to the network.",
                ), 13)
                .style(text_secondary_color()),
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
                .size(14)
                .style(input_style()),
                Space::with_height(16),
                container(
                    row![
                        button(text_scaled(t("Hủy", "Cancel"), 14))
                            .on_press(SendMessage::CancelSend)
                            .padding(10)
                            .style(secondary_button_style()),
                        Space::with_width(10),
                        button(text_scaled(t("Broadcast giao dịch", "Broadcast transaction"), 14))
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
        text_scaled(label, 13)
            .style(text_secondary_color()),
        Space::with_width(Length::Fill),
        text_scaled(value, 13).style(text_primary_color()),
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
