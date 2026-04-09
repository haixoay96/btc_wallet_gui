use std::collections::HashMap;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;

use super::structure::{ApiAddressUtxo, ApiTx};
use crate::core::wallet::WalletNetwork;

/// Default Esplora API endpoint
#[allow(dead_code)]
const DEFAULT_ESPLORA_ENDPOINT: &str = "https://blockstream.info/api";
const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 15;
const ESPLORA_CONFIRMED_PAGE_SIZE: usize = 25;

pub struct EsploraClient {
    client: Client,
    base_url: String,
}

impl EsploraClient {
    pub fn new(network: WalletNetwork) -> Result<Self> {
        Self::with_custom_endpoint(None, network)
    }

    pub fn with_custom_endpoint(
        custom_endpoint: Option<String>,
        network: WalletNetwork,
    ) -> Result<Self> {
        // Load timeout from storage if available, otherwise use default
        let timeout_secs = if let Ok(storage) = crate::infra::storage::Storage::new() {
            storage
                .load_timeout_secs()
                .unwrap_or(DEFAULT_HTTP_TIMEOUT_SECS)
        } else {
            DEFAULT_HTTP_TIMEOUT_SECS
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .context("Không khởi tạo được HTTP client")?;

        let base_url = custom_endpoint
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| network.blockstream_base_url().to_string());

        Ok(Self { client, base_url })
    }

    pub fn test_connection(endpoint: &str, timeout_secs: u64) -> Result<String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;
        let health_url = format!("{}/blocks/tip/height", endpoint);
        let resp = client.get(&health_url).send()?;
        let height: String = resp.text()?;
        Ok(height.trim().to_string())
    }

    pub fn fetch_all_address_txs(&self, address: &str) -> Result<Vec<ApiTx>> {
        let base_url = &self.base_url;
        let first_page_url = format!("{base_url}/address/{address}/txs");
        let mut txs = self.fetch_txs_page(&first_page_url, address)?;

        let confirmed_count = txs.iter().filter(|tx| tx.status.confirmed).count();
        let mut last_seen_txid = txs
            .iter()
            .rev()
            .find(|tx| tx.status.confirmed)
            .map(|tx| tx.txid.clone());

        while confirmed_count >= ESPLORA_CONFIRMED_PAGE_SIZE && last_seen_txid.is_some() {
            let cursor = last_seen_txid.take().unwrap_or_default();
            let page_url = format!("{base_url}/address/{address}/txs/chain/{cursor}");
            let page = self.fetch_txs_page(&page_url, address)?;

            if page.is_empty() {
                break;
            }

            last_seen_txid = page.last().map(|tx| tx.txid.clone());
            let page_len = page.len();
            txs.extend(page);

            if page_len < ESPLORA_CONFIRMED_PAGE_SIZE {
                break;
            }
        }

        Ok(txs)
    }

    pub fn fetch_address_utxos(&self, address: &str) -> Result<Vec<ApiAddressUtxo>> {
        let url = format!("{}/address/{address}/utxo", &self.base_url);
        self.client
            .get(&url)
            .send()
            .with_context(|| format!("Không gọi được API UTXO: {url}"))?
            .error_for_status()
            .with_context(|| format!("Lỗi response API UTXO: {url}"))?
            .json()
            .with_context(|| format!("Không parse được UTXO của {address}"))
    }

    pub fn fetch_fee_rate_sat_vb(&self) -> Result<f64> {
        let url = format!("{}/fee-estimates", &self.base_url);

        let fee_map: HashMap<String, f64> = self
            .client
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

        fee_map
            .values()
            .copied()
            .filter(|rate| rate.is_finite() && *rate > 0.0)
            .reduce(f64::min)
            .ok_or_else(|| anyhow!("Không lấy được fee-rate hợp lệ từ Blockstream"))
    }

    pub fn broadcast_transaction(&self, raw_hex: &str) -> Result<String> {
        let url = format!("{}/tx", &self.base_url);
        let response = self
            .client
            .post(&url)
            .body(raw_hex.to_owned())
            .send()
            .with_context(|| format!("Không gọi được API broadcast: {url}"))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("Broadcast thất bại (HTTP {status}): {body}"));
        }

        Ok(response.text().unwrap_or_default())
    }

    fn fetch_txs_page(&self, url: &str, address: &str) -> Result<Vec<ApiTx>> {
        self.client
            .get(url)
            .send()
            .with_context(|| format!("Không gọi được API lịch sử: {url}"))?
            .error_for_status()
            .with_context(|| format!("Lỗi response API lịch sử: {url}"))?
            .json()
            .with_context(|| format!("Không parse được dữ liệu lịch sử của {address}"))
    }

    /// Lấy chiều cao blockchain hiện tại
    pub fn get_blockchain_height(&self) -> Result<u32> {
        let url = format!("{}/blocks/tip/height", &self.base_url);
        self.client
            .get(&url)
            .send()
            .with_context(|| format!("Không gọi được API chiều cao blockchain: {url}"))?
            .error_for_status()
            .with_context(|| format!("Lỗi response API chiều cao blockchain: {url}"))?
            .json()
            .with_context(|| "Không parse được chiều cao blockchain")
    }
}

#[cfg(test)]
mod tests {
    use super::{WalletNetwork, ESPLORA_CONFIRMED_PAGE_SIZE};

    #[test]
    fn blockstream_base_url_stays_network_specific() {
        assert!(WalletNetwork::Mainnet
            .blockstream_base_url()
            .ends_with("/api"));
        assert!(WalletNetwork::Testnet
            .blockstream_base_url()
            .contains("/testnet/api"));
        assert_eq!(ESPLORA_CONFIRMED_PAGE_SIZE, 25);
    }
}
