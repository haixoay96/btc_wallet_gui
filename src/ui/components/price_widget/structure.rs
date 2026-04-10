use crate::infra::price_api::BtcPriceData;

/// Message for price widget interactions
#[derive(Debug, Clone)]
pub enum PriceWidgetMessage {
    RefreshPrice,
    PriceFetched(Result<BtcPriceData, String>),
}
