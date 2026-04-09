use std::collections::HashSet;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use bitcoin::{
    absolute, consensus, key::Secp256k1, sighash::*, transaction::Version, Address, Amount,
    OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
};

use super::network::AddressChain;
use super::structure::{
    BuildTxResult, ChangeStrategy, InputSource, SpendableUtxo, TxBuildOptions, TxDirection,
    TxRecord, Wallet,
};
use crate::wallet::esplora::EsploraClient;
use crate::wallet::secrets::WalletSecrets;
use crate::wallet::{
    DEFAULT_AUTO_FEE_RATE_SAT_VB, DEFAULT_GAP_LIMIT, DUST_LIMIT_SAT, ESTIMATE_OVERHEAD_VB,
    ESTIMATE_P2WPKH_INPUT_VB, ESTIMATE_P2WPKH_OUTPUT_VB,
};

impl Wallet {
    pub fn create_transaction_with_options(
        &mut self,
        secrets: &WalletSecrets,
        to_address: &str,
        amount_sat: u64,
        fee_sat: u64,
        options: TxBuildOptions,
    ) -> Result<BuildTxResult> {
        if amount_sat == 0 {
            return Err(anyhow!("amount_sat phải > 0"));
        }

        if self.addresses.is_empty() {
            self.derive_next_addresses(secrets, DEFAULT_GAP_LIMIT)?;
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
                ChangeStrategy::NewAddress => self
                    .derive_next_change_addresses(secrets, 1)?
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow!("Không tạo được change address"))?,
                ChangeStrategy::ExistingIndex(index) => self
                    .addresses
                    .iter()
                    .find(|addr| addr.index == index && addr.chain == AddressChain::External)
                    .map(|addr| addr.address.clone())
                    .ok_or_else(|| anyhow!("change index {} không tồn tại trong ví", index))?,
            };

            let checked_change = Address::from_str(&change_address)?
                .require_network(self.network.bitcoin_network())?;

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

        self.sign_transaction(secrets, &selected, &mut tx)?;

        let txid = tx.compute_txid().to_string();

        if options.broadcast {
            let raw_hex = consensus::encode::serialize_hex(&tx);
            self.broadcast_transaction(&raw_hex)?;
        }

        self.history.insert(
            0,
            TxRecord {
                txid: txid.clone(),
                direction: TxDirection::Outgoing,
                amount_sat: -(i64::try_from(amount_sat.saturating_add(fee_sat))
                    .unwrap_or(i64::MAX)),
                fee_sat: Some(fee_sat),
                confirmed: false,
                block_time: None,
                confirmations: 0,
            },
        );

        Ok(BuildTxResult {
            txid,
            broadcasted: options.broadcast,
        })
    }

    // ─── Fee estimation ──────────────────────────────────────────────

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

    pub fn estimate_fee_for_send_all(&self, input_source: &InputSource) -> Result<(u64, u64)> {
        let utxos = self.collect_spendable_utxos_by_source(input_source)?;

        if utxos.is_empty() {
            return Err(anyhow!("Không có UTXO khả dụng"));
        }

        let total_input: u64 = utxos.iter().try_fold(0u64, |acc, utxo| {
            acc.checked_add(utxo.value)
                .ok_or_else(|| anyhow!("Tổng UTXO bị overflow"))
        })?;

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

    fn estimate_auto_fee_sat(&self, input_count: usize, output_count: usize) -> Result<u64> {
        let vbytes = self.estimate_p2wpkh_vbytes(input_count, output_count)?;
        let fee_rate_sat_vb = self
            .estimate_fee_rate_sat_vb()
            .unwrap_or(DEFAULT_AUTO_FEE_RATE_SAT_VB);
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
        EsploraClient::new(self.network)?.fetch_fee_rate_sat_vb()
    }

    // ─── UTXO selection ──────────────────────────────────────────────

    fn collect_spendable_utxos_by_source(
        &self,
        input_source: &InputSource,
    ) -> Result<Vec<SpendableUtxo>> {
        let known_indices = self
            .addresses
            .iter()
            .filter(|addr| addr.chain.matches_receive_flow())
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

                utxos.retain(|utxo| {
                    utxo.chain.matches_receive_flow()
                        && selected_indices.contains(&utxo.address_index)
                });

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

    fn collect_wallet_utxos(&self) -> Result<Vec<SpendableUtxo>> {
        let esplora = EsploraClient::new(self.network)?;
        let mut utxos = Vec::new();

        for addr in &self.addresses {
            let rows: Vec<crate::wallet::api_types::ApiAddressUtxo> =
                esplora.fetch_address_utxos(&addr.address)?;

            let checked_address = Address::from_str(&addr.address)?
                .require_network(self.network.bitcoin_network())?;

            for row in rows {
                utxos.push(SpendableUtxo {
                    txid: Txid::from_str(&row.txid)
                        .with_context(|| format!("txid không hợp lệ từ API: {}", row.txid))?,
                    vout: row.vout,
                    value: row.value,
                    address_index: addr.index,
                    chain: addr.chain,
                    address: checked_address.clone(),
                });
            }
        }

        Ok(utxos)
    }

    // ─── Signing & broadcast ─────────────────────────────────────────

    fn sign_transaction(
        &self,
        secrets: &WalletSecrets,
        selected_utxos: &[SpendableUtxo],
        tx: &mut Transaction,
    ) -> Result<()> {
        let secp = Secp256k1::new();
        let account_xprv = Self::parse_account_xprv(secrets)?;

        for (input_index, utxo) in selected_utxos.iter().enumerate() {
            let (_, private_key, public_key) = Self::derive_address_and_keys(
                &secp,
                &account_xprv,
                self.network,
                utxo.chain,
                utxo.address_index,
            )?;

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

            tx.input[input_index].witness = Witness::from_slice(&[
                signature_bytes.as_slice(),
                public_key.to_bytes().as_slice(),
            ]);
        }

        Ok(())
    }

    fn broadcast_transaction(&self, raw_hex: &str) -> Result<String> {
        EsploraClient::new(self.network)?.broadcast_transaction(raw_hex)
    }
}
