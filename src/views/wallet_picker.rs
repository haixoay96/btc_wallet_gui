use std::fmt;

use crate::wallet::Wallet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletChoice {
    pub index: usize,
    pub label: String,
}

impl fmt::Display for WalletChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

pub fn wallet_choices(wallets: &[Wallet]) -> Vec<WalletChoice> {
    wallets
        .iter()
        .enumerate()
        .map(|(index, wallet)| WalletChoice {
            index,
            label: format!("{} ({})", wallet.name, wallet.network.as_str()),
        })
        .collect()
}

pub fn selected_wallet_choice(wallets: &[Wallet], selected_wallet: usize) -> Option<WalletChoice> {
    wallets.get(selected_wallet).map(|wallet| WalletChoice {
        index: selected_wallet,
        label: format!("{} ({})", wallet.name, wallet.network.as_str()),
    })
}
