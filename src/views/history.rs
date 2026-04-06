use crate::i18n::t;
use crate::components::{skeleton_transactions, modal};
use crate::theme::{
    card_style, pick_list_menu_style, pick_list_style, secondary_button_style,
    selected_button_style, text_color, Colors, primary_button_style,
};
use crate::views::wallet_picker::{selected_wallet_choice, wallet_choices};
use crate::wallet::{TxDirection, TxRecord, Wallet, WalletNetwork};
use crate::utils::{format_btc_with_spaces, format_number_with_spaces};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};
use chrono::DateTime;
use iced::{
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, Space},
    Alignment, Color, Element, Length,
};

fn format_timestamp(timestamp: u64) -> String {
    let datetime = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_default();
    datetime.format("%d/%m/%Y %H:%M:%S").to_string()
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
        if btc < 0.0 { return None; }
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

#[derive(Debug, Clone)]
pub enum HistoryMessage {
    SelectWallet(usize),
    Refresh,
    FilterAll,
    FilterIncoming,
    FilterOutgoing,
    FilterPending,
    FilterSelfTransfer,
    CopyTxid(String),
    OpenExplorer(String),
    SearchChanged(String),
    ExportCsv,
    ExportPdf,
    
    // Advanced Filters
    DateFromChanged(String),
    DateToChanged(String),
    MinAmountChanged(String),
    MaxAmountChanged(String),
    
    // Pagination
    PageChanged(usize),
    ItemsPerPageChanged(usize),

    // Detail Modal
    ViewTransaction(usize),
    CloseTransactionDetail,
}

#[derive(Debug, Clone)]
pub enum HistoryEvent {
    Refresh,
    ExportCsv,
    ExportPdf,
}

pub struct HistoryView {
    filter: Filter,
    search_query: String,
    
    // Advanced Filters
    date_from: String,
    date_to: String,
    min_amount: String,
    max_amount: String,
    
    // Pagination
    current_page: usize,
    items_per_page: usize,
    
    // Modal
    selected_tx_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Filter {
    All,
    Incoming,
    Outgoing,
    Pending,
    SelfTransfer,
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
            HistoryMessage::FilterAll => { self.filter = Filter::All; self.current_page = 0; None }
            HistoryMessage::FilterIncoming => { self.filter = Filter::Incoming; self.current_page = 0; None }
            HistoryMessage::FilterOutgoing => { self.filter = Filter::Outgoing; self.current_page = 0; None }
            HistoryMessage::FilterPending => { self.filter = Filter::Pending; self.current_page = 0; None }
            HistoryMessage::FilterSelfTransfer => { self.filter = Filter::SelfTransfer; self.current_page = 0; None }
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
            HistoryMessage::DateFromChanged(val) => { self.date_from = val; self.current_page = 0; None }
            HistoryMessage::DateToChanged(val) => { self.date_to = val; self.current_page = 0; None }
            HistoryMessage::MinAmountChanged(val) => { self.min_amount = val; self.current_page = 0; None }
            HistoryMessage::MaxAmountChanged(val) => { self.max_amount = val; self.current_page = 0; None }

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
    ) -> Element<'a, HistoryMessage> {
        let wallet_options = wallet_choices(wallets);
        let selected_wallet_option = selected_wallet_choice(wallets, selected_wallet);
        let wallet = wallets.get(selected_wallet);

        let title = text(t("Lịch sử giao dịch", "Transaction History"))
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let wallet_selector = column![
            text(t("Từ ví", "From Wallet"))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
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

        let mut content = column![title, wallet_selector].spacing(16).padding(32);

        // Filter buttons
        let refresh_label = if is_refreshing {
            t("Đang làm mới...", "Refreshing...")
        } else {
            t("Làm mới", "Refresh")
        };
        let mut refresh_button = button(text(refresh_label).size(12))
            .padding(8)
            .style(secondary_button_style());
        if !is_refreshing {
            refresh_button = refresh_button.on_press(HistoryMessage::Refresh);
        }

        let filter_row = row![
            button(text(t("Tất cả", "All")).size(12))
                .on_press(HistoryMessage::FilterAll)
                .padding(8)
                .style(if self.filter == Filter::All {
                    selected_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(8),
            button(text(t("Nhận", "Incoming")).size(12))
                .on_press(HistoryMessage::FilterIncoming)
                .padding(8)
                .style(if self.filter == Filter::Incoming {
                    selected_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(8),
            button(text(t("Gửi", "Outgoing")).size(12))
                .on_press(HistoryMessage::FilterOutgoing)
                .padding(8)
                .style(if self.filter == Filter::Outgoing {
                    selected_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(8),
            button(text(t("Chờ xác nhận", "Pending")).size(12))
                .on_press(HistoryMessage::FilterPending)
                .padding(8)
                .style(if self.filter == Filter::Pending {
                    selected_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(8),
            button(text(t("Tự chuyển", "Self transfer")).size(12))
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
            .width(Length::Fixed(120.0));
            
        let date_to_input = text_input("DD/MM/YYYY (Đến)", &self.date_to)
            .on_input(HistoryMessage::DateToChanged)
            .padding(6)
            .size(12)
            .width(Length::Fixed(120.0));

        let amount_min_input = text_input("Min BTC", &self.min_amount)
            .on_input(HistoryMessage::MinAmountChanged)
            .padding(6)
            .size(12)
            .width(Length::Fixed(80.0));

        let amount_max_input = text_input("Max BTC", &self.max_amount)
            .on_input(HistoryMessage::MaxAmountChanged)
            .padding(6)
            .size(12)
            .width(Length::Fixed(80.0));

        let advanced_filter_row = row![
            text(t("Ngày:", "Date:")).size(12).style(text_color(Colors::TEXT_SECONDARY)),
            date_from_input,
            Space::with_width(8),
            date_to_input,
            Space::with_width(16),
            text(t("Tiền:", "Amt:")).size(12).style(text_color(Colors::TEXT_SECONDARY)),
            amount_min_input,
            Space::with_width(8),
            amount_max_input,
        ].align_y(Alignment::Center);

        content = content.push(advanced_filter_row);
        content = content.push(Space::with_height(8));

        // Search & Export Row
        let search_input = text_input(
            t("Tìm kiếm txid...", "Search txid..."),
            &self.search_query,
        )
        .on_input(HistoryMessage::SearchChanged)
        .padding(8)
        .size(12)
        .width(Length::Fixed(200.0));

        let export_row = row![
            button(text(t("Xuất CSV", "Export CSV")).size(11))
                .on_press(HistoryMessage::ExportCsv)
                .padding(6)
                .style(secondary_button_style()),
            Space::with_width(6),
            button(text(t("Xuất PDF", "Export PDF")).size(11))
                .on_press(HistoryMessage::ExportPdf)
                .padding(6)
                .style(secondary_button_style()),
        ];

        let search_export_row = row![
            search_input,
            Space::with_width(8),
            export_row,
        ]
        .align_y(Alignment::Center);

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
                            Filter::SelfTransfer => matches!(tx.direction, TxDirection::SelfTransfer),
                        };
                        if !type_match { return false; }

                        // 2. Search Filter
                        let search_match = search_lower.is_empty() 
                            || tx.txid.to_lowercase().contains(&search_lower);
                        if !search_match { return false; }

                        // 3. Date Filter
                        if let Some(time) = tx.block_time {
                            if let Some(start) = parsed_date_from {
                                if start > 0 && time < start as u64 { return false; }
                            }
                            if let Some(end) = parsed_date_to {
                                // End date inclusive (end of day)
                                let end_of_day = (end + 86400).max(0) as u64;
                                if time > end_of_day { return false; } 
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
                            if abs_amt < min { return false; }
                        }
                        if let Some(max) = parsed_max_amt {
                            if abs_amt > max { return false; }
                        }

                        true
                    })
                    .collect();

                // Pagination Logic
                let total_items = filtered_txs.len();
                let total_pages = if self.items_per_page == 0 { 1 } else { (total_items + self.items_per_page - 1) / self.items_per_page };
                
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
                            text(t("Không có giao dịch", "No transactions found"))
                                .size(16)
                                .style(text_color(Colors::TEXT_MUTED)),
                        )
                        .padding(40)
                        .center_x(Length::Fill),
                    );
                } else {
                    content = content.push(Space::with_height(8));
                    content = content.push(
                        row![
                            text(format!(
                                "{} {} ",
                                total_items,
                                t("giao dịch", "transactions")
                            )).size(14).style(text_color(Colors::TEXT_SECONDARY)),
                            Space::with_width(Length::Fill),
                            text(format!(
                                "{} {}/{} ",
                                t("Trang", "Page"),
                                if total_pages == 0 { 0 } else { current_page + 1 },
                                total_pages
                            )).size(12).style(text_color(Colors::TEXT_MUTED))
                        ]
                    );
                    content = content.push(Space::with_height(8));

                    let mut tx_list = column![];
                    
                    for (idx, tx) in page_txs.iter().enumerate() {
                        // Find original index in wallet history for modal
                        let original_idx = wallet.history.iter().position(|t| t.txid == tx.txid).unwrap_or(0);

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

                        let explorer_url = match wallet.network {
                            WalletNetwork::Mainnet => format!("https://blockstream.info/tx/{}", tx.txid),
                            WalletNetwork::Testnet => format!("https://blockstream.info/testnet/tx/{}", tx.txid),
                        };

                        // Row Clickable
                        let tx_row = container(
                            column![
                                row![
                                    text(direction_text).size(14).style(text_color(amount_color)),
                                    Space::with_width(12),
                                    text(txid_short).size(14).style(text_color(Colors::TEXT_PRIMARY)),
                                    Space::with_width(Length::Fill),
                                    text(format!("{}{}", amount_sign, format_btc_and_sat(tx.amount_sat)))
                                        .size(14)
                                        .style(text_color(amount_color)),
                                ].align_y(Alignment::Center),
                                Space::with_height(4),
                                row![
                                    if tx.confirmed {
                                        text(Bootstrap::Check.to_string())
                                            .font(BOOTSTRAP_FONT)
                                            .size(11)
                                            .style(text_color(Colors::SUCCESS))
                                    } else {
                                        text(Bootstrap::Clock.to_string()) // Or ArrowRepeat
                                            .font(BOOTSTRAP_FONT)
                                            .size(11)
                                            .style(text_color(Colors::WARNING))
                                    },
                                    Space::with_width(4),
                                    text(if tx.confirmed {
                                        format!("{} ({} conf)", t("Đã xác nhận", "Confirmed"), tx.confirmations)
                                    } else {
                                        t("Chờ xác nhận", "Pending").to_string()
                                    })
                                    .size(11)
                                    .style(text_color(if tx.confirmed { Colors::SUCCESS } else { Colors::WARNING })),
                                    Space::with_width(Length::Fill),
                                    if let Some(block_time) = tx.block_time {
                                        text(format_timestamp(block_time)).size(11).style(text_color(Colors::TEXT_MUTED))
                                    } else {
                                        text("").size(11)
                                    },
                                ].align_y(Alignment::Center)
                            ]
                            .spacing(4),
                        )
                        .style(move |_| iced::widget::container::Style {
                            background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.02))),
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
                        let mut prev_btn = button(text("<<").size(12)).padding(6);
                        if current_page > 0 {
                            prev_btn = prev_btn.on_press(HistoryMessage::PageChanged(current_page - 1));
                        } else {
                            prev_btn = prev_btn.style(muted_button_style());
                        }
                        page_controls = page_controls.push(prev_btn);

                        // Page Numbers (Simplified)
                        let mut start_p = current_page.saturating_sub(2);
                        let mut end_p = std::cmp::min(start_p + 5, total_pages);
                        if end_p - start_p < 5 && start_p > 0 {
                            start_p = std::cmp::max(0, end_p as isize - 5) as usize;
                        }

                        for p in start_p..end_p {
                            let btn = button(text(format!("{}", p + 1)).size(12))
                                .padding(6)
                                .style(if p == current_page { selected_button_style() } else { secondary_button_style() })
                                .on_press(HistoryMessage::PageChanged(p));
                            page_controls = page_controls.push(btn);
                        }

                        // Next
                        let mut next_btn = button(text(">>").size(12)).padding(6);
                        if current_page < total_pages - 1 {
                            next_btn = next_btn.on_press(HistoryMessage::PageChanged(current_page + 1));
                        } else {
                            next_btn = next_btn.style(muted_button_style());
                        }
                        page_controls = page_controls.push(next_btn);

                        // Items per page
                        let items_picker = pick_list(vec![20, 50, 100], Some(self.items_per_page), HistoryMessage::ItemsPerPageChanged)
                            .width(Length::Fixed(60.0))
                            .padding(4)
                            .text_size(11);

                        content = content.push(Space::with_height(8));
                        content = content.push(row![
                            page_controls,
                            Space::with_width(Length::Fill),
                            text(t("Hiển thị:", "Show:")).size(11).style(text_color(Colors::TEXT_SECONDARY)),
                            items_picker
                        ].align_y(Alignment::Center));
                    }
                }
            }
        } else {
            content = content.push(Space::with_height(40));
            content = content.push(
                container(
                    text(t("Vui lòng chọn ví trước", "Please select a wallet first"))
                        .size(18)
                        .style(text_color(Colors::ERROR)),
                )
                .padding(40)
                .center_x(Length::Fill),
            );
        }

        // Transaction Detail Modal
        let main_content = container(content).width(Length::Fill).height(Length::Fill).into();

        if let Some(idx) = self.selected_tx_index {
            if let Some(wallet) = wallet {
                if let Some(tx) = wallet.history.get(idx) {
                    let modal_content = self.render_tx_detail_modal(tx, Some(wallet.network));
                    return modal(main_content, "", modal_content, HistoryMessage::CloseTransactionDetail).into();
                }
            }
        }

        main_content.into()
    }

    fn render_tx_detail_modal<'a>(
        &self,
        tx: &TxRecord,
        network: Option<WalletNetwork>,
    ) -> Element<'a, HistoryMessage> {
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
            Some(WalletNetwork::Testnet) => format!("https://blockstream.info/testnet/tx/{}", tx.txid),
            None => String::new(),
        };

        let info_row = |label: &str, value: String| -> Element<'a, HistoryMessage> {
            row![
                text(format!("{}:", label)).size(13).style(text_color(Colors::TEXT_SECONDARY)).width(Length::Fixed(100.0)),
                text(value).size(13).style(text_color(Colors::TEXT_PRIMARY)),
                Space::with_width(Length::Fill),
            ].spacing(8).into()
        };

        column![
            row![
                text(direction_text).size(18).style(text_color(amount_color)).style(text_color(Colors::TEXT_PRIMARY)),
                Space::with_width(Length::Fill),
                button(text(Bootstrap::X.to_string()).font(BOOTSTRAP_FONT).size(16))
                    .on_press(HistoryMessage::CloseTransactionDetail)
                    .padding(4)
                    .style(secondary_button_style())
            ].align_y(Alignment::Center),
            Space::with_height(16),
            info_row(t("TxID", "TxID"), tx.txid.clone()),
            Space::with_height(8),
            info_row(t("Thời gian", "Time"), tx.block_time.map(format_timestamp).unwrap_or_else(|| t("Đang chờ...", "Pending...").to_string())),
            Space::with_height(8),
            info_row(t("Số tiền", "Amount"), format_btc_and_sat(tx.amount_sat)),
            Space::with_height(8),
            info_row(t("Phí", "Fee"), tx.fee_sat.map(|f| format_btc_and_sat(f as i64)).unwrap_or_else(|| "N/A".to_string())),
            Space::with_height(8),
            info_row(t("Trạng thái", "Status"), if tx.confirmed { 
                format!("{} ({} confirmations)", t("Đã xác nhận", "Confirmed"), tx.confirmations) 
            } else { 
                t("Chờ xác nhận", "Pending").to_string() 
            }),
            Space::with_height(16),
            row![
                button(text(t("Sao chép TxID", "Copy TxID")).size(12))
                    .on_press(HistoryMessage::CopyTxid(tx.txid.clone()))
                    .padding(8)
                    .style(secondary_button_style()),
                Space::with_width(8),
                button(text(t("Block Explorer", "Block Explorer")).size(12))
                    .on_press(HistoryMessage::OpenExplorer(explorer_url))
                    .padding(8)
                    .style(primary_button_style()),
            ]
        ]
        .padding(16)
        .width(Length::Fixed(450.0))
        .into()
    }

    /// Get the currently selected transaction ID for copying
    pub fn get_selected_txid(&self) -> Option<&String> {
        // Return None if no transaction is selected in modal
        // This is used by keyboard shortcut to copy last viewed tx
        None // For now, return None - keyboard copy will use history list context
    }
}

fn muted_button_style() -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, _| iced::widget::button::Style {
        background: None,
        border: iced::Border::default(),
        text_color: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
        ..Default::default()
    }
}
