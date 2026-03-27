use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::wallet::WalletEntry;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserProfile {
    #[serde(default)]
    pub nickname: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PersistedState {
    #[serde(default)]
    pub profile: UserProfile,
    #[serde(default)]
    pub wallets: Vec<WalletEntry>,
}

const LEGACY_DATA_FILE: &str = "wallet_data.json";

pub fn load_plain_state(path: &Path) -> Result<PersistedState> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Không đọc được file legacy: {}", path.display()))?;

    let state: PersistedState = serde_json::from_str(&content)
        .with_context(|| format!("Không parse được JSON legacy: {}", path.display()))?;

    Ok(state)
}

pub fn archive_legacy_file(path: &Path) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(LEGACY_DATA_FILE);
    let backup_path = path.with_file_name(format!("{file_name}.migrated.bak"));

    if backup_path.exists() {
        fs::remove_file(path).with_context(|| {
            format!(
                "Không xóa được file legacy sau khi đã có backup: {}",
                path.display()
            )
        })?;
        return Ok(());
    }

    match fs::rename(path, &backup_path) {
        Ok(_) => Ok(()),
        Err(_) => {
            fs::copy(path, &backup_path).with_context(|| {
                format!(
                    "Không copy được file legacy sang backup: {}",
                    backup_path.display()
                )
            })?;
            fs::remove_file(path).with_context(|| {
                format!(
                    "Không xóa được file legacy sau khi copy backup: {}",
                    path.display()
                )
            })?;
            Ok(())
        }
    }
}
