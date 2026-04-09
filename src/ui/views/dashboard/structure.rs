use crate::ui::views::sidebar::NavItem;

#[derive(Debug, Clone)]
pub enum DashboardMessage {
    Refresh,
    Navigate(NavItem),
}

pub struct DashboardView {
    pub total_balance: i64,
    pub confirmed_balance: i64,
    pub pending_balance: i64,
    pub wallet_count: usize,
    pub backup_needed_wallets: usize,
    pub last_synced_label: Option<String>,
}
