use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiAddressUtxo {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
}

#[derive(Debug, Deserialize)]
pub struct AddressStats {
    #[allow(dead_code)]
    pub address: String,
    #[serde(default)]
    pub chain_stats: AddressChainStats,
    #[allow(dead_code)]
    #[serde(default)]
    pub mempool_stats: AddressChainStats,
}

#[derive(Debug, Deserialize, Default)]
pub struct AddressChainStats {
    #[serde(default)]
    pub spent_txo_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct ApiTxStatus {
    pub confirmed: bool,
    pub block_time: Option<u64>,
    #[serde(default)]
    pub block_height: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ApiPrevout {
    #[serde(default)]
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}

#[derive(Debug, Deserialize)]
pub struct ApiVin {
    #[serde(default)]
    pub prevout: Option<ApiPrevout>,
}

#[derive(Debug, Deserialize)]
pub struct ApiVout {
    #[serde(default)]
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}

#[derive(Debug, Deserialize)]
pub struct ApiTx {
    pub txid: String,
    pub vin: Vec<ApiVin>,
    pub vout: Vec<ApiVout>,
    #[serde(default)]
    pub fee: Option<u64>,
    pub status: ApiTxStatus,
}
