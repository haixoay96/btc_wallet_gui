// Re-export từ core/wallet để backward compatibility
pub use crate::core::wallet::network::{AddressChain, WalletNetwork};
pub use crate::core::wallet::structure::*;
pub use crate::core::wallet::validation::{validate_address_for_network, validate_bitcoin_address};

// Re-export secrets module (vẫn ở vị trí cũ)
pub mod secrets;
pub use secrets::{
    runtime_wallets_from_stored, stored_wallets_from_runtime, StoredWallet, WalletSecretsRef,
    WalletSecretsVault,
};

// Re-export esplora và api_types (sẽ chuyển sang infra/network ở phase sau)
pub mod esplora;
pub mod api_types;

// Constants
pub const DEFAULT_GAP_LIMIT: u32 = 5;
pub const DUST_LIMIT_SAT: u64 = 546;
pub const DEFAULT_AUTO_FEE_RATE_SAT_VB: f64 = 2.0;
pub const ESTIMATE_OVERHEAD_VB: u64 = 10;
pub const ESTIMATE_P2WPKH_INPUT_VB: u64 = 68;
pub const ESTIMATE_P2WPKH_OUTPUT_VB: u64 = 31;

// Re-export api_types, esplora cho internal use
pub use api_types::*;
pub use esplora::EsploraClient;
