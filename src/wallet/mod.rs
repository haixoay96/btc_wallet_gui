use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use bip39::{Language, Mnemonic};
use bitcoin::{
    absolute,
    bip32::{ChildNumber, DerivationPath, Xpriv, Xpub},
    consensus,
    key::Secp256k1,
    sighash::{EcdsaSighashType, SighashCache},
    transaction::Version,
    Address, Amount, CompressedPublicKey, Network, OutPoint, PrivateKey, ScriptBuf, Sequence,
    Transaction, TxIn, TxOut, Txid, Witness,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

mod address;
mod history;
mod tx;

impl Wallet {
    pub fn new(name: &str, network: WalletNetwork) -> Result<Self> {
        address::create_new_wallet(name, network)
    }

    pub fn from_mnemonic(
        name: &str,
        network: WalletNetwork,
        mnemonic_phrase: &str,
    ) -> Result<Self> {
        address::import_wallet_from_mnemonic(name, network, mnemonic_phrase)
    }

    pub fn from_slip39_shares(
        name: &str,
        network: WalletNetwork,
        share_phrases: &[String],
        slip39_passphrase: &str,
    ) -> Result<Self> {
        address::import_wallet_from_slip39_shares(name, network, share_phrases, slip39_passphrase)
    }

    pub fn split_mnemonic_to_slip39_shares(
        mnemonic_phrase: &str,
        threshold: u8,
        share_count: u8,
        slip39_passphrase: &str,
    ) -> Result<Vec<String>> {
        address::split_mnemonic_to_slip39_shares(
            mnemonic_phrase,
            threshold,
            share_count,
            slip39_passphrase,
        )
    }

    pub fn from_account_xprv(
        name: &str,
        network: WalletNetwork,
        account_xprv: &str,
    ) -> Result<Self> {
        address::import_wallet_from_account_xprv(name, network, account_xprv)
    }

    pub fn derive_next_addresses(&mut self, count: u32) -> Result<Vec<String>> {
        address::derive_next_addresses(&mut self.entry, count)
    }

    pub fn refresh_history(&mut self) -> Result<usize> {
        history::refresh_history(&mut self.entry)
    }

    pub fn create_transaction_with_options(
        &mut self,
        to_address: &str,
        amount_sat: u64,
        fee_sat: u64,
        options: TxBuildOptions,
    ) -> Result<BuildTxResult> {
        tx::create_transaction_with_options(
            &mut self.entry,
            to_address,
            amount_sat,
            fee_sat,
            options,
        )
    }

    pub fn estimate_auto_fee_for_amount(
        &self,
        amount_sat: u64,
        input_source: &InputSource,
    ) -> Result<u64> {
        tx::estimate_auto_fee_for_amount(&self.entry, amount_sat, input_source)
    }

    pub fn create_send_all_transaction_with_options(
        &mut self,
        to_address: &str,
        fee_mode: FeeMode,
        options: TxBuildOptions,
    ) -> Result<BuildTxResult> {
        tx::create_send_all_transaction_with_options(&mut self.entry, to_address, fee_mode, options)
    }

    pub fn balance(&self) -> i64 {
        self.entry.history.iter().map(|tx| tx.amount_sat).sum()
    }

    pub fn confirmed_balance(&self) -> i64 {
        self.entry
            .history
            .iter()
            .filter(|tx| tx.confirmed)
            .map(|tx| tx.amount_sat)
            .sum()
    }
}

const DEFAULT_GAP_LIMIT: u32 = 5;
const DUST_LIMIT_SAT: u64 = 546;
const DEFAULT_AUTO_FEE_RATE_SAT_VB: f64 = 2.0;
const ESTIMATE_OVERHEAD_VB: u64 = 10;
const ESTIMATE_P2WPKH_INPUT_VB: u64 = 68;
const ESTIMATE_P2WPKH_OUTPUT_VB: u64 = 31;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WalletNetwork {
    Mainnet,
    Testnet,
}

impl WalletNetwork {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
        }
    }

    pub fn coin_type(self) -> u32 {
        match self {
            Self::Mainnet => 0,
            Self::Testnet => 1,
        }
    }

    pub fn bitcoin_network(self) -> Network {
        match self {
            Self::Mainnet => Network::Bitcoin,
            Self::Testnet => Network::Testnet,
        }
    }

    pub fn blockstream_base_url(self) -> &'static str {
        match self {
            Self::Mainnet => "https://blockstream.info/api",
            Self::Testnet => "https://blockstream.info/testnet/api",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "mainnet" | "main" | "bitcoin" | "btc" => Ok(Self::Mainnet),
            "testnet" | "test" | "tb" => Ok(Self::Testnet),
            _ => Err(anyhow!(
                "Network không hợp lệ: {value}. Dùng mainnet hoặc testnet"
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressEntry {
    pub index: u32,
    pub address: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxDirection {
    Incoming,
    Outgoing,
    SelfTransfer,
}

impl TxDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Incoming => "in",
            Self::Outgoing => "out",
            Self::SelfTransfer => "self",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxRecord {
    pub txid: String,
    pub direction: TxDirection,
    pub amount_sat: i64,
    pub fee_sat: Option<u64>,
    pub confirmed: bool,
    pub block_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEntry {
    pub name: String,
    pub network: WalletNetwork,
    pub mnemonic: Option<String>,
    #[serde(default)]
    pub mnemonic_backed_up: bool,
    pub account_xprv: String,
    pub account_xpub: String,
    pub next_index: u32,
    pub addresses: Vec<AddressEntry>,
    pub history: Vec<TxRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub entry: WalletEntry,
}

#[derive(Debug, Clone)]
pub struct BuildTxResult {
    pub raw_hex: String,
    pub txid: String,
    pub broadcasted: bool,
}

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

#[derive(Debug, Clone, Copy)]
pub enum FeeMode {
    FixedSat(u64),
    Auto,
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
struct SpendableUtxo {
    txid: Txid,
    vout: u32,
    value: u64,
    address_index: u32,
    address: Address,
}

#[derive(Debug, Deserialize)]
struct ApiAddressUtxo {
    txid: String,
    vout: u32,
    value: u64,
}

#[derive(Debug, Deserialize)]
struct ApiTxStatus {
    confirmed: bool,
    block_time: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ApiPrevout {
    #[serde(default)]
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Debug, Deserialize)]
struct ApiVin {
    #[serde(default)]
    prevout: Option<ApiPrevout>,
}

#[derive(Debug, Deserialize)]
struct ApiVout {
    #[serde(default)]
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Debug, Deserialize)]
struct ApiTx {
    txid: String,
    vin: Vec<ApiVin>,
    vout: Vec<ApiVout>,
    #[serde(default)]
    fee: Option<u64>,
    status: ApiTxStatus,
}
