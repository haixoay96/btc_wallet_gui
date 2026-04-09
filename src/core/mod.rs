pub mod contact;
pub mod wallet;

// Re-export wallet network types
pub use wallet::{AddressChain, WalletNetwork};

// Re-export wallet constants
pub use crate::shared::constants::{
    DEFAULT_AUTO_FEE_RATE_SAT_VB, DEFAULT_GAP_LIMIT, DUST_LIMIT_SAT, ESTIMATE_OVERHEAD_VB,
    ESTIMATE_P2WPKH_INPUT_VB, ESTIMATE_P2WPKH_OUTPUT_VB,
};

// Re-export wallet secrets
pub use wallet::secrets::{
    runtime_wallets_from_stored, stored_wallets_from_runtime, StoredWallet, WalletSecretsRef,
    WalletSecretsVault,
};

// Re-export wallet validation
pub use wallet::validation::{validate_address_for_network, validate_bitcoin_address};

// Re-export wallet structure types
pub use wallet::structure::{
    AddressEntry, BuildTxResult, ChangeStrategy, InputSource, SpendableUtxo, TxBuildOptions,
    TxDirection, TxRecord, Wallet,
};

// Re-export network types
pub use crate::infra::network::{ApiAddressUtxo, ApiTx, EsploraClient};
