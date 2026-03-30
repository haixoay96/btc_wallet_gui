use bitcoin::{Address, OutPoint, Txid};

#[derive(Debug, Clone)]
pub struct SpendableUtxo {
    pub txid: Txid,
    pub vout: u32,
    pub value: u64,
    pub address_index: u32,
    pub address: Address,
}