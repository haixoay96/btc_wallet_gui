use crate::i18n::t;
use crate::theme::{card_style, primary_button_style, secondary_button_style, text_color, Colors};
use crate::wallet::{TxDirection, TxRecord, Wallet, WalletNetwork};
use chrono::DateTime;
use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Alignment, Element, Length,
};

fn format_timestamp(timestamp: u64) -> String {
    let datetime = DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_default();
    datetime.format("%d/%m/%Y %H:%M:%S").to_string()
}

fn format_btc_and_sat(amount_sat: i64) -> String {
    let amount_btc = amount_sat as f64 / 100_000_000.0;
    format!("{:.8} BTC ({} sat)", amount_btc.abs(), amount_sat.abs())
}

#[derive(Debug, Clone)]
pub enum HistoryMessage {
    Refresh,
    FilterAll,
    FilterIncoming,
    FilterOutgoing,
    CopyTxid(String),
    OpenExplorer(String, WalletNetwork),
}

#[derive(Debug, Clone)]
pub enum HistoryEvent {
    Refresh,
}

pub struct HistoryView {
    filter: Filter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Filter {
    All,
    Incoming,
    Outgoing,
}

impl HistoryView {
    pub fn new() -> Self {
        Self {
            filter: Filter::All,
        }
    }

    pub fn update(&mut self, message: HistoryMessage) -> Option<HistoryEvent> {
        match message {
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
            HistoryMessage::CopyTxid(_) => None, // Handled by clipboard action
            HistoryMessage::OpenExplorer(_, _) => None, // Handled by open URL action
        }
    }

    pub fn view(&self, wallet: Option<&Wallet>) -> Element<'_, HistoryMessage> {
        let title = text(t("Lịch sử giao dịch", "Transaction History"))
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let mut content = column![title].spacing(16).padding(32);

        // Filter buttons
        let filter_row = row![
            button(text(t("Tất cả", "All")).size(12))
                .on_press(HistoryMessage::FilterAll)
                .padding(8)
                .style(if self.filter == Filter::All {
                    primary_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(8),
            button(text(t("Nhận", "Incoming")).size(12))
                .on_press(HistoryMessage::FilterIncoming)
                .padding(8)
                .style(if self.filter == Filter::Incoming {
                    primary_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(8),
            button(text(t("Gửi", "Outgoing")).size(12))
                .on_press(HistoryMessage::FilterOutgoing)
                .padding(8)
                .style(if self.filter == Filter::Outgoing {
                    primary_button_style()
                } else {
                    secondary_button_style()
                }),
            Space::with_width(Length::Fill),
            button(text(format!("🔄 {}", t("Làm mới", "Refresh"))).size(12))
                .on_press(HistoryMessage::Refresh)
                .padding(8)
                .style(secondary_button_style()),
        ];

        content = content.push(filter_row);

        if let Some(wallet) = wallet {
            let filtered_txs: Vec<&TxRecord> = wallet
                .history
                .iter()
                .filter(|tx| match self.filter {
                    Filter::All => true,
                    Filter::Incoming => matches!(tx.direction, TxDirection::Incoming),
                    Filter::Outgoing => matches!(tx.direction, TxDirection::Outgoing),
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
                    let amount_btc = tx.amount_sat as f64 / 100_000_000.0;
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
                                    .on_press(HistoryMessage::OpenExplorer(explorer_url.clone(), wallet.network))
                                    .padding(8)
                                    .style(secondary_button_style()),
                                Space::with_width(Length::Fill),
                                text(format!("{}{}", amount_sign, format_btc_and_sat(tx.amount_sat)))
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
                                Space::with_width(16),
                                if let Some(fee) = tx.fee_sat {
                                    text(format!("{}: {}", t("Phí", "Fee"), format_btc_and_sat(fee as i64)))
                                        .size(12)
                                        .style(text_color(Colors::TEXT_MUTED))
                                } else {
                                    text("")
                                },
                                Space::with_width(Length::Fill),
                                if let Some(block_time) = tx.block_time {
                                    text(format_timestamp(block_time))
                                        .size(12)
                                        .style(text_color(Colors::TEXT_MUTED))
                                } else {
                                    text("")
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

                content = content.push(
                    scrollable(tx_list)
                        .height(Length::Fixed(400.0))
                );
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
