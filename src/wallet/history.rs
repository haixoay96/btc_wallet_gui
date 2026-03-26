use super::address::derive_next_addresses;
use super::*;

pub fn refresh_history(entry: &mut WalletEntry) -> Result<usize> {
    if entry.addresses.is_empty() {
        derive_next_addresses(entry, DEFAULT_GAP_LIMIT)?;
    }

    let client = Client::new();
    let base_url = entry.network.blockstream_base_url();
    let own_addresses: HashSet<String> =
        entry.addresses.iter().map(|a| a.address.clone()).collect();

    let mut tx_map: HashMap<String, TxRecord> = HashMap::new();

    for addr in &entry.addresses {
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
        b.block_time
            .unwrap_or_default()
            .cmp(&a.block_time.unwrap_or_default())
            .then_with(|| a.txid.cmp(&b.txid))
    });

    let count = history.len();
    entry.history = history;
    Ok(count)
}