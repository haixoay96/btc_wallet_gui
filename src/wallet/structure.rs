use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use bip39::{Language, Mnemonic};
use bitcoin::{
    bip32::{ChildNumber, DerivationPath, Xpriv, Xpub},
    key::Secp256k1,
    Address, CompressedPublicKey, PrivateKey,
};
use serde::{Deserialize, Serialize};
use sssmc39::{combine_mnemonics, generate_mnemonics};

use super::build_tx_result::BuildTxResult;
use super::fee_mode::FeeMode;
use super::history;
use super::input_source::InputSource;
use super::tx;
use super::tx_build_options::TxBuildOptions;
use super::wallet_network::WalletNetwork;
use super::DEFAULT_GAP_LIMIT;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressEntry {
    pub index: u32,
    pub address: String,
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
pub struct Wallet {
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

impl Wallet {
    pub fn new(name: &str, network: WalletNetwork) -> Result<Self> {
        let mnemonic = Mnemonic::generate_in(Language::English, 12)?;
        Self::create_wallet_from_mnemonic(name, network, mnemonic, false)
    }

    pub fn from_mnemonic(
        name: &str,
        network: WalletNetwork,
        mnemonic_phrase: &str,
    ) -> Result<Self> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
            .context("Mnemonic không hợp lệ")?;
        Self::create_wallet_from_mnemonic(name, network, mnemonic, true)
    }

    pub fn from_slip39_shares(
        name: &str,
        network: WalletNetwork,
        share_phrases: &[String],
        slip39_passphrase: &str,
    ) -> Result<Self> {
        if share_phrases.is_empty() {
            return Err(anyhow!("Vui lòng nhập ít nhất một SLIP-0039 share"));
        }

        let parsed_shares = Self::parse_slip39_shares(share_phrases)?;
        let entropy = combine_mnemonics(&parsed_shares, slip39_passphrase)
            .map_err(|err| anyhow!("Không thể khôi phục SLIP-0039 shares: {err}"))?;

        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
            .context("Entropy khôi phục từ SLIP-0039 không hợp lệ với BIP39")?;

        Self::create_wallet_from_mnemonic(name, network, mnemonic, true)
    }

    pub fn split_mnemonic_to_slip39_shares(
        mnemonic_phrase: &str,
        threshold: u8,
        share_count: u8,
        slip39_passphrase: &str,
    ) -> Result<Vec<String>> {
        if threshold == 0 {
            return Err(anyhow!("Ngưỡng K phải >= 1"));
        }

        if share_count < threshold {
            return Err(anyhow!("Tổng số share N phải >= ngưỡng K"));
        }

        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
            .context("Mnemonic không hợp lệ")?;
        let entropy = mnemonic.to_entropy();

        let groups = [(threshold, share_count)];
        let generated = generate_mnemonics(1, &groups, &entropy, slip39_passphrase, 0)
            .map_err(|err| anyhow!("Không thể tạo SLIP-0039 shares: {err}"))?;

        let group = generated
            .first()
            .ok_or_else(|| anyhow!("Không tạo được group share SLIP-0039"))?;

        group
            .member_shares
            .iter()
            .map(|share| {
                share
                    .to_mnemonic()
                    .map(|words| words.join(" "))
                    .map_err(|err| anyhow!("Không thể encode SLIP-0039 share: {err}"))
            })
            .collect()
    }

    fn parse_slip39_shares(share_phrases: &[String]) -> Result<Vec<Vec<String>>> {
        let mut shares = Vec::with_capacity(share_phrases.len());
        for (index, phrase) in share_phrases.iter().enumerate() {
            let normalized = phrase.trim();
            let phrase_body = normalized
                .split_once(':')
                .and_then(|(prefix, rest)| {
                    if prefix.trim().to_ascii_lowercase().starts_with("share_") {
                        Some(rest.trim())
                    } else {
                        None
                    }
                })
                .unwrap_or(normalized);

            let words = phrase_body
                .split_whitespace()
                .map(|word| word.trim().to_ascii_lowercase())
                .filter(|word| !word.is_empty())
                .collect::<Vec<_>>();

            if words.is_empty() {
                return Err(anyhow!("SLIP-0039 share #{} đang để trống", index + 1));
            }

            shares.push(words);
        }

        Ok(shares)
    }

    pub fn from_account_xprv(
        name: &str,
        network: WalletNetwork,
        account_xprv: &str,
    ) -> Result<Self> {
        let secp = Secp256k1::new();
        let parsed_xprv = Xpriv::from_str(account_xprv).context("xprv không hợp lệ")?;
        let account_xpub = Xpub::from_priv(&secp, &parsed_xprv);

        let mut wallet = Wallet {
            name: name.trim().to_string(),
            network,
            mnemonic: None,
            mnemonic_backed_up: true,
            account_xprv: parsed_xprv.to_string(),
            account_xpub: account_xpub.to_string(),
            next_index: 0,
            addresses: Vec::new(),
            history: Vec::new(),
        };

        wallet.derive_next_addresses(DEFAULT_GAP_LIMIT)?;
        Ok(wallet)
    }

    fn create_wallet_from_mnemonic(
        name: &str,
        network: WalletNetwork,
        mnemonic: Mnemonic,
        mnemonic_backed_up: bool,
    ) -> Result<Wallet> {
        let secp = Secp256k1::new();
        let seed = mnemonic.to_seed_normalized("");
        let root_xprv = Xpriv::new_master(network.bitcoin_network(), &seed)?;

        let account_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(84)?,
            ChildNumber::from_hardened_idx(network.coin_type())?,
            ChildNumber::from_hardened_idx(0)?,
        ]);

        let account_xprv = root_xprv.derive_priv(&secp, &account_path)?;
        let account_xpub = Xpub::from_priv(&secp, &account_xprv);

        let mut wallet = Wallet {
            name: name.trim().to_string(),
            network,
            mnemonic: Some(mnemonic.to_string()),
            mnemonic_backed_up,
            account_xprv: account_xprv.to_string(),
            account_xpub: account_xpub.to_string(),
            next_index: 0,
            addresses: Vec::new(),
            history: Vec::new(),
        };

        wallet.derive_next_addresses(DEFAULT_GAP_LIMIT)?;
        Ok(wallet)
    }

    pub fn derive_next_addresses(&mut self, count: u32) -> Result<Vec<String>> {
        let secp = Secp256k1::new();
        let account_xprv = Xpriv::from_str(&self.account_xprv)?;

        let mut new_addresses = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let index = self.next_index;
            let (address, _, _) = Self::derive_address_and_keys(&secp, &account_xprv, self.network, index)?;

            self.addresses.push(AddressEntry {
                index,
                address: address.to_string(),
            });
            self.next_index += 1;
            new_addresses.push(address.to_string());
        }

        Ok(new_addresses)
    }

    pub(super) fn derive_address_and_keys(
        secp: &Secp256k1<bitcoin::secp256k1::All>,
        account_xprv: &Xpriv,
        network: WalletNetwork,
        index: u32,
    ) -> Result<(Address, PrivateKey, bitcoin::PublicKey)> {
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_normal_idx(0)?,
            ChildNumber::from_normal_idx(index)?,
        ]);

        let child_xprv = account_xprv.derive_priv(secp, &derivation_path)?;
        let private_key = PrivateKey::new(child_xprv.private_key, network.bitcoin_network());
        let public_key = private_key.public_key(secp);
        let compressed_pubkey = CompressedPublicKey::from_slice(&public_key.to_bytes())?;
        let address = Address::p2wpkh(&compressed_pubkey, network.bitcoin_network());

        Ok((address, private_key, public_key))
    }

    pub fn refresh_history(&mut self) -> Result<usize> {
        history::refresh_history(self)
    }

    pub fn create_transaction_with_options(
        &mut self,
        to_address: &str,
        amount_sat: u64,
        fee_sat: u64,
        options: TxBuildOptions,
    ) -> Result<BuildTxResult> {
        tx::create_transaction_with_options(
            self,
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
        tx::estimate_auto_fee_for_amount(self, amount_sat, input_source)
    }

    pub fn create_send_all_transaction_with_options(
        &mut self,
        to_address: &str,
        fee_mode: FeeMode,
        options: TxBuildOptions,
    ) -> Result<BuildTxResult> {
        tx::create_send_all_transaction_with_options(self, to_address, fee_mode, options)
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