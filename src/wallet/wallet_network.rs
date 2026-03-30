use std::str::FromStr;

use anyhow::{anyhow, Result};
use bitcoin::Network;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WalletNetwork {
    Mainnet,
    Testnet,
}

impl WalletNetwork {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
        }
    }

    pub fn coin_type(self) -> u32 {
        match self {
            Self::Mainnet => 0,
            Self::Testnet => 1,
        }
    }

    pub fn bitcoin_network(self) -> Network {
        match self {
            Self::Mainnet => Network::Bitcoin,
            Self::Testnet => Network::Testnet,
        }
    }

    pub fn blockstream_base_url(self) -> &'static str {
        match self {
            Self::Mainnet => "https://blockstream.info/api",
            Self::Testnet => "https://blockstream.info/testnet/api",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "mainnet" | "main" | "bitcoin" | "btc" => Ok(Self::Mainnet),
            "testnet" | "test" | "tb" => Ok(Self::Testnet),
            _ => Err(anyhow!(
                "Network không hợp lệ: {value}. Dùng mainnet hoặc testnet"
            )),
        }
    }
}