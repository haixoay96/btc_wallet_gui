use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use bip39::{Language, Mnemonic};
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::key::Secp256k1;
use sssmc39::{combine_mnemonics, generate_mnemonics};

use super::network::WalletNetwork;
use super::structure::Wallet;
use crate::wallet::secrets::{WalletBundle, WalletSecrets};
use crate::wallet::DEFAULT_GAP_LIMIT;

// ─── Wallet generation / import ──────────────────────────────────────────

impl Wallet {
    /// Generate a new wallet with random mnemonic
    pub fn generate(name: &str, network: WalletNetwork) -> Result<WalletBundle> {
        let mnemonic = Mnemonic::generate_in(Language::English, 12)?;
        Self::create_wallet_from_mnemonic(name, network, mnemonic, false)
    }

    /// Import wallet from BIP39 mnemonic phrase
    pub fn from_mnemonic(
        name: &str,
        network: WalletNetwork,
        mnemonic_phrase: &str,
    ) -> Result<WalletBundle> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
            .context("Mnemonic không hợp lệ")?;
        Self::create_wallet_from_mnemonic(name, network, mnemonic, true)
    }

    /// Import wallet from SLIP39 shares
    pub fn from_slip39_shares(
        name: &str,
        network: WalletNetwork,
        share_phrases: &[String],
        slip39_passphrase: &str,
    ) -> Result<WalletBundle> {
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

    /// Split BIP39 mnemonic into SLIP39 shares
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

    // ─── Private helpers ──────────────────────────────────────────────

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

    fn create_wallet_from_mnemonic(
        name: &str,
        network: WalletNetwork,
        mnemonic: Mnemonic,
        mnemonic_backed_up: bool,
    ) -> Result<WalletBundle> {
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
        let mnemonic_phrase = mnemonic.to_string();
        let mnemonic_word_count = mnemonic_phrase.split_whitespace().count();

        let mut wallet = Wallet {
            name: name.trim().to_string(),
            network,
            mnemonic_backed_up,
            has_mnemonic: true,
            mnemonic_word_count: Some(mnemonic_word_count),
            account_xpub: account_xpub.to_string(),
            next_external_index: 0,
            next_internal_index: 0,
            addresses: Vec::new(),
            history: Vec::new(),
        };

        let secrets = WalletSecrets::new(Some(mnemonic_phrase), account_xprv.to_string());
        wallet.derive_next_addresses(&secrets, DEFAULT_GAP_LIMIT)?;
        Ok(WalletBundle::new(wallet, secrets))
    }

    pub(super) fn parse_account_xprv(secrets: &WalletSecrets) -> Result<Xpriv> {
        Xpriv::from_str(secrets.account_xprv()).context("account_xprv không hợp lệ")
    }
}
