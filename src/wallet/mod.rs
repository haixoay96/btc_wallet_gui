mod api_types;
mod esplora;
mod secrets;
mod structure;
mod validation;

pub use secrets::{
    runtime_wallets_from_stored, stored_wallets_from_runtime, StoredWallet, WalletSecretsRef,
    WalletSecretsVault,
};
pub use structure::{
    AddressChain, ChangeStrategy, InputSource, TxBuildOptions, TxDirection, TxRecord, Wallet,
    WalletNetwork,
};
pub use validation::{validate_address_for_network, validate_bitcoin_address};

pub(super) const DEFAULT_GAP_LIMIT: u32 = 5;
pub(super) const DUST_LIMIT_SAT: u64 = 546;
pub(super) const DEFAULT_AUTO_FEE_RATE_SAT_VB: f64 = 2.0;
pub(super) const ESTIMATE_OVERHEAD_VB: u64 = 10;
pub(super) const ESTIMATE_P2WPKH_INPUT_VB: u64 = 68;
pub(super) const ESTIMATE_P2WPKH_OUTPUT_VB: u64 = 31;
