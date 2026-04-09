pub mod derivation;
pub mod generator;
pub mod network;
pub mod secrets;
pub mod structure;
pub mod sync;
pub mod transaction;
pub mod validation;

pub use network::{AddressChain, WalletNetwork};
pub use secrets::{
    runtime_wallets_from_stored, stored_wallets_from_runtime, StoredWallet, WalletSecretsRef,
    WalletSecretsVault,
};
pub use structure::{ChangeStrategy, InputSource, TxBuildOptions, TxDirection, TxRecord, Wallet};
pub use validation::{validate_address_for_network, validate_bitcoin_address};

// Re-export constants from shared
pub use crate::shared::constants::{
    DEFAULT_AUTO_FEE_RATE_SAT_VB, DEFAULT_GAP_LIMIT, DUST_LIMIT_SAT, ESTIMATE_OVERHEAD_VB,
    ESTIMATE_P2WPKH_INPUT_VB, ESTIMATE_P2WPKH_OUTPUT_VB,
};

// Re-export EsploraClient from infra/network for backward compatibility
