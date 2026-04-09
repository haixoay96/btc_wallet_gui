use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletChoice {
    pub index: usize,
    pub label: String,
}
