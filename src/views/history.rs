use crate::i18n::t;
use crate::components::{skeleton_transactions, error_card};
use crate::theme::{
    card_style, pick_list_menu_style, pick_list_style, secondary_button_style,
    selected_button_style, text_color, Colors,
};
use crate::views::wallet_picker::{selected_wallet_choice, wallet_choices};
use crate::wallet::{TxDirection, TxRecord, Wallet, WalletNetwork};
use crate::utils::{format_btc_with_spaces, format_number_with_spaces};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};
use chrono::DateTime;
use iced::{
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, Space},
    Alignment, Element, Length,
};

fn format_timestamp(timestamp: u64) -> String {
    let datetime = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_default();
    datetime.format("%d/%m/%Y %H:%M:%S").to_string()
}

fn format_btc_and_sat(amount_sat: i64) -> String {
    let abs_sat = amount_sat.unsigned_abs();
    let formatted_btc = format_btc_with_spaces(abs_sat);
    let formatted_sat = format_number_with_spaces(abs_sat, 3);
    format!("{} BTC ({} sat)", formatted_btc, formatted_sat)
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
        }
    }

    pub fn update(&mut self, message: HistoryMessage) -> Option<HistoryEvent> {
        match message {
            HistoryMessage::SelectWallet(_) => {
                self.search_query.clear();
                None
            }
            HistoryMessage::Refresh => Some(HistoryEvent::Refresh),
            HistoryMessage::FilterAll => {
                self.filter = Filter::All;
                None
            }
            HistoryMessage::FilterIncoming => {
                self.filter = Filter::Incoming;
                None
            }
            HistoryMessage::FilterOutgoing => {
                self.filter = Filter::Outgoing;
                None
            }
            HistoryMessage::FilterPending => {
                self.filter = Filter::Pending;
                None
            }
            HistoryMessage::FilterSelfTransfer => {
                self.filter = Filter::SelfTransfer;
                None
            }
            HistoryMessage::CopyTxid(_) => None,
            HistoryMessage::OpenExplorer(_) => None,
            HistoryMessage::SearchChanged(query) => {
                self.search_query = query;
                None
            }
            HistoryMessage::ExportCsv => Some(HistoryEvent::ExportCsv),
            HistoryMessage::ExportPdf => Some(HistoryEvent::ExportPdf),
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

        // Search & Export row
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

        content = content.push(filter_row);
        content = content.push(Space::with_height(8));
        content = content.push(search_export_row);

        if let Some(wallet) = wallet {
            // Show skeleton when refreshing
            if is_refreshing {
                content = content.push(Space::with_height(16));
                content = content.push(skeleton_transactions(5).map(|_| HistoryMessage::Refresh));
            } else {
                let search_lower = self.search_query.to_lowercase();
                let filtered_txs: Vec<&TxRecord> = wallet
                    .history
                    .iter()
                    .filter(|tx| {
                        // Filter by type
                        let type_match = match self.filter {
                            Filter::All => true,
                            Filter::Incoming => matches!(tx.direction, TxDirection::Incoming),
                            Filter::Outgoing => matches!(tx.direction, TxDirection::Outgoing),
                            Filter::Pending => !tx.confirmed,
                            Filter::SelfTransfer => matches!(tx.direction, TxDirection::SelfTransfer),
                        };
                        // Filter by search query (txid)
                        let search_match = search_lower.is_empty()
                            || tx.txid.to_lowercase().contains(&search_lower);
                        type_match && search_match
                    })
                    .collect();

                if filtered_txs.is_empty() {
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
                    content = content.push(Space::with_height(16));
                    content = content.push(
                        text(format!(
                            "{} {}",
                            filtered_txs.len(),
                            t("giao dịch", "transactions")
                        ))
                        .size(14)
                        .style(text_color(Colors::TEXT_SECONDARY)),
                    );
                    content = content.push(Space::with_height(8));

                    let mut tx_list = column![];

                    for tx in filtered_txs.iter() {
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
                        WalletNetwork::Mainnet => {
                            format!("https://blockstream.info/tx/{}", tx.txid)
                        }
                        WalletNetwork::Testnet => {
                            format!("https://blockstream.info/testnet/tx/{}", tx.txid)
                        }
                    };

                    let tx_row = container(
                        column![
                            row![
                                text(direction_text)
                                    .size(14)
                                    .style(text_color(amount_color)),
                                Space::with_width(12),
                                text(txid_short)
                                    .size(14)
                                    .style(text_color(Colors::TEXT_PRIMARY)),
                                Space::with_width(12),
                                button(text(t("Sao chép", "Copy")).size(12))
                                    .on_press(HistoryMessage::CopyTxid(tx.txid.clone()))
                                    .padding(8)
                                    .style(secondary_button_style()),
                                Space::with_width(8),
                                button(text(t("Mở", "Open")).size(12))
                                    .on_press(HistoryMessage::OpenExplorer(explorer_url.clone()))
                                    .padding(8)
                                    .style(secondary_button_style()),
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
                            Space::with_height(8),
                            row![
                                text(if tx.confirmed {
                                    t("Đã xác nhận", "Confirmed")
                                } else {
                                    t("Chờ xác nhận", "Pending")
                                })
                                .size(12)
                                .style(text_color(
                                    if tx.confirmed {
                                        Colors::SUCCESS
                                    } else {
                                        Colors::WARNING
                                    }
                                )),
                                if tx.confirmed && tx.confirmations > 0 {
                                    text(format!(" ({})", tx.confirmations))
                                        .size(11)
                                        .style(text_color(Colors::SUCCESS))
                                } else {
                                    text("")
                                },
                                Space::with_width(16),
                                if let Some(fee) = tx.fee_sat {
                                    text(format!(
                                        "{}: {}",
                                        t("Phí", "Fee"),
                                        format_btc_and_sat(fee as i64)
                                    ))
                                    .size(12)
                                    .style(text_color(Colors::TEXT_MUTED))
                                } else {
                                    text("")
                                },
                                Space::with_width(Length::Fill),
                                if let Some(block_time) = tx.block_time {
                                    let time_text: Element<'_, HistoryMessage> = row![
                                        text(format_timestamp(block_time))
                                            .size(12)
                                            .style(text_color(Colors::TEXT_MUTED)),
                                        Space::with_width(4),
                                        text(Bootstrap::Check.to_string())
                                            .size(12)
                                            .font(BOOTSTRAP_FONT)
                                            .style(text_color(Colors::SUCCESS)),
                                    ]
                                    .align_y(Alignment::Center)
                                    .into();
                                    time_text
                                } else {
                                    let pending_text: Element<'_, HistoryMessage> = text(t("Chờ xác nhận", "Pending"))
                                        .size(11)
                                        .style(text_color(Colors::WARNING))
                                        .into();
                                    pending_text
                                },
                            ]
                            .align_y(Alignment::Center),
                        ]
                        .spacing(8),
                    )
                    .style(card_style())
                    .padding(16)
                    .width(Length::Fill);

                    tx_list = tx_list.push(tx_row);
                    tx_list = tx_list.push(Space::with_height(12));
                }

                content = content.push(scrollable(tx_list).height(Length::Fill));
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

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
