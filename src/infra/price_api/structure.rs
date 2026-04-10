use serde::Deserialize;

/// BTC price data from API
#[derive(Debug, Clone)]
pub struct BtcPriceData {
    /// Price in USD
    pub price_usd: f64,
    /// 24h change percentage
    pub change_24h: f64,
}

/// CoinGecko API response structure
#[derive(Debug, Deserialize)]
pub struct CoinGeckoResponse {
    pub bitcoin: CoinGeckoPrice,
}

#[derive(Debug, Deserialize)]
pub struct CoinGeckoPrice {
    pub usd: f64,
    pub usd_24h_change: f64,
}
