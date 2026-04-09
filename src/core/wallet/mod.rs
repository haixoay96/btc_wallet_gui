pub mod derivation;
pub mod generator;
pub mod network;
pub mod structure;
pub mod sync;
pub mod transaction;
pub mod validation;

pub use network::{AddressChain, WalletNetwork};
pub use structure::{
    AddressEntry, BuildTxResult, ChangeStrategy, InputSource, SpendableUtxo, TxBuildOptions,
    TxDirection, TxRecord, Wallet,
};
pub use validation::{validate_address_for_network, validate_bitcoin_address};
