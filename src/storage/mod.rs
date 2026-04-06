use std::{fs, io::ErrorKind, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

mod encryption;
mod paths;

pub use self::encryption::{decrypt_blob, encrypt_blob, EncryptedEnvelope};
use self::paths::StoragePaths;
use crate::i18n::AppLanguage;
use crate::wallet::{
    runtime_wallets_from_stored, stored_wallets_from_runtime, StoredWallet, Wallet,
    WalletSecretsVault,
};

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
#[derive(Debug)]
pub struct Storage {
    paths: StoragePaths,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppPreferences {
    #[serde(default)]
    language: AppLanguage,
    #[serde(default)]
    last_selected_wallet: Option<usize>,
    #[serde(default)]
    last_viewed_page: Option<String>,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            language: AppLanguage::English,
            last_selected_wallet: None,
            last_viewed_page: None,
        }
    }
}

impl Storage {
    pub fn new() -> Result<Self> {
        let paths = StoragePaths::resolve()?;
        Ok(Self { paths })
    }

    pub fn load_state(&self, passphrase: &str) -> Result<PersistedState> {
        if self.paths.encrypted_state_file.exists() {
            return self.load_encrypted_state(&self.paths.encrypted_state_file, passphrase);
        }
        Ok(PersistedState::default())
    }

    pub fn save_state(&self, state: &PersistedState, passphrase: &str) -> Result<()> {
        self.save_encrypted_state(&self.paths.encrypted_state_file, state, passphrase)
    }

    pub fn has_existing_state(&self) -> bool {
        self.paths.encrypted_state_file.exists()
    }

    pub fn load_language_preference(&self) -> Result<AppLanguage> {
        if !self.paths.preferences_file.exists() {
            return Ok(AppLanguage::English);
        }

        let content = fs::read_to_string(&self.paths.preferences_file).with_context(|| {
            format!(
                "Không đọc được file cài đặt app: {}",
                self.paths.preferences_file.display()
            )
        })?;

        let prefs: AppPreferences = serde_json::from_str(&content).with_context(|| {
            format!(
                "File cài đặt app không đúng định dạng JSON: {}",
                self.paths.preferences_file.display()
            )
        })?;

        Ok(prefs.language)
    }

    pub fn save_language_preference(&self, language: AppLanguage) -> Result<()> {
        let current_prefs = self.load_preferences()?;
        let prefs = AppPreferences {
            language,
            last_selected_wallet: current_prefs.last_selected_wallet,
            last_viewed_page: current_prefs.last_viewed_page,
        };
        self.save_preferences(&prefs)
    }

    pub fn save_wallet_selection(&self, wallet_index: usize, page: &str) -> Result<()> {
        let prefs = AppPreferences {
            language: self.load_language_preference()?,
            last_selected_wallet: Some(wallet_index),
            last_viewed_page: Some(page.to_string()),
        };
        self.save_preferences(&prefs)
    }

    pub fn load_wallet_selection(&self) -> Result<(Option<usize>, Option<String>)> {
        let prefs = self.load_preferences()?;
        Ok((prefs.last_selected_wallet, prefs.last_viewed_page))
    }

    fn save_preferences(&self, prefs: &AppPreferences) -> Result<()> {
        let encoded =
            serde_json::to_vec_pretty(prefs).context("Không serialize được app preferences")?;

        let parent = self
            .paths
            .preferences_file
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)
            .with_context(|| format!("Không tạo được thư mục dữ liệu: {}", parent.display()))?;

        let tmp_path = self.paths.preferences_file.with_extension("json.tmp");
        fs::write(&tmp_path, encoded)
            .with_context(|| format!("Không ghi được file tạm: {}", tmp_path.display()))?;
        fs::rename(&tmp_path, &self.paths.preferences_file).with_context(|| {
            format!(
                "Không đổi tên file tạm sang file đích: {}",
                self.paths.preferences_file.display()
            )
        })?;

        Ok(())
    }

    fn load_preferences(&self) -> Result<AppPreferences> {
        if !self.paths.preferences_file.exists() {
            return Ok(AppPreferences::default());
        }

        let encoded = fs::read_to_string(&self.paths.preferences_file)
            .with_context(|| {
                format!(
                    "Không đọc được file preferences: {}",
                    self.paths.preferences_file.display()
                )
            })?;

        let prefs: AppPreferences = serde_json::from_str(&encoded).context("Không parse được app preferences")?;

        Ok(prefs)
    }

    pub fn rotate_passphrase(&self, old_pass: &str, new_pass: &str) -> Result<()> {
        let state = self.load_state(old_pass)?;
        self.save_state(&state, new_pass)
    }

    pub fn clear_all_data(&self) -> Result<()> {
        remove_file_if_exists(&self.paths.encrypted_state_file)?;

        if self.paths.data_dir.exists() {
            match fs::remove_dir_all(&self.paths.data_dir) {
                Ok(_) => {}
                Err(err) if err.kind() == ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(anyhow::anyhow!(
                        "Không xóa được thư mục dữ liệu {}: {}",
                        self.paths.data_dir.display(),
                        err
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn export_encrypted_backup(
        &self,
        state: &PersistedState,
        passphrase: &str,
        path: &Path,
    ) -> Result<()> {
        self.save_encrypted_state(path, state, passphrase)
    }

    pub fn import_backup(&self, path: &Path, passphrase: &str) -> Result<PersistedState> {
        let content = fs::read(path)
            .with_context(|| format!("Không đọc được backup file: {}", path.display()))?;

        if let Ok(envelope) = serde_json::from_slice::<EncryptedEnvelope>(&content) {
            let plaintext = decrypt_blob(&envelope, passphrase)?;
            let state: PersistedState = serde_json::from_slice(&plaintext)
                .context("Backup decrypted không đúng định dạng JSON")?;
            return Ok(state);
        }

        let state: PersistedState =
            serde_json::from_slice(&content).context("Backup không đúng định dạng wallet state")?;
        Ok(state)
    }

    fn save_encrypted_state(
        &self,
        path: &std::path::Path,
        state: &PersistedState,
        passphrase: &str,
    ) -> Result<()> {
        let json = serde_json::to_vec_pretty(state).context("Không serialize được wallet state")?;
        let envelope = encrypt_blob(&json, passphrase)?;
        let encoded =
            serde_json::to_vec_pretty(&envelope).context("Không serialize encrypted payload")?;

        let parent = path
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)
            .with_context(|| format!("Không tạo được thư mục dữ liệu: {}", parent.display()))?;

        let tmp_path = path.with_extension("enc.tmp");
        fs::write(&tmp_path, encoded)
            .with_context(|| format!("Không ghi được file tạm: {}", tmp_path.display()))?;

        fs::rename(&tmp_path, path).with_context(|| {
            format!("Không đổi tên file tạm sang file đích: {}", path.display())
        })?;

        Ok(())
    }

    fn load_encrypted_state(
        &self,
        path: &std::path::Path,
        passphrase: &str,
    ) -> Result<PersistedState> {
        let content = fs::read(path)
            .with_context(|| format!("Không đọc được file encrypted: {}", path.display()))?;

        let envelope: EncryptedEnvelope = serde_json::from_slice(&content)
            .with_context(|| format!("File encrypted không đúng định dạng: {}", path.display()))?;

        let plaintext = decrypt_blob(&envelope, passphrase)?;
        let state: PersistedState = serde_json::from_slice(&plaintext)
            .context("Dữ liệu decrypted không đúng định dạng JSON")?;

        Ok(state)
    }
}

fn remove_file_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(anyhow::anyhow!(
            "Không thể xóa file {}: {}",
            path.display(),
            err
        )),
    }
}
