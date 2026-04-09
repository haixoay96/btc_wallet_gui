use super::structure::WalletChoice;

use super::structure::*;
use std::fmt;

use crate::core::wallet::Wallet;

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
