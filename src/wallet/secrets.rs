use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use super::structure::Wallet;

pub type WalletSecretsRef = Arc<WalletSecrets>;
pub type WalletSecretsVault = HashMap<String, WalletSecretsRef>;

#[derive(Debug)]
pub struct WalletSecrets {
    mnemonic: Option<SecretString>,
    account_xprv: SecretString,
}

pub struct WalletBundle {
    pub wallet: Wallet,
    pub secrets: WalletSecretsRef,
}

#[derive(Serialize, Deserialize, Default)]
pub struct StoredWalletSecrets {
    #[serde(default)]
    pub mnemonic: Option<String>,
    #[serde(default)]
    pub account_xprv: String,
}

#[derive(Serialize, Deserialize)]
pub struct StoredWallet {
    #[serde(flatten)]
    pub wallet: Wallet,
    #[serde(default)]
    pub secrets: StoredWalletSecrets,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    legacy_mnemonic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    legacy_account_xprv: Option<String>,
}

impl WalletSecrets {
    pub fn new(mnemonic: Option<String>, account_xprv: String) -> Self {
        Self {
            mnemonic: mnemonic.map(SecretString::from),
            account_xprv: SecretString::from(account_xprv),
        }
    }

    pub fn mnemonic_phrase(&self) -> Option<&str> {
        self.mnemonic.as_ref().map(|value| value.expose_secret())
    }

    pub fn account_xprv(&self) -> &str {
        self.account_xprv.expose_secret()
    }

    pub fn has_mnemonic(&self) -> bool {
        self.mnemonic.is_some()
    }

    pub fn mnemonic_word_count(&self) -> Option<usize> {
        self.mnemonic_phrase()
            .map(|value| value.split_whitespace().count())
    }
}

impl WalletBundle {
    pub fn new(wallet: Wallet, secrets: WalletSecrets) -> Self {
        Self {
            wallet,
            secrets: Arc::new(secrets),
        }
    }
}

impl StoredWallet {
    pub fn from_runtime(wallet: &Wallet, secrets: &WalletSecretsRef) -> Self {
        Self {
            wallet: wallet.clone(),
            secrets: StoredWalletSecrets {
                mnemonic: secrets.mnemonic_phrase().map(ToOwned::to_owned),
                account_xprv: secrets.account_xprv().to_string(),
            },
            legacy_mnemonic: None,
            legacy_account_xprv: None,
        }
    }

    pub fn into_runtime(mut self) -> Result<WalletBundle> {
        let mnemonic = self
            .secrets
            .mnemonic
            .take()
            .or_else(|| self.legacy_mnemonic.take());
        let account_xprv = if self.secrets.account_xprv.is_empty() {
            self.legacy_account_xprv
                .take()
                .ok_or_else(|| anyhow!("Wallet storage thiếu account_xprv"))?
        } else {
            std::mem::take(&mut self.secrets.account_xprv)
        };

        let secrets = WalletSecrets::new(mnemonic, account_xprv);
        let mut wallet = self.wallet.clone();
        wallet.sync_secret_metadata(&secrets);

        Ok(WalletBundle::new(wallet, secrets))
    }
}

impl Drop for StoredWalletSecrets {
    fn drop(&mut self) {
        if let Some(mnemonic) = &mut self.mnemonic {
            mnemonic.zeroize();
        }
        self.account_xprv.zeroize();
    }
}

impl Drop for StoredWallet {
    fn drop(&mut self) {
        if let Some(mnemonic) = &mut self.legacy_mnemonic {
            mnemonic.zeroize();
        }
        if let Some(account_xprv) = &mut self.legacy_account_xprv {
            account_xprv.zeroize();
        }
    }
}

pub fn stored_wallets_from_runtime(
    wallets: &[Wallet],
    wallet_vault: &WalletSecretsVault,
) -> Result<Vec<StoredWallet>> {
    wallets
        .iter()
        .map(|wallet| {
            let secrets = wallet_vault
                .get(wallet.wallet_id())
                .ok_or_else(|| anyhow!("Thiếu secret cho wallet '{}'", wallet.name))?;
            Ok(StoredWallet::from_runtime(wallet, secrets))
        })
        .collect()
}

pub fn runtime_wallets_from_stored(
    stored_wallets: Vec<StoredWallet>,
) -> Result<(Vec<Wallet>, WalletSecretsVault)> {
    let mut wallets = Vec::with_capacity(stored_wallets.len());
    let mut wallet_vault = WalletSecretsVault::with_capacity(stored_wallets.len());

    for stored_wallet in stored_wallets {
        let bundle = stored_wallet.into_runtime()?;
        wallet_vault.insert(
            bundle.wallet.wallet_id().to_string(),
            bundle.secrets.clone(),
        );
        wallets.push(bundle.wallet);
    }

    Ok((wallets, wallet_vault))
}
