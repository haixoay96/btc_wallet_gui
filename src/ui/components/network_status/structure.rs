/// Network connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkStatus {
    Connected { block_height: u32 },
    Disconnected,
    Checking,
}

/// Message for network status interactions
#[derive(Debug, Clone)]
pub enum DashboardNetworkMessage {
    CheckConnection,
    ConnectionCheckResult(Result<u32, String>),
}
