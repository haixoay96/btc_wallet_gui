use bitcoin::{Address, Txid};
use serde::{Deserialize, Serialize};

use super::network::{AddressChain, WalletNetwork};
use crate::core::wallet::secrets::WalletSecrets;

// ─── Enums ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum InputSource {
    All,
    AddressIndexes(Vec<u32>),
}

#[derive(Debug, Clone)]
pub enum ChangeStrategy {
    NewAddress,
    ExistingIndex(u32),
}

#[derive(Debug, Clone)]
pub struct TxBuildOptions {
    pub broadcast: bool,
    pub input_source: InputSource,
    pub change_strategy: ChangeStrategy,
}

impl Default for TxBuildOptions {
    fn default() -> Self {
        Self {
            broadcast: false,
            input_source: InputSource::All,
            change_strategy: ChangeStrategy::NewAddress,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuildTxResult {
    pub txid: String,
    pub broadcasted: bool,
}

#[derive(Debug, Clone)]
pub struct SpendableUtxo {
    pub txid: Txid,
    pub vout: u32,
    pub value: u64,
    pub address_index: u32,
    pub chain: AddressChain,
    pub address: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressEntry {
    pub index: u32,
    pub address: String,
    #[serde(default)]
    pub chain: AddressChain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxRecord {
    pub txid: String,
    pub direction: TxDirection,
    pub amount_sat: i64,
    pub fee_sat: Option<u64>,
    pub confirmed: bool,
    pub block_time: Option<u64>,
    #[serde(default)]
    pub confirmations: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxDirection {
    Incoming,
    Outgoing,
    SelfTransfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub name: String,
    pub network: WalletNetwork,
    #[serde(default)]
    pub mnemonic_backed_up: bool,
    #[serde(default)]
    pub has_mnemonic: bool,
    #[serde(default)]
    pub mnemonic_word_count: Option<usize>,
    pub account_xpub: String,
    #[serde(default, alias = "next_index")]
    pub next_external_index: u32,
    #[serde(default)]
    pub next_internal_index: u32,
    pub addresses: Vec<AddressEntry>,
    pub history: Vec<TxRecord>,
    #[serde(default)]
    pub tags: Vec<String>,
}

// ─── Impl Wallet (simple methods) ────────────────────────────────────────

impl Wallet {
    pub fn wallet_id(&self) -> &str {
        &self.account_xpub
    }

    pub fn sync_secret_metadata(&mut self, secrets: &WalletSecrets) {
        self.has_mnemonic = secrets.has_mnemonic();
        self.mnemonic_word_count = secrets.mnemonic_word_count();
    }

    pub fn balance(&self) -> i64 {
        self.history.iter().map(|tx| tx.amount_sat).sum()
    }

    pub fn confirmed_balance(&self) -> i64 {
        self.history
            .iter()
            .filter(|tx| tx.confirmed)
            .map(|tx| tx.amount_sat)
            .sum()
    }
}
