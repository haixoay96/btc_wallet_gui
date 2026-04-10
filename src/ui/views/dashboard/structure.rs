use crate::ui::components::backup_reminder::BackupReminderMessage;
use crate::ui::components::network_status::{DashboardNetworkMessage, NetworkStatus};
use crate::ui::components::sparkline::BalancePoint;
use crate::ui::views::sidebar::NavItem;
use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub enum DashboardMessage {
    Refresh,
    Navigate(NavItem),
    Network(DashboardNetworkMessage),
    BackupReminder(BackupReminderMessage),
}

/// Aggregated recent transaction for dashboard preview
#[derive(Debug, Clone)]
pub struct RecentTxItem {
    pub txid: String,
    pub amount_sat: i64,
    #[allow(dead_code)]
    pub confirmed: bool,
    pub block_time: Option<u64>,
    pub wallet_name: String,
}

impl RecentTxItem {
    pub fn formatted_amount(&self) -> String {
        let sign = if self.amount_sat >= 0 { "+" } else { "" };
        let abs = self.amount_sat.unsigned_abs();
        let btc = abs as f64 / 100_000_000.0;
        format!("{}{:.8} BTC", sign, btc)
    }

    pub fn time_ago(&self) -> String {
        match self.block_time {
            Some(ts) => {
                let dt = DateTime::from_timestamp(ts as i64, 0).unwrap_or_default();
                let now = Local::now();
                let duration = now.signed_duration_since(dt);
                if duration.num_days() > 0 {
                    format!("{}d ago", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{}h ago", duration.num_hours())
                } else if duration.num_minutes() > 0 {
                    format!("{}m ago", duration.num_minutes())
                } else {
                    "Just now".to_string()
                }
            }
            None => "Pending".to_string(),
        }
    }

    pub fn is_incoming(&self) -> bool {
        self.amount_sat > 0
    }
}

pub struct DashboardView {
    pub total_balance: i64,
    pub confirmed_balance: i64,
    pub pending_balance: i64,
    pub wallet_count: usize,
    pub backup_needed_wallets: usize,
    pub last_synced_label: Option<String>,
    pub network_status: NetworkStatus,
    pub recent_transactions: Vec<RecentTxItem>,
    pub show_backup_reminder: bool,
    pub balance_history: Vec<BalancePoint>,
}
