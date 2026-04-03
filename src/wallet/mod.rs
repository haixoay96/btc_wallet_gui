mod api_types;
mod structure;

pub use structure::{
    ChangeStrategy, InputSource, TxBuildOptions,
    TxDirection, TxRecord, Wallet, WalletNetwork,
};


pub(super) const DEFAULT_GAP_LIMIT: u32 = 5;
pub(super) const DUST_LIMIT_SAT: u64 = 546;
pub(super) const DEFAULT_AUTO_FEE_RATE_SAT_VB: f64 = 2.0;
pub(super) const ESTIMATE_OVERHEAD_VB: u64 = 10;
pub(super) const ESTIMATE_P2WPKH_INPUT_VB: u64 = 68;
pub(super) const ESTIMATE_P2WPKH_OUTPUT_VB: u64 = 31;
