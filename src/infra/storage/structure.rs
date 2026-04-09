use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::core::wallet::{
    runtime_wallets_from_stored, stored_wallets_from_runtime, StoredWallet, Wallet,
    WalletSecretsVault,
};
use crate::infra::paths::StoragePaths;
use crate::ui::i18n::AppLanguage;

// ─── State structs ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserProfile {
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub language: AppLanguage,
}

#[derive(Default, Serialize, Deserialize)]
pub struct PersistedState {
    #[serde(default)]
    pub profile: UserProfile,
    #[serde(default)]
    pub wallets: Vec<StoredWallet>,
}

pub struct RuntimeState {
    pub profile: UserProfile,
    pub wallets: Vec<Wallet>,
    pub wallet_vault: WalletSecretsVault,
}

impl PersistedState {
    pub fn from_runtime(
        profile: UserProfile,
        wallets: &[Wallet],
        wallet_vault: &WalletSecretsVault,
    ) -> Result<Self> {
        Ok(Self {
            profile,
            wallets: stored_wallets_from_runtime(wallets, wallet_vault)?,
        })
    }

    pub fn into_runtime(self) -> Result<RuntimeState> {
        let (wallets, wallet_vault) = runtime_wallets_from_stored(self.wallets)?;
        Ok(RuntimeState {
            profile: self.profile,
            wallets,
            wallet_vault,
        })
    }
}

// ─── Storage struct ──────────────────────────────────────────────────────

#[derive(Debug)]
pub struct Storage {
    pub(crate) paths: StoragePaths,
}

impl Storage {
    pub fn new() -> anyhow::Result<Self> {
        let paths = StoragePaths::resolve()?;
        Ok(Self { paths })
    }

    pub fn file_path(&self) -> &std::path::Path {
        &self.paths.encrypted_state_file
    }

    pub fn has_existing_state(&self) -> bool {
        self.paths.encrypted_state_file.exists()
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new().expect("Failed to initialize storage")
    }
}

// ─── Preferences struct ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreferences {
    #[serde(default)]
    pub language: AppLanguage,
    #[serde(default)]
    pub last_selected_wallet: Option<usize>,
    #[serde(default)]
    pub last_viewed_page: Option<String>,
    #[serde(default)]
    pub theme: AppTheme,
    #[serde(default)]
    pub font_scale: f64,
    #[serde(default)]
    pub high_contrast: bool,
    #[serde(default)]
    pub onboarding_completed: bool,
    #[serde(default)]
    pub esplora_endpoint: String,
    #[serde(default)]
    pub timeout_secs: u64,
    #[serde(default)]
    pub enable_debug: bool,
    #[serde(default)]
    pub auto_refresh: bool,
    #[serde(default)]
    pub show_satoshis: bool,
    #[serde(default)]
    pub compact_mode: bool,
    #[serde(default)]
    pub show_btc_price: bool,
    #[serde(default)]
    pub wallet_sort_field: WalletSortField,
    #[serde(default)]
    pub wallet_sort_ascending: bool,
    /// Timestamp (unix epoch seconds) of last backup reminder dismissal
    #[serde(default)]
    pub last_backup_reminder_dismissed: Option<i64>,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            language: AppLanguage::English,
            last_selected_wallet: None,
            last_viewed_page: None,
            theme: AppTheme::Dark,
            font_scale: 1.0,
            high_contrast: false,
            onboarding_completed: false,
            esplora_endpoint: String::new(),
            timeout_secs: 15,
            enable_debug: false,
            auto_refresh: false,
            show_satoshis: false,
            compact_mode: false,
            show_btc_price: false,
            wallet_sort_field: WalletSortField::Balance,
            wallet_sort_ascending: false,
            last_backup_reminder_dismissed: None,
        }
    }
}

// ─── Theme/Sort enums ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppTheme {
    #[default]
    Dark,
    Light,
    System,
}

impl fmt::Display for AppTheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppTheme::Dark => write!(f, "Dark"),
            AppTheme::Light => write!(f, "Light"),
            AppTheme::System => write!(f, "System"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WalletSortField {
    #[default]
    Balance,
    Name,
    Created,
    Network,
}

impl fmt::Display for WalletSortField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletSortField::Balance => write!(f, "Balance"),
            WalletSortField::Name => write!(f, "Name"),
            WalletSortField::Created => write!(f, "Created"),
            WalletSortField::Network => write!(f, "Network"),
        }
    }
}
