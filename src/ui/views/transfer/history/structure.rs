use crate::wallet::{TxDirection, TxRecord};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Filter {
    All,
    Incoming,
    Outgoing,
    Pending,
    SelfTransfer,
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
    DateFromChanged(String),
    DateToChanged(String),
    MinAmountChanged(String),
    MaxAmountChanged(String),
    PageChanged(usize),
    ItemsPerPageChanged(usize),
    ViewTransaction(usize),
    CloseTransactionDetail,
}

pub enum HistoryEvent {
    Refresh,
    ExportCsv,
    ExportPdf,
}

pub struct HistoryView {
    pub filter: Filter,
    pub search_query: String,
    pub date_from: String,
    pub date_to: String,
    pub min_amount: String,
    pub max_amount: String,
    pub current_page: usize,
    pub items_per_page: usize,
    pub selected_tx_index: Option<usize>,
}
