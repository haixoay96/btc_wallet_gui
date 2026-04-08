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

// Re-export constants từ shared/
pub use crate::shared::constants::{
    DEFAULT_GAP_LIMIT, DUST_LIMIT_SAT, DEFAULT_AUTO_FEE_RATE_SAT_VB,
    ESTIMATE_OVERHEAD_VB, ESTIMATE_P2WPKH_INPUT_VB, ESTIMATE_P2WPKH_OUTPUT_VB,
};

// Re-export api_types, esplora cho internal use
pub use api_types::*;
pub use esplora::EsploraClient;
