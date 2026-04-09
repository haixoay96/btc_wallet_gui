use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletChoice {
    pub index: usize,
    pub label: String,
}

impl fmt::Display for WalletChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}
