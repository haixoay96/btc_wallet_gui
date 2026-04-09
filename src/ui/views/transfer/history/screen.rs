use crate::core::wallet::{TxDirection, TxRecord, Wallet, WalletNetwork};
use crate::i18n::t;
use crate::ui::components::wallet_picker::{selected_wallet_choice, wallet_choices};
use crate::ui::components::{modal, skeleton_transactions};
use crate::ui::theme::{
    input_style, pick_list_menu_style, pick_list_style, primary_button_style,
    secondary_button_style, selected_button_style, text_color, text_muted_color,
    text_primary_color, text_scaled, text_secondary_color, Colors,
};
use crate::utils::{format_btc_with_spaces, format_number_with_spaces};
use chrono::DateTime;
use iced::{
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, Space},
    Alignment, Color, Element, Length,
};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

use super::structure::*;

fn format_timestamp(timestamp: u64) -> String {
    let datetime = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_default();
    datetime.format("%d/%m/%Y %H:%M:%S").to_string()
}

/// Get confirmation status info for color coding and display
fn confirmation_status(tx: &TxRecord) -> (String, String, Color, String) {
    // Returns: (icon, status_text, color, estimated_time)
    if !tx.confirmed || tx.confirmations == 0 {
        (
            "clock".to_string(),
            t("Chờ xác nhận", "Pending").to_string(),
            Colors::WARNING,
            format!("~{}", t("đang chờ", "waiting")),
        )
    } else if tx.confirmations >= 6 {
        (
            "check".to_string(),
            t("Đã xác nhận", "Confirmed").to_string(),
            Colors::SUCCESS,
            format!(
                "✓ {} ({} {})",
                t("Đủ xác nhận", "Fully confirmed"),
                tx.confirmations,
                t("conf", "conf")
            ),
        )
    } else if tx.confirmations >= 3 {
        let remaining = 6 - tx.confirmations;
        let est_minutes = remaining * 10;
        (
            "check-circle".to_string(),
            format!("{} ({}/6)", t("Gần đủ", "Almost"), tx.confirmations),
            Colors::CONFIRMED_PARTIAL,
            format!(
                "~{} {} (~{} {})",
                remaining,
                t("conf còn lại", "conf remaining"),
                est_minutes,
                t("phút", "min")
            ),
        )
    } else {
        let remaining = 6 - tx.confirmations;
        let est_minutes = remaining * 10;
        (
            "hourglass".to_string(),
            format!(
                "{} ({}/6)",
                t("Ít xác nhận", "Low confirmations"),
                tx.confirmations
            ),
            Colors::CONFIRMED_LOW,
            format!(
                "~{} {} (~{} {})",
                remaining,
                t("conf còn lại", "conf remaining"),
                est_minutes,
                t("phút", "min")
            ),
        )
    }
}

fn parse_date_input(input: &str) -> Option<i64> {
    if input.is_empty() {
        return None;
    }
    // Try parsing DD/MM/YYYY
    if let Ok(date) = chrono::NaiveDate::parse_from_str(input, "%d/%m/%Y") {
        let datetime = date.and_hms_opt(0, 0, 0)?;
        Some(datetime.and_utc().timestamp())
    } else {
        None
    }
}

fn parse_amount_input(input: &str) -> Option<u64> {
    if input.is_empty() {
        return None;
    }
    // Parse as f64 BTC
    if let Ok(btc) = input.trim().parse::<f64>() {
        if btc < 0.0 {
            return None;
        }
        Some((btc * 100_000_000.0) as u64)
    } else {
        None
    }
}

fn format_btc_and_sat(amount_sat: i64) -> String {
    let abs_sat = amount_sat.unsigned_abs();
    let formatted_btc = format_btc_with_spaces(abs_sat);
    let formatted_sat = format_number_with_spaces(abs_sat, 3);
    let sign = if amount_sat < 0 { "-" } else { "" };
    format!("{}{} BTC ({} sat)", sign, formatted_btc, formatted_sat)
}

impl HistoryView {
    pub fn new() -> Self {
        Self {
            filter: Filter::All,
            search_query: String::new(),
            date_from: String::new(),
            date_to: String::new(),
            min_amount: String::new(),
            max_amount: String::new(),
            current_page: 0,
            items_per_page: 50,
            selected_tx_index: None,
        }
    }

    pub fn update(&mut self, message: HistoryMessage) -> Option<HistoryEvent> {
        match message {
            HistoryMessage::SelectWallet(_) => {
                self.search_query.clear();
                self.current_page = 0;
                None
            }
            HistoryMessage::Refresh => Some(HistoryEvent::Refresh),
            HistoryMessage::FilterAll => {
                self.filter = Filter::All;
                self.current_page = 0;
                None
            }
            HistoryMessage::FilterIncoming => {
                self.filter = Filter::Incoming;
                self.current_page = 0;
                None
            }
            HistoryMessage::FilterOutgoing => {
                self.filter = Filter::Outgoing;
                self.current_page = 0;
                None
            }
            HistoryMessage::FilterPending => {
                self.filter = Filter::Pending;
                self.current_page = 0;
                None
            }
            HistoryMessage::FilterSelfTransfer => {
                self.filter = Filter::SelfTransfer;
                self.current_page = 0;
                None
            }
            HistoryMessage::CopyTxid(_) => None,
            HistoryMessage::OpenExplorer(_) => None,
            HistoryMessage::SearchChanged(query) => {
                self.search_query = query;
                self.current_page = 0;
                None
            }
            HistoryMessage::ExportCsv => Some(HistoryEvent::ExportCsv),
            HistoryMessage::ExportPdf => Some(HistoryEvent::ExportPdf),

            // Filters
            HistoryMessage::DateFromChanged(val) => {
                self.date_from = val;
                self.current_page = 0;
                None
            }
            HistoryMessage::DateToChanged(val) => {
                self.date_to = val;
                self.current_page = 0;
                None
            }
            HistoryMessage::MinAmountChanged(val) => {
                self.min_amount = val;
                self.current_page = 0;
                None
            }
            HistoryMessage::MaxAmountChanged(val) => {
                self.max_amount = val;
                self.current_page = 0;
                None
            }

            // Pagination
            HistoryMessage::PageChanged(page) => {
                self.current_page = page;
                None
            }
            HistoryMessage::ItemsPerPageChanged(val) => {
                self.items_per_page = val;
                self.current_page = 0;
                None
            }

            // Modal
            HistoryMessage::ViewTransaction(idx) => {
                self.selected_tx_index = Some(idx);
                None
            }
            HistoryMessage::CloseTransactionDetail => {
                self.selected_tx_index = None;
                None
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        wallets: &'a [Wallet],
        selected_wallet: usize,
        is_refreshing: bool,
        compact: bool,
    ) -> Element<'a, HistoryMessage> {
        let wallet_options = wallet_choices(wallets);
        let selected_wallet_option = selected_wallet_choice(wallets, selected_wallet);
        let wallet = wallets.get(selected_wallet);

        let content_padding = if compact { 16 } else { 32 };
        let content_spacing = if compact { 12 } else { 20 };

        let title = text_scaled(t("Lịch sử giao dịch", "Transaction History"), 32)
            .style(text_primary_color());

        let wallet_selector = column![
            text_scaled(t("Từ ví", "From Wallet"), 14).style(text_secondary_color()),
            Space::with_height(4),
            pick_list(wallet_options, selected_wallet_option, |choice| {
                HistoryMessage::SelectWallet(choice.index)
            })
            .placeholder(t("Chọn ví...", "Select wallet..."))
            .width(Length::Fill)
            .padding(12)
            .style(pick_list_style())
            .menu_style(pick_list_menu_style()),
        ]
        .spacing(4);

        let mut content = column![title, wallet_selector]
            .spacing(content_spacing)
            .padding(content_padding);

        // Filter buttons
        let refresh_label = if is_refreshing {
            t("Đang làm mới...", "Refreshing...")
        } else {
            t("Làm mới", "Refresh")
        };
        let mut refresh_button = button(text_scaled(refresh_label, 12))
            .padding(8)
            .style(secondary_button_style());
        if !is_refreshing {
            refresh_button = refresh_button.on_press(HistoryMessage::Refresh);
        }

        let filter_row = row![
            button(text_scaled(t("Tất cả", "All"), 12))
                .on_press(HistoryMessage::FilterAll)
                .padding(8)
                .style(if self.filter == Filter::All {
                    selected_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(8),
            button(text_scaled(t("Nhận", "Incoming"), 12))
                .on_press(HistoryMessage::FilterIncoming)
                .padding(8)
                .style(if self.filter == Filter::Incoming {
                    selected_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(8),
            button(text_scaled(t("Gửi", "Outgoing"), 12))
                .on_press(HistoryMessage::FilterOutgoing)
                .padding(8)
                .style(if self.filter == Filter::Outgoing {
                    selected_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(8),
            button(text_scaled(t("Chờ xác nhận", "Pending"), 12))
                .on_press(HistoryMessage::FilterPending)
                .padding(8)
                .style(if self.filter == Filter::Pending {
                    selected_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(8),
            button(text_scaled(t("Tự chuyển", "Self transfer"), 12))
                .on_press(HistoryMessage::FilterSelfTransfer)
                .padding(8)
                .style(if self.filter == Filter::SelfTransfer {
                    selected_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(Length::Fill),
            refresh_button,
        ];

        content = content.push(filter_row);
        content = content.push(Space::with_height(8));

        // Advanced Filter Row (Date & Amount)
        let date_from_input = text_input("DD/MM/YYYY (Từ)", &self.date_from)
            .on_input(HistoryMessage::DateFromChanged)
            .padding(6)
            .size(12)
            .width(Length::Fixed(120.0))
            .style(input_style());

        let date_to_input = text_input("DD/MM/YYYY (Đến)", &self.date_to)
            .on_input(HistoryMessage::DateToChanged)
            .padding(6)
            .size(12)
            .width(Length::Fixed(120.0))
            .style(input_style());

        let amount_min_input = text_input("Min BTC", &self.min_amount)
            .on_input(HistoryMessage::MinAmountChanged)
            .padding(6)
            .size(12)
            .width(Length::Fixed(80.0))
            .style(input_style());

        let amount_max_input = text_input("Max BTC", &self.max_amount)
            .on_input(HistoryMessage::MaxAmountChanged)
            .padding(6)
            .size(12)
            .width(Length::Fixed(80.0))
            .style(input_style());

        let advanced_filter_row = row![
            text_scaled(t("Ngày:", "Date:"), 12).style(text_secondary_color()),
            date_from_input,
            Space::with_width(8),
            date_to_input,
            Space::with_width(16),
            text_scaled(t("Tiền:", "Amt:"), 12).style(text_secondary_color()),
            amount_min_input,
            Space::with_width(8),
            amount_max_input,
        ]
        .align_y(Alignment::Center);

        content = content.push(advanced_filter_row);
        content = content.push(Space::with_height(8));

        // Search & Export Row
        let search_input = text_input(t("Tìm kiếm txid...", "Search txid..."), &self.search_query)
            .on_input(HistoryMessage::SearchChanged)
            .padding(8)
            .size(12)
            .width(Length::Fixed(200.0))
            .style(input_style());

        let export_row = row![
            button(text_scaled(t("Xuất CSV", "Export CSV"), 11))
                .on_press(HistoryMessage::ExportCsv)
                .padding(6)
                .style(secondary_button_style()),
            Space::with_width(6),
            button(text_scaled(t("Xuất PDF", "Export PDF"), 11))
                .on_press(HistoryMessage::ExportPdf)
                .padding(6)
                .style(secondary_button_style()),
        ];

        let search_export_row =
            row![search_input, Space::with_width(8), export_row,].align_y(Alignment::Center);

        content = content.push(search_export_row);
        content = content.push(Space::with_height(8));

        if let Some(wallet) = wallet {
            if is_refreshing {
                content = content.push(Space::with_height(16));
                content = content.push(skeleton_transactions(5).map(|_| HistoryMessage::Refresh));
            } else {
                // Filtering Logic
                let search_lower = self.search_query.to_lowercase();
                let parsed_date_from = parse_date_input(&self.date_from);
                let parsed_date_to = parse_date_input(&self.date_to);
                let parsed_min_amt = parse_amount_input(&self.min_amount);
                let parsed_max_amt = parse_amount_input(&self.max_amount);

                let filtered_txs: Vec<&TxRecord> = wallet
                    .history
                    .iter()
                    .filter(|tx| {
                        // 1. Type Filter
                        let type_match = match self.filter {
                            Filter::All => true,
                            Filter::Incoming => matches!(tx.direction, TxDirection::Incoming),
                            Filter::Outgoing => matches!(tx.direction, TxDirection::Outgoing),
                            Filter::Pending => !tx.confirmed,
                            Filter::SelfTransfer => {
                                matches!(tx.direction, TxDirection::SelfTransfer)
                            }
                        };
                        if !type_match {
                            return false;
                        }

                        // 2. Search Filter
                        let search_match = search_lower.is_empty()
                            || tx.txid.to_lowercase().contains(&search_lower);
                        if !search_match {
                            return false;
                        }

                        // 3. Date Filter
                        if let Some(time) = tx.block_time {
                            if let Some(start) = parsed_date_from {
                                if start > 0 && time < start as u64 {
                                    return false;
                                }
                            }
                            if let Some(end) = parsed_date_to {
                                // End date inclusive (end of day)
                                let end_of_day = (end + 86400).max(0) as u64;
                                if time > end_of_day {
                                    return false;
                                }
                            }
                        } else {
                            // Pending txs: only show if no date range specified
                            if parsed_date_from.is_some() || parsed_date_to.is_some() {
                                return false;
                            }
                        }

                        // 4. Amount Filter (Absolute value)
                        let abs_amt = tx.amount_sat.unsigned_abs();
                        if let Some(min) = parsed_min_amt {
                            if abs_amt < min {
                                return false;
                            }
                        }
                        if let Some(max) = parsed_max_amt {
                            if abs_amt > max {
                                return false;
                            }
                        }

                        true
                    })
                    .collect();

                // Pagination Logic
                let total_items = filtered_txs.len();
                let total_pages = if self.items_per_page == 0 {
                    1
                } else {
                    total_items.div_ceil(self.items_per_page)
                };

                let current_page = if self.current_page >= total_pages && total_pages > 0 {
                    total_pages - 1
                } else {
                    self.current_page
                };

                let start_idx = current_page * self.items_per_page;
                let end_idx = std::cmp::min(start_idx + self.items_per_page, total_items);

                let page_txs = if start_idx < total_items {
                    &filtered_txs[start_idx..end_idx]
                } else {
                    &[]
                };

                // Display Info
                if total_items == 0 {
                    content = content.push(Space::with_height(40));
                    content = content.push(
                        container(
                            text_scaled(t("Không có giao dịch", "No transactions found"), 16)
                                .style(text_muted_color()),
                        )
                        .padding(40)
                        .center_x(Length::Fill),
                    );
                } else {
                    content = content.push(Space::with_height(8));
                    content = content.push(row![
                        text(format!(
                            "{} {} ",
                            total_items,
                            t("giao dịch", "transactions")
                        ))
                        .size(14)
                        .style(text_secondary_color()),
                        Space::with_width(Length::Fill),
                        text(format!(
                            "{} {}/{} ",
                            t("Trang", "Page"),
                            if total_pages == 0 {
                                0
                            } else {
                                current_page + 1
                            },
                            total_pages
                        ))
                        .size(12)
                        .style(text_muted_color())
                    ]);
                    content = content.push(Space::with_height(8));

                    let mut tx_list = column![];

                    for tx in page_txs.iter() {
                        // Find original index in wallet history for modal
                        let original_idx = wallet
                            .history
                            .iter()
                            .position(|t| t.txid == tx.txid)
                            .unwrap_or(0);

                        let direction_text = match tx.direction {
                            TxDirection::Incoming => t("NHẬN", "IN"),
                            TxDirection::Outgoing => t("GỬI", "OUT"),
                            TxDirection::SelfTransfer => t("TỰ", "SELF"),
                        };
                        let amount_color = match tx.direction {
                            TxDirection::Incoming => Colors::SUCCESS,
                            TxDirection::Outgoing => Colors::ERROR,
                            TxDirection::SelfTransfer => Colors::TEXT_SECONDARY,
                        };
                        let amount_sign = match tx.direction {
                            TxDirection::Incoming => "+",
                            TxDirection::Outgoing => "-",
                            TxDirection::SelfTransfer => "",
                        };
                        let txid_short = format!("{}...", &tx.txid[..16.min(tx.txid.len())]);

                        let _explorer_url = match wallet.network {
                            WalletNetwork::Mainnet => {
                                format!("https://blockstream.info/tx/{}", tx.txid)
                            }
                            WalletNetwork::Testnet => {
                                format!("https://blockstream.info/testnet/tx/{}", tx.txid)
                            }
                        };

                        // Row Clickable
                        let tx_row = container(
                            column![
                                row![
                                    text_scaled(direction_text, 14).style(text_color(amount_color)),
                                    Space::with_width(12),
                                    text_scaled(txid_short, 14).style(text_primary_color()),
                                    Space::with_width(Length::Fill),
                                    text(format!(
                                        "{}{}",
                                        amount_sign,
                                        format_btc_and_sat(tx.amount_sat)
                                    ))
                                    .size(14)
                                    .style(text_color(amount_color)),
                                ]
                                .align_y(Alignment::Center),
                                Space::with_height(4),
                                row![
                                    {
                                        let (icon_str, _, status_color, _) =
                                            confirmation_status(tx);
                                        let icon_bootstrap = if icon_str == "check" {
                                            Bootstrap::Check.to_string()
                                        } else if icon_str == "check-circle" {
                                            Bootstrap::CheckCircle.to_string()
                                        } else if icon_str == "hourglass" {
                                            Bootstrap::Hourglass.to_string()
                                        } else {
                                            Bootstrap::Clock.to_string()
                                        };
                                        text(icon_bootstrap)
                                            .font(BOOTSTRAP_FONT)
                                            .size(11)
                                            .style(text_color(status_color))
                                    },
                                    Space::with_width(4),
                                    {
                                        let (_, status_text, status_color, est_time) =
                                            confirmation_status(tx);
                                        column![
                                            text_scaled(status_text, 11)
                                                .style(text_color(status_color)),
                                            text_scaled(est_time, 9).style(text_muted_color()),
                                        ]
                                        .spacing(1)
                                    },
                                    Space::with_width(Length::Fill),
                                    if let Some(block_time) = tx.block_time {
                                        text_scaled(format_timestamp(block_time), 11)
                                            .style(text_muted_color())
                                    } else {
                                        text_scaled("", 11)
                                    },
                                ]
                                .align_y(Alignment::Center)
                            ]
                            .spacing(4),
                        )
                        .style(move |_| iced::widget::container::Style {
                            background: Some(iced::Background::Color(Color::from_rgba(
                                1.0, 1.0, 1.0, 0.02,
                            ))),
                            border: iced::border::rounded(8),
                            ..Default::default()
                        })
                        .padding(12)
                        .width(Length::Fill);

                        // Make row clickable to open modal
                        let clickable_row = button(tx_row)
                            .style(|_, _| iced::widget::button::Style {
                                background: None,
                                border: iced::Border::default(),
                                text_color: Colors::TEXT_PRIMARY,
                                ..Default::default()
                            })
                            .on_press(HistoryMessage::ViewTransaction(original_idx));

                        tx_list = tx_list.push(clickable_row);
                        tx_list = tx_list.push(Space::with_height(8));
                    }

                    content = content.push(scrollable(tx_list).height(Length::Fill));

                    // Pagination Controls
                    if total_pages > 1 {
                        let mut page_controls = row![].spacing(6).align_y(Alignment::Center);

                        // Prev
                        let mut prev_btn = button(text_scaled("<<", 12)).padding(6);
                        if current_page > 0 {
                            prev_btn =
                                prev_btn.on_press(HistoryMessage::PageChanged(current_page - 1));
                        } else {
                            prev_btn = prev_btn.style(muted_button_style());
                        }
                        page_controls = page_controls.push(prev_btn);

                        // Page Numbers (Simplified)
                        let mut start_p = current_page.saturating_sub(2);
                        let end_p = std::cmp::min(start_p + 5, total_pages);
                        if end_p - start_p < 5 && start_p > 0 {
                            start_p = std::cmp::max(0, end_p as isize - 5) as usize;
                        }

                        for p in start_p..end_p {
                            let btn = button(text_scaled(format!("{}", p + 1), 12))
                                .padding(6)
                                .style(if p == current_page {
                                    selected_button_style()
                                } else {
                                    secondary_button_style()
                                })
                                .on_press(HistoryMessage::PageChanged(p));
                            page_controls = page_controls.push(btn);
                        }

                        // Next
                        let mut next_btn = button(text_scaled(">>", 12)).padding(6);
                        if current_page < total_pages - 1 {
                            next_btn =
                                next_btn.on_press(HistoryMessage::PageChanged(current_page + 1));
                        } else {
                            next_btn = next_btn.style(muted_button_style());
                        }
                        page_controls = page_controls.push(next_btn);

                        // Items per page
                        let items_picker = pick_list(
                            vec![20, 50, 100],
                            Some(self.items_per_page),
                            HistoryMessage::ItemsPerPageChanged,
                        )
                        .width(Length::Fixed(60.0))
                        .padding(4)
                        .text_size(11);

                        content = content.push(Space::with_height(8));
                        content = content.push(
                            row![
                                page_controls,
                                Space::with_width(Length::Fill),
                                text_scaled(t("Hiển thị:", "Show:"), 11)
                                    .style(text_secondary_color()),
                                items_picker
                            ]
                            .align_y(Alignment::Center),
                        );
                    }
                }
            }
        } else {
            content = content.push(Space::with_height(40));
            content = content.push(
                container(
                    text_scaled(
                        t("Vui lòng chọn ví trước", "Please select a wallet first"),
                        18,
                    )
                    .style(text_color(Colors::ERROR)),
                )
                .padding(40)
                .center_x(Length::Fill),
            );
        }

        // Transaction Detail Modal
        let main_content = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        if let Some(idx) = self.selected_tx_index {
            if let Some(wallet) = wallet {
                if let Some(tx) = wallet.history.get(idx) {
                    let (modal_title, modal_content) =
                        self.render_tx_detail_modal(tx, Some(wallet.network));
                    return modal(
                        main_content,
                        modal_title,
                        modal_content,
                        HistoryMessage::CloseTransactionDetail,
                        compact,
                    );
                }
            }
        }

        main_content
    }

    fn render_tx_detail_modal<'a>(
        &self,
        tx: &TxRecord,
        network: Option<WalletNetwork>,
    ) -> (&'static str, Element<'a, HistoryMessage>) {
        let amount_color = match tx.direction {
            TxDirection::Incoming => Colors::SUCCESS,
            TxDirection::Outgoing => Colors::ERROR,
            TxDirection::SelfTransfer => Colors::TEXT_SECONDARY,
        };
        let direction_text = match tx.direction {
            TxDirection::Incoming => t("Giao dịch đến", "Incoming Transaction"),
            TxDirection::Outgoing => t("Giao dịch đi", "Outgoing Transaction"),
            TxDirection::SelfTransfer => t("Tự chuyển", "Self Transfer"),
        };

        let explorer_url = match network {
            Some(WalletNetwork::Mainnet) => format!("https://blockstream.info/tx/{}", tx.txid),
            Some(WalletNetwork::Testnet) => {
                format!("https://blockstream.info/testnet/tx/{}", tx.txid)
            }
            None => String::new(),
        };

        let info_row = |label: &str, value: String| -> Element<'a, HistoryMessage> {
            row![
                text_scaled(format!("{}:", label), 13)
                    .style(text_secondary_color())
                    .width(Length::Fixed(100.0)),
                text_scaled(value, 13).style(text_primary_color()),
                Space::with_width(Length::Fill),
            ]
            .spacing(8)
            .into()
        };

        let modal_content = column![
            text_scaled(direction_text, 18)
                .style(text_color(amount_color))
                .style(text_primary_color()),
            Space::with_height(16),
            info_row(t("TxID", "TxID"), tx.txid.clone()),
            Space::with_height(8),
            info_row(
                t("Thời gian", "Time"),
                tx.block_time.map(format_timestamp).unwrap_or_else(|| t(
                    "Đang chờ...",
                    "Pending..."
                )
                .to_string())
            ),
            Space::with_height(8),
            info_row(t("Số tiền", "Amount"), format_btc_and_sat(tx.amount_sat)),
            Space::with_height(8),
            info_row(
                t("Phí", "Fee"),
                tx.fee_sat
                    .map(|f| format_btc_and_sat(f as i64))
                    .unwrap_or_else(|| "N/A".to_string())
            ),
            Space::with_height(8),
            info_row(t("Trạng thái", "Status"), {
                let (_, status_text, _, est_time) = confirmation_status(tx);
                format!("{} - {}", status_text, est_time)
            }),
            Space::with_height(16),
            row![
                button(text_scaled(t("Sao chép TxID", "Copy TxID"), 12))
                    .on_press(HistoryMessage::CopyTxid(tx.txid.clone()))
                    .padding(8)
                    .style(secondary_button_style()),
                Space::with_width(8),
                button(text_scaled(t("Block Explorer", "Block Explorer"), 12))
                    .on_press(HistoryMessage::OpenExplorer(explorer_url))
                    .padding(8)
                    .style(primary_button_style()),
            ]
        ]
        .padding(16)
        .width(Length::Fixed(450.0))
        .into();

        (direction_text, modal_content)
    }
}

fn muted_button_style(
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, _| iced::widget::button::Style {
        background: None,
        border: iced::Border::default(),
        text_color: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
        ..Default::default()
    }
}
