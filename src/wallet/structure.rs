use std::collections::{HashMap, HashSet};
use std::str::FromStr;

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
use sssmc39::{combine_mnemonics, generate_mnemonics};

use super::api_types::{ApiAddressUtxo, ApiTx};
use super::{
    DEFAULT_AUTO_FEE_RATE_SAT_VB, DEFAULT_GAP_LIMIT, DUST_LIMIT_SAT, ESTIMATE_OVERHEAD_VB,
    ESTIMATE_P2WPKH_INPUT_VB, ESTIMATE_P2WPKH_OUTPUT_VB,
};

#[derive(Debug, Clone, Copy)]
pub enum FeeMode {
    FixedSat(u64),
    Auto,
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
    pub raw_hex: String,
    pub txid: String,
    pub broadcasted: bool,
}

#[derive(Debug, Clone)]
pub struct SpendableUtxo {
    pub txid: Txid,
    pub vout: u32,
    pub value: u64,
    pub address_index: u32,
    pub address: Address,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WalletNetwork {
    Mainnet,
    Testnet,
}


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
        if self.addresses.is_empty() {
            self.derive_next_addresses(DEFAULT_GAP_LIMIT)?;
        }

        let client = Client::new();
        let base_url = self.network.blockstream_base_url();
        let own_addresses: HashSet<String> =
            self.addresses.iter().map(|a| a.address.clone()).collect();

        let mut tx_map: HashMap<String, TxRecord> = HashMap::new();

        for addr in &self.addresses {
            let url = format!("{base_url}/address/{}/txs", addr.address);
            let txs: Vec<ApiTx> = client
                .get(&url)
                .send()
                .with_context(|| format!("Không gọi được API lịch sử: {url}"))?
                .error_for_status()
                .with_context(|| format!("Lỗi response API lịch sử: {url}"))?
                .json()
                .with_context(|| format!("Không parse được dữ liệu lịch sử của {}", addr.address))?;

            for tx in txs {
                let received: u64 = tx
                    .vout
                    .iter()
                    .filter_map(|v| {
                        let address = v.scriptpubkey_address.as_ref()?;
                        if own_addresses.contains(address) {
                            Some(v.value)
                        } else {
                            None
                        }
                    })
                    .sum();

                let spent: u64 = tx
                    .vin
                    .iter()
                    .filter_map(|vin| vin.prevout.as_ref())
                    .filter_map(|prevout| {
                        let address = prevout.scriptpubkey_address.as_ref()?;
                        if own_addresses.contains(address) {
                            Some(prevout.value)
                        } else {
                            None
                        }
                    })
                    .sum();

                let net = i128::from(received) - i128::from(spent);
                let amount_sat = i64::try_from(net).unwrap_or(if net.is_negative() {
                    i64::MIN
                } else {
                    i64::MAX
                });
                let direction = if amount_sat > 0 {
                    TxDirection::Incoming
                } else if amount_sat < 0 {
                    TxDirection::Outgoing
                } else {
                    TxDirection::SelfTransfer
                };

                tx_map.insert(
                    tx.txid.clone(),
                    TxRecord {
                        txid: tx.txid,
                        direction,
                        amount_sat,
                        fee_sat: tx.fee,
                        confirmed: tx.status.confirmed,
                        block_time: tx.status.block_time,
                    },
                );
            }
        }

        let mut history: Vec<TxRecord> = tx_map.into_values().collect();
        history.sort_by(|a, b| {
            match (a.block_time, b.block_time) {
                (None, None) => a.txid.cmp(&b.txid),
                (None, Some(_)) => std::cmp::Ordering::Less, // a (None) comes first
                (Some(_), None) => std::cmp::Ordering::Greater, // b (None) comes first
                (Some(a_time), Some(b_time)) => b_time.cmp(&a_time).then_with(|| a.txid.cmp(&b.txid)),
            }
        });

        let count = history.len();
        self.history = history;
        Ok(count)
    }

    pub fn create_transaction_with_options(
        &mut self,
        to_address: &str,
        amount_sat: u64,
        fee_sat: u64,
        options: TxBuildOptions,
    ) -> Result<BuildTxResult> {
        if amount_sat == 0 {
            return Err(anyhow!("amount_sat phải > 0"));
        }

        if self.addresses.is_empty() {
            self.derive_next_addresses(DEFAULT_GAP_LIMIT)?;
        }

        let unchecked = Address::from_str(to_address).context("Địa chỉ nhận không hợp lệ")?;
        let to_address = unchecked
            .require_network(self.network.bitcoin_network())
            .context("Địa chỉ nhận không đúng network của ví")?;

        let mut utxos = self.collect_spendable_utxos_by_source(&options.input_source)?;
        utxos.sort_by_key(|u| u.value);
        utxos.reverse();

        let selected = self.select_utxos_for_target(&utxos, amount_sat, fee_sat)?;
        let target = amount_sat
            .checked_add(fee_sat)
            .ok_or_else(|| anyhow!("amount + fee bị overflow"))?;
        let total_input = selected.iter().try_fold(0u64, |acc, utxo| {
            acc.checked_add(utxo.value)
                .ok_or_else(|| anyhow!("Tổng UTXO bị overflow"))
        })?;
        let change = total_input - target;

        let mut tx_outs = vec![TxOut {
            value: Amount::from_sat(amount_sat),
            script_pubkey: to_address.script_pubkey(),
        }];

        if change >= DUST_LIMIT_SAT {
            let change_address = match options.change_strategy {
                ChangeStrategy::NewAddress => self.derive_next_addresses(1)?
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow!("Không tạo được change address"))?,
                ChangeStrategy::ExistingIndex(index) => self
                    .addresses
                    .iter()
                    .find(|addr| addr.index == index)
                    .map(|addr| addr.address.clone())
                    .ok_or_else(|| anyhow!("change index {} không tồn tại trong ví", index))?,
            };

            let checked_change =
                Address::from_str(&change_address)?.require_network(self.network.bitcoin_network())?;

            tx_outs.push(TxOut {
                value: Amount::from_sat(change),
                script_pubkey: checked_change.script_pubkey(),
            });
        }

        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: absolute::LockTime::ZERO,
            input: selected
                .iter()
                .map(|utxo| TxIn {
                    previous_output: OutPoint {
                        txid: utxo.txid,
                        vout: utxo.vout,
                    },
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                    witness: Witness::new(),
                })
                .collect(),
            output: tx_outs,
        };

        self.sign_transaction(&selected, &mut tx)?;

        let raw_hex = consensus::encode::serialize_hex(&tx);
        let txid = tx.compute_txid().to_string();

        if options.broadcast {
            self.broadcast_transaction(&raw_hex)?;
        }

        self.history.insert(
            0,
            TxRecord {
                txid: txid.clone(),
                direction: TxDirection::Outgoing,
                amount_sat: -(i64::try_from(amount_sat).unwrap_or(i64::MAX)),
                fee_sat: Some(fee_sat),
                confirmed: false,
                block_time: None,
            },
        );

        Ok(BuildTxResult {
            raw_hex,
            txid,
            broadcasted: options.broadcast,
        })
    }

    pub fn estimate_auto_fee_for_amount(
        &self,
        amount_sat: u64,
        input_source: &InputSource,
    ) -> Result<u64> {
        if amount_sat == 0 {
            return Err(anyhow!("amount_sat phải > 0"));
        }

        let mut utxos = self.collect_spendable_utxos_by_source(input_source)?;
        utxos.sort_by_key(|u| u.value);
        utxos.reverse();

        let mut selected_count = 0usize;
        let mut total_input = 0u64;

        for utxo in utxos {
            selected_count += 1;
            total_input = total_input
                .checked_add(utxo.value)
                .ok_or_else(|| anyhow!("Tổng UTXO bị overflow"))?;

            let fee_no_change = self.estimate_auto_fee_sat(selected_count, 1)?;
            if let Some(target_no_change) = amount_sat.checked_add(fee_no_change) {
                if total_input >= target_no_change {
                    let change_no_change = total_input - target_no_change;
                    if change_no_change < DUST_LIMIT_SAT {
                        return Ok(fee_no_change);
                    }
                }
            }

            let fee_change = self.estimate_auto_fee_sat(selected_count, 2)?;
            if let Some(target_change) = amount_sat.checked_add(fee_change) {
                if total_input >= target_change {
                    let change = total_input - target_change;
                    if change >= DUST_LIMIT_SAT {
                        return Ok(fee_change);
                    }
                }
            }
        }

        Err(anyhow!(
            "Không đủ số dư để estimate fee cho amount hiện tại"
        ))
    }

    pub fn estimate_fee_for_send_all(
        &self,
        input_source: &InputSource,
    ) -> Result<(u64, u64)> {
        let utxos = self.collect_spendable_utxos_by_source(input_source)?;
        
        if utxos.is_empty() {
            return Err(anyhow!("Không có UTXO khả dụng"));
        }

        let total_input: u64 = utxos.iter().try_fold(0u64, |acc, utxo| {
            acc.checked_add(utxo.value)
                .ok_or_else(|| anyhow!("Tổng UTXO bị overflow"))
        })?;

        // Estimate fee for all inputs, 1 output (no change)
        let fee = self.estimate_auto_fee_sat(utxos.len(), 1)?;

        if total_input <= fee {
            return Err(anyhow!(
                "Không đủ số dư để trả phí. Tổng={} sat, phí={} sat",
                total_input,
                fee
            ));
        }

        let max_amount = total_input - fee;
        if max_amount < DUST_LIMIT_SAT {
            return Err(anyhow!(
                "Số tiền gửi sau khi trừ phí quá nhỏ ({} sat)",
                max_amount
            ));
        }

        Ok((max_amount, fee))
    }

    pub fn create_send_all_transaction_with_options(
        &mut self,
        to_address: &str,
        fee_mode: FeeMode,
        options: TxBuildOptions,
    ) -> Result<BuildTxResult> {
        if self.addresses.is_empty() {
            self.derive_next_addresses(DEFAULT_GAP_LIMIT)?;
        }

        let unchecked = Address::from_str(to_address).context("Địa chỉ nhận không hợp lệ")?;
        let to_address = unchecked
            .require_network(self.network.bitcoin_network())
            .context("Địa chỉ nhận không đúng network của ví")?;

        let mut utxos = self.collect_spendable_utxos_by_source(&options.input_source)?;
        utxos.sort_by_key(|u| u.value);
        utxos.reverse();

        if utxos.is_empty() {
            return Err(anyhow!("Không có UTXO để chuyển hết số dư"));
        }

        let total_input = utxos.iter().try_fold(0u64, |acc, utxo| {
            acc.checked_add(utxo.value)
                .ok_or_else(|| anyhow!("Tổng UTXO bị overflow"))
        })?;

        let fee_sat = match fee_mode {
            FeeMode::FixedSat(value) => value,
            FeeMode::Auto => self.estimate_auto_fee_sat(utxos.len(), 1)?,
        };

        if total_input <= fee_sat {
            return Err(anyhow!(
                "Không đủ số dư để trả fee. Tổng={} sat, fee={} sat",
                total_input,
                fee_sat
            ));
        }

        let amount_sat = total_input - fee_sat;
        if amount_sat < DUST_LIMIT_SAT {
            return Err(anyhow!(
                "Số tiền gửi sau khi trừ fee quá nhỏ ({} sat)",
                amount_sat
            ));
        }

        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: absolute::LockTime::ZERO,
            input: utxos
                .iter()
                .map(|utxo| TxIn {
                    previous_output: OutPoint {
                        txid: utxo.txid,
                        vout: utxo.vout,
                    },
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                    witness: Witness::new(),
                })
                .collect(),
            output: vec![TxOut {
                value: Amount::from_sat(amount_sat),
                script_pubkey: to_address.script_pubkey(),
            }],
        };

        self.sign_transaction(&utxos, &mut tx)?;

        let raw_hex = consensus::encode::serialize_hex(&tx);
        let txid = tx.compute_txid().to_string();

        if options.broadcast {
            self.broadcast_transaction(&raw_hex)?;
        }

        self.history.insert(
            0,
            TxRecord {
                txid: txid.clone(),
                direction: TxDirection::Outgoing,
                amount_sat: -(i64::try_from(amount_sat).unwrap_or(i64::MAX)),
                fee_sat: Some(fee_sat),
                confirmed: false,
                block_time: None,
            },
        );

        Ok(BuildTxResult {
            raw_hex,
            txid,
            broadcasted: options.broadcast,
        })
    }

    fn collect_spendable_utxos_by_source(
        &self,
        input_source: &InputSource,
    ) -> Result<Vec<SpendableUtxo>> {
        let known_indices = self
            .addresses
            .iter()
            .map(|addr| addr.index)
            .collect::<HashSet<_>>();

        let mut utxos = self.collect_wallet_utxos()?;

        match input_source {
            InputSource::All => {}
            InputSource::AddressIndexes(indexes) => {
                if indexes.is_empty() {
                    return Err(anyhow!("from list không được rỗng"));
                }

                let selected_indices = indexes.iter().copied().collect::<HashSet<_>>();
                let mut missing = selected_indices
                    .iter()
                    .filter(|idx| !known_indices.contains(idx))
                    .copied()
                    .collect::<Vec<_>>();

                if !missing.is_empty() {
                    missing.sort_unstable();
                    return Err(anyhow!(
                        "from index không tồn tại trong ví: {}",
                        missing
                            .iter()
                            .map(u32::to_string)
                            .collect::<Vec<_>>()
                            .join(",")
                    ));
                }

                utxos.retain(|utxo| selected_indices.contains(&utxo.address_index));

                if utxos.is_empty() {
                    return Err(anyhow!("Không có UTXO khả dụng ở các địa chỉ from đã chọn"));
                }
            }
        }

        Ok(utxos)
    }

    fn select_utxos_for_target(
        &self,
        utxos: &[SpendableUtxo],
        amount_sat: u64,
        fee_sat: u64,
    ) -> Result<Vec<SpendableUtxo>> {
        let target = amount_sat
            .checked_add(fee_sat)
            .ok_or_else(|| anyhow!("amount + fee bị overflow"))?;

        let mut selected = Vec::new();
        let mut total_input = 0u64;

        for utxo in utxos {
            total_input = total_input
                .checked_add(utxo.value)
                .ok_or_else(|| anyhow!("Tổng UTXO bị overflow"))?;
            selected.push(utxo.clone());

            if total_input >= target {
                return Ok(selected);
            }
        }

        Err(anyhow!(
            "Không đủ số dư. Cần {} sat (bao gồm fee), hiện có {} sat",
            target,
            total_input
        ))
    }

    fn estimate_auto_fee_sat(
        &self,
        input_count: usize,
        output_count: usize,
    ) -> Result<u64> {
        let vbytes = self.estimate_p2wpkh_vbytes(input_count, output_count)?;
        let fee_rate_sat_vb = self.estimate_fee_rate_sat_vb().unwrap_or(DEFAULT_AUTO_FEE_RATE_SAT_VB);
        let fee = (fee_rate_sat_vb * vbytes as f64).ceil() as u64;
        Ok(fee.max(1))
    }

    fn estimate_p2wpkh_vbytes(&self, input_count: usize, output_count: usize) -> Result<u64> {
        let input_vb = u64::try_from(input_count)
            .ok()
            .and_then(|count| count.checked_mul(ESTIMATE_P2WPKH_INPUT_VB))
            .ok_or_else(|| anyhow!("Số input quá lớn để estimate fee"))?;

        let output_vb = u64::try_from(output_count)
            .ok()
            .and_then(|count| count.checked_mul(ESTIMATE_P2WPKH_OUTPUT_VB))
            .ok_or_else(|| anyhow!("Số output quá lớn để estimate fee"))?;

        ESTIMATE_OVERHEAD_VB
            .checked_add(input_vb)
            .and_then(|value| value.checked_add(output_vb))
            .ok_or_else(|| anyhow!("Estimate vbytes bị overflow"))
    }

    fn estimate_fee_rate_sat_vb(&self) -> Result<f64> {
        let client = Client::new();
        let url = format!("{}/fee-estimates", self.network.blockstream_base_url());

        let fee_map: HashMap<String, f64> = client
            .get(&url)
            .send()
            .with_context(|| format!("Không gọi được API fee-estimates: {url}"))?
            .error_for_status()
            .with_context(|| format!("Lỗi response API fee-estimates: {url}"))?
            .json()
            .with_context(|| format!("Không parse được dữ liệu fee-estimates: {url}"))?;

        for target in ["1", "2", "3", "6", "12"] {
            if let Some(rate) = fee_map.get(target) {
                if rate.is_finite() && *rate > 0.0 {
                    return Ok(*rate);
                }
            }
        }

        if let Some(rate) = fee_map
            .values()
            .copied()
            .filter(|rate| rate.is_finite() && *rate > 0.0)
            .reduce(f64::min)
        {
            return Ok(rate);
        }

        Err(anyhow!("Không lấy được fee-rate hợp lệ từ Blockstream"))
    }

    fn broadcast_transaction(&self, raw_hex: &str) -> Result<String> {
        let client = Client::new();
        let url = format!("{}/tx", self.network.blockstream_base_url());
        let response = client
            .post(&url)
            .body(raw_hex.to_owned())
            .send()
            .with_context(|| format!("Không gọi được API broadcast: {url}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("Broadcast thất bại ({status}): {body}"));
        }

        let txid = response.text().unwrap_or_default();
        Ok(txid)
    }

    fn collect_wallet_utxos(&self) -> Result<Vec<SpendableUtxo>> {
        let client = Client::new();
        let base_url = self.network.blockstream_base_url();
        let mut utxos = Vec::new();

        for addr in &self.addresses {
            let url = format!("{base_url}/address/{}/utxo", addr.address);
            let rows: Vec<ApiAddressUtxo> = client
                .get(&url)
                .send()
                .with_context(|| format!("Không gọi được API UTXO: {url}"))?
                .error_for_status()
                .with_context(|| format!("Lỗi response API UTXO: {url}"))?
                .json()
                .with_context(|| format!("Không parse được UTXO của {}", addr.address))?;

            let checked_address =
                Address::from_str(&addr.address)?.require_network(self.network.bitcoin_network())?;

            for row in rows {
                utxos.push(SpendableUtxo {
                    txid: Txid::from_str(&row.txid)
                        .with_context(|| format!("txid không hợp lệ từ API: {}", row.txid))?,
                    vout: row.vout,
                    value: row.value,
                    address_index: addr.index,
                    address: checked_address.clone(),
                });
            }
        }

        Ok(utxos)
    }

    fn sign_transaction(
        &self,
        selected_utxos: &[SpendableUtxo],
        tx: &mut Transaction,
    ) -> Result<()> {
        let secp = Secp256k1::new();
        let account_xprv = Xpriv::from_str(&self.account_xprv)?;

        for (input_index, utxo) in selected_utxos.iter().enumerate() {
            let (_, private_key, public_key) =
                Self::derive_address_and_keys(&secp, &account_xprv, self.network, utxo.address_index)?;

            let script_code = utxo.address.script_pubkey();

            let sighash = SighashCache::new(&mut *tx).p2wpkh_signature_hash(
                input_index,
                &script_code,
                Amount::from_sat(utxo.value),
                EcdsaSighashType::All,
            )?;

            let msg = bitcoin::secp256k1::Message::from_digest_slice(sighash.as_ref())?;
            let signature = secp.sign_ecdsa(&msg, &private_key.inner);

            let mut signature_bytes = signature.serialize_der().to_vec();
            signature_bytes.push(EcdsaSighashType::All as u8);

            tx.input[input_index].witness =
                Witness::from_slice(&[signature_bytes.as_slice(), public_key.to_bytes().as_slice()]);
        }

        Ok(())
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