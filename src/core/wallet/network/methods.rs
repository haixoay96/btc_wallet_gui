use bitcoin::Network;

use super::structure::{AddressChain, WalletNetwork};

// ─── Impl WalletNetwork ──────────────────────────────────────────────────

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
}

impl std::str::FromStr for WalletNetwork {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed = value.trim();
        if trimmed.eq_ignore_ascii_case("mainnet") || trimmed.eq_ignore_ascii_case("bitcoin") {
            Ok(Self::Mainnet)
        } else if trimmed.eq_ignore_ascii_case("testnet")
            || trimmed.eq_ignore_ascii_case("testnet3")
        {
            Ok(Self::Testnet)
        } else {
            Err("unsupported wallet network")
        }
    }
}

// ─── Impl AddressChain ───────────────────────────────────────────────────

impl AddressChain {
    pub fn branch_index(self) -> u32 {
        match self {
            Self::External => 0,
            Self::Internal => 1,
        }
    }

    pub fn matches_receive_flow(self) -> bool {
        matches!(self, Self::External)
    }
}
