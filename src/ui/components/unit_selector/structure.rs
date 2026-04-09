use crate::i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BtcUnit {
    #[default]
    Btc,
    Satoshi,
    MilliBtc,
}
