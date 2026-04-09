use crate::ui::components::network_status::{DashboardNetworkMessage, NetworkStatus};
use crate::ui::views::sidebar::NavItem;

#[derive(Debug, Clone)]
pub enum DashboardMessage {
    Refresh,
    Navigate(NavItem),
    Network(DashboardNetworkMessage),
}

pub struct DashboardView {
    pub total_balance: i64,
    pub confirmed_balance: i64,
    pub pending_balance: i64,
    pub wallet_count: usize,
    pub backup_needed_wallets: usize,
    pub last_synced_label: Option<String>,
    pub network_status: NetworkStatus,
}
