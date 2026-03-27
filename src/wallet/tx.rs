use super::address::{derive_address_and_keys, derive_next_addresses};
use super::*;

pub fn create_transaction_with_options(
    entry: &mut WalletEntry,
    to_address: &str,
    amount_sat: u64,
    fee_sat: u64,
    options: TxBuildOptions,
) -> Result<BuildTxResult> {
    if amount_sat == 0 {
        return Err(anyhow!("amount_sat phải > 0"));
    }

    if entry.addresses.is_empty() {
        derive_next_addresses(entry, DEFAULT_GAP_LIMIT)?;
    }

    let unchecked = Address::from_str(to_address).context("Địa chỉ nhận không hợp lệ")?;
    let to_address = unchecked
        .require_network(entry.network.bitcoin_network())
        .context("Địa chỉ nhận không đúng network của ví")?;

    let mut utxos = collect_spendable_utxos_by_source(entry, &options.input_source)?;
    utxos.sort_by_key(|u| u.value);
    utxos.reverse();

    let selected = select_utxos_for_target(&utxos, amount_sat, fee_sat)?;
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
            ChangeStrategy::NewAddress => derive_next_addresses(entry, 1)?
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("Không tạo được change address"))?,
            ChangeStrategy::ExistingIndex(index) => entry
                .addresses
                .iter()
                .find(|addr| addr.index == index)
                .map(|addr| addr.address.clone())
                .ok_or_else(|| anyhow!("change index {} không tồn tại trong ví", index))?,
        };

        let checked_change =
            Address::from_str(&change_address)?.require_network(entry.network.bitcoin_network())?;

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

    sign_transaction(entry, &selected, &mut tx)?;

    let raw_hex = consensus::encode::serialize_hex(&tx);
    let txid = tx.compute_txid().to_string();

    if options.broadcast {
        broadcast_transaction(entry.network, &raw_hex)?;
    }

    entry.history.insert(
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
    entry: &WalletEntry,
    amount_sat: u64,
    input_source: &InputSource,
) -> Result<u64> {
    if amount_sat == 0 {
        return Err(anyhow!("amount_sat phải > 0"));
    }

    let mut utxos = collect_spendable_utxos_by_source(entry, input_source)?;
    utxos.sort_by_key(|u| u.value);
    utxos.reverse();

    let mut selected_count = 0usize;
    let mut total_input = 0u64;

    for utxo in utxos {
        selected_count += 1;
        total_input = total_input
            .checked_add(utxo.value)
            .ok_or_else(|| anyhow!("Tổng UTXO bị overflow"))?;

        let fee_no_change = estimate_auto_fee_sat(entry.network, selected_count, 1)?;
        if let Some(target_no_change) = amount_sat.checked_add(fee_no_change) {
            if total_input >= target_no_change {
                let change_no_change = total_input - target_no_change;
                if change_no_change < DUST_LIMIT_SAT {
                    return Ok(fee_no_change);
                }
            }
        }

        let fee_change = estimate_auto_fee_sat(entry.network, selected_count, 2)?;
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

pub fn create_send_all_transaction_with_options(
    entry: &mut WalletEntry,
    to_address: &str,
    fee_mode: FeeMode,
    options: TxBuildOptions,
) -> Result<BuildTxResult> {
    if entry.addresses.is_empty() {
        derive_next_addresses(entry, DEFAULT_GAP_LIMIT)?;
    }

    let unchecked = Address::from_str(to_address).context("Địa chỉ nhận không hợp lệ")?;
    let to_address = unchecked
        .require_network(entry.network.bitcoin_network())
        .context("Địa chỉ nhận không đúng network của ví")?;

    let mut utxos = collect_spendable_utxos_by_source(entry, &options.input_source)?;
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
        FeeMode::Auto => estimate_auto_fee_sat(entry.network, utxos.len(), 1)?,
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

    sign_transaction(entry, &utxos, &mut tx)?;

    let raw_hex = consensus::encode::serialize_hex(&tx);
    let txid = tx.compute_txid().to_string();

    if options.broadcast {
        broadcast_transaction(entry.network, &raw_hex)?;
    }

    entry.history.insert(
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
    entry: &WalletEntry,
    input_source: &InputSource,
) -> Result<Vec<SpendableUtxo>> {
    let known_indices = entry
        .addresses
        .iter()
        .map(|addr| addr.index)
        .collect::<HashSet<_>>();

    let mut utxos = collect_wallet_utxos(entry)?;

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
    network: WalletNetwork,
    input_count: usize,
    output_count: usize,
) -> Result<u64> {
    let vbytes = estimate_p2wpkh_vbytes(input_count, output_count)?;
    let fee_rate_sat_vb = estimate_fee_rate_sat_vb(network).unwrap_or(DEFAULT_AUTO_FEE_RATE_SAT_VB);
    let fee = (fee_rate_sat_vb * vbytes as f64).ceil() as u64;
    Ok(fee.max(1))
}

fn estimate_p2wpkh_vbytes(input_count: usize, output_count: usize) -> Result<u64> {
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

fn estimate_fee_rate_sat_vb(network: WalletNetwork) -> Result<f64> {
    let client = Client::new();
    let url = format!("{}/fee-estimates", network.blockstream_base_url());

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

fn broadcast_transaction(network: WalletNetwork, raw_hex: &str) -> Result<String> {
    let client = Client::new();
    let url = format!("{}/tx", network.blockstream_base_url());
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

fn collect_wallet_utxos(entry: &WalletEntry) -> Result<Vec<SpendableUtxo>> {
    let client = Client::new();
    let base_url = entry.network.blockstream_base_url();
    let mut utxos = Vec::new();

    for addr in &entry.addresses {
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
            Address::from_str(&addr.address)?.require_network(entry.network.bitcoin_network())?;

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
    entry: &WalletEntry,
    selected_utxos: &[SpendableUtxo],
    tx: &mut Transaction,
) -> Result<()> {
    let secp = Secp256k1::new();
    let account_xprv = Xpriv::from_str(&entry.account_xprv)?;

    for (input_index, utxo) in selected_utxos.iter().enumerate() {
        let (_, private_key, public_key) =
            derive_address_and_keys(&secp, &account_xprv, entry.network, utxo.address_index)?;

        let script_code = ScriptBuf::new_p2pkh(&public_key.pubkey_hash());

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

        let _ = &utxo.address;
    }

    Ok(())
}
