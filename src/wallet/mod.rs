mod api_types;
mod build_tx_result;
mod change_strategy;
mod fee_mode;
mod history;
mod input_source;
mod spendable_utxo;
mod structure;
mod tx;
mod tx_build_options;
mod wallet_network;

pub use change_strategy::ChangeStrategy;
pub use fee_mode::FeeMode;
pub use input_source::InputSource;
pub use structure::{TxDirection, TxRecord, Wallet};

// Backward compatibility alias
pub use tx_build_options::TxBuildOptions;
pub use wallet_network::WalletNetwork;


pub(super) const DEFAULT_GAP_LIMIT: u32 = 5;
pub(super) const DUST_LIMIT_SAT: u64 = 546;
pub(super) const DEFAULT_AUTO_FEE_RATE_SAT_VB: f64 = 2.0;
pub(super) const ESTIMATE_OVERHEAD_VB: u64 = 10;
pub(super) const ESTIMATE_P2WPKH_INPUT_VB: u64 = 68;
pub(super) const ESTIMATE_P2WPKH_OUTPUT_VB: u64 = 31;
