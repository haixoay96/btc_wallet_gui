use std::collections::{HashMap, HashSet};

use anyhow::Result;

use super::network::AddressChain;
use super::structure::{AddressEntry, TxDirection, TxRecord, Wallet};
use crate::core::wallet::secrets::WalletSecrets;
use crate::core::wallet::DEFAULT_GAP_LIMIT;
use crate::infra::network::ApiTx;
use crate::infra::network::EsploraClient;

struct ChainSyncResult {
    entries: Vec<AddressEntry>,
    txs: Vec<ApiTx>,
    next_index: u32,
}

impl Wallet {
    pub fn refresh_history(&mut self, secrets: &WalletSecrets) -> Result<usize> {
        let esplora = EsploraClient::new(self.network)?;

        // Lấy chiều cao blockchain hiện tại để tính confirmations
        let current_height = esplora.get_blockchain_height().ok();

        let ChainSyncResult {
            entries: mut external_entries,
            txs: mut external_txs,
            next_index: next_external_index,
        } = self.sync_chain_history(secrets, AddressChain::External, &esplora)?;
        let ChainSyncResult {
            entries: internal_entries,
            txs: internal_txs,
            next_index: next_internal_index,
        } = self.sync_chain_history(secrets, AddressChain::Internal, &esplora)?;

        external_entries.extend(internal_entries);
        let mut addresses = external_entries;
        addresses.sort_by_key(|entry| {
            let chain_order = match entry.chain {
                AddressChain::External => 0u8,
                AddressChain::Internal => 1u8,
            };
            (chain_order, entry.index)
        });

        external_txs.extend(internal_txs);
        let txs = external_txs;

        let own_addresses = addresses
            .iter()
            .map(|entry| entry.address.clone())
            .collect::<HashSet<_>>();

        self.addresses = addresses;
        self.next_external_index = next_external_index;
        self.next_internal_index = next_internal_index;
        self.history = Self::tx_records_from_api(txs, &own_addresses, current_height);

        Ok(self.history.len())
    }

    fn sync_chain_history(
        &self,
        secrets: &WalletSecrets,
        chain: AddressChain,
        esplora: &EsploraClient,
    ) -> Result<ChainSyncResult> {
        use bitcoin::key::Secp256k1;

        let secp = Secp256k1::new();
        let account_xprv = Self::parse_account_xprv(secrets)?;
        let existing_max_index = self
            .addresses
            .iter()
            .filter(|entry| entry.chain == chain)
            .map(|entry| entry.index)
            .max()
            .map(|index| index + 1)
            .unwrap_or(0);

        let minimum_scan_count = existing_max_index.max(DEFAULT_GAP_LIMIT);
        let mut entries = Vec::new();
        let mut txs = Vec::new();
        let mut index = 0u32;
        let mut empty_streak = 0u32;

        loop {
            let (address, _, _) =
                Self::derive_address_and_keys(&secp, &account_xprv, self.network, chain, index)?;
            let address_string = address.to_string();
            let address_txs = esplora.fetch_all_address_txs(&address_string)?;
            let has_activity = !address_txs.is_empty();

            entries.push(AddressEntry {
                index,
                address: address_string,
                chain,
            });
            txs.extend(address_txs);

            if has_activity {
                empty_streak = 0;
            } else {
                empty_streak += 1;
            }

            index += 1;

            if index >= minimum_scan_count && empty_streak >= DEFAULT_GAP_LIMIT {
                break;
            }
        }

        Ok(ChainSyncResult {
            entries,
            txs,
            next_index: index,
        })
    }

    fn tx_records_from_api(
        txs: Vec<ApiTx>,
        own_addresses: &HashSet<String>,
        current_height: Option<u32>,
    ) -> Vec<TxRecord> {
        let mut tx_map: HashMap<String, TxRecord> = HashMap::new();

        for tx in txs {
            let received: u64 = tx
                .vout
                .iter()
                .filter_map(|vout| {
                    let address = vout.scriptpubkey_address.as_ref()?;
                    if own_addresses.contains(address) {
                        Some(vout.value)
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

            // Tính số confirmations
            let confirmations = if tx.status.confirmed {
                if let (Some(height), Some(block_height)) = (current_height, tx.status.block_height)
                {
                    height.saturating_sub(block_height) + 1
                } else {
                    0
                }
            } else {
                0
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
                    confirmations,
                },
            );
        }

        let mut history: Vec<TxRecord> = tx_map.into_values().collect();
        history.sort_by(|a, b| match (a.block_time, b.block_time) {
            (None, None) => a.txid.cmp(&b.txid),
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(a_time), Some(b_time)) => b_time.cmp(&a_time).then_with(|| a.txid.cmp(&b.txid)),
        });
        history
    }
}
