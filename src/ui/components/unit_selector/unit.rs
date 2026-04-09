use super::structure::BtcUnit;
use crate::i18n::t;

/// BTC denomination units

impl std::fmt::Display for BtcUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl BtcUnit {
    pub fn all() -> Vec<Self> {
        vec![Self::Btc, Self::Satoshi, Self::MilliBtc]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Btc => t("BTC", "BTC"),
            Self::Satoshi => t("Satoshi", "Satoshi"),
            Self::MilliBtc => t("mBTC", "mBTC"),
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Btc => "BTC",
            Self::Satoshi => "sat",
            Self::MilliBtc => "mBTC",
        }
    }
}
