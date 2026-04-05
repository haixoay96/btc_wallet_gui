use crate::i18n::t;

/// BTC denomination units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BtcUnit {
    #[default]
    Btc,
    Satoshi,
    MilliBtc,
}

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

    pub fn sat_to_display(&self, sat: u64) -> String {
        match self {
            Self::Btc => {
                let btc = sat as f64 / 100_000_000.0;
                format!("{:.8}", btc)
            }
            Self::Satoshi => sat.to_string(),
            Self::MilliBtc => {
                let mbtc = sat as f64 / 100_000.0;
                format!("{:.5}", mbtc)
            }
        }
    }

    pub fn display_to_sat(&self, display: &str) -> Option<u64> {
        match self {
            Self::Btc => {
                let btc: f64 = display.parse().ok()?;
                Some((btc * 100_000_000.0).round() as u64)
            }
            Self::Satoshi => {
                let sat: u64 = display.parse().ok()?;
                Some(sat)
            }
            Self::MilliBtc => {
                let mbtc: f64 = display.parse().ok()?;
                Some((mbtc * 100_000.0).round() as u64)
            }
        }
    }
}

/// Format amount with unit label
pub fn format_amount_with_unit(sat: u64, unit: BtcUnit) -> String {
    let display = unit.sat_to_display(sat);
    format!("{} {}", display, unit.symbol())
}
