#[derive(Debug, Clone)]
pub struct BuildTxResult {
    pub raw_hex: String,
    pub txid: String,
    pub broadcasted: bool,
}