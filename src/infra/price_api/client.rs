use anyhow::{Context, Result};
use reqwest::blocking::Client;
use std::time::Duration;

use super::structure::{BtcPriceData, CoinGeckoResponse};

/// Price API client for fetching BTC price from CoinGecko
pub struct PriceClient {
    client: Client,
}

impl PriceClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Fetch fresh BTC price from CoinGecko API
    pub fn fetch_price(&self) -> Result<BtcPriceData> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true";

        let response = self
            .client
            .get(url)
            .header("User-Agent", "btc_wallet_gui/0.1.0")
            .header("Accept", "application/json")
            .send()
            .with_context(|| format!("Failed to send request to CoinGecko: {url}"))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(anyhow::anyhow!(
                "CoinGecko API error (HTTP {}): {}",
                status,
                body
            ));
        }

        let response: CoinGeckoResponse = response
            .json()
            .with_context(|| "Failed to parse CoinGecko response")?;

        Ok(BtcPriceData {
            price_usd: response.bitcoin.usd,
            change_24h: response.bitcoin.usd_24h_change,
        })
    }
}
