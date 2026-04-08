use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use anyhow::{Context, Result};

use crate::infra::storage::structure::{PersistedState, Storage};
use crate::infra::storage::encryption::{decrypt_blob, encrypt_blob, EncryptedEnvelope};

// ─── State Load/Save ─────────────────────────────────────────────────────

impl Storage {
    pub fn load_state(&self, passphrase: &str) -> Result<PersistedState> {
        if self.paths.encrypted_state_file.exists() {
            return self.load_encrypted_state(&self.paths.encrypted_state_file, passphrase);
        }
        Ok(PersistedState::default())
    }

    pub fn save_state(&self, state: &PersistedState, passphrase: &str) -> Result<()> {
        self.save_encrypted_state(&self.paths.encrypted_state_file, state, passphrase)
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
                    return Err(anyhow::anyhow!("Không xóa được thư mục dữ liệu {}: {}", self.paths.data_dir.display(), err));
                }
            }
        }
        Ok(())
    }

    pub fn export_encrypted_backup(&self, state: &PersistedState, passphrase: &str, path: &Path) -> Result<()> {
        self.save_encrypted_state(path, state, passphrase)
    }

    pub fn import_backup(&self, path: &Path, passphrase: &str) -> Result<PersistedState> {
        let content = fs::read(path).with_context(|| format!("Không đọc được backup file: {}", path.display()))?;
        if let Ok(envelope) = serde_json::from_slice::<EncryptedEnvelope>(&content) {
            let plaintext = decrypt_blob(&envelope, passphrase)?;
            let state: PersistedState = serde_json::from_slice(&plaintext).context("Backup decrypted không đúng định dạng JSON")?;
            return Ok(state);
        }
        let state: PersistedState = serde_json::from_slice(&content).context("Backup không đúng định dạng wallet state")?;
        Ok(state)
    }

    // ─── Private helpers ─────────────────────────────────────────────

    fn save_encrypted_state(&self, path: &Path, state: &PersistedState, passphrase: &str) -> Result<()> {
        let json = serde_json::to_vec_pretty(state).context("Không serialize được wallet state")?;
        let envelope = encrypt_blob(&json, passphrase)?;
        let encoded = serde_json::to_vec_pretty(&envelope).context("Không serialize encrypted payload")?;
        let parent = path.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).with_context(|| format!("Không tạo được thư mục: {}", parent.display()))?;
        let tmp_path = path.with_extension("enc.tmp");
        fs::write(&tmp_path, encoded).with_context(|| format!("Không ghi được file tạm: {}", tmp_path.display()))?;
        fs::rename(&tmp_path, path).with_context(|| format!("Không đổi tên file: {}", path.display()))?;
        Ok(())
    }

    fn load_encrypted_state(&self, path: &Path, passphrase: &str) -> Result<PersistedState> {
        let content = fs::read(path).with_context(|| format!("Không đọc được file encrypted: {}", path.display()))?;
        let envelope: EncryptedEnvelope = serde_json::from_slice(&content).with_context(|| format!("File encrypted không đúng định dạng: {}", path.display()))?;
        let plaintext = decrypt_blob(&envelope, passphrase)?;
        let state: PersistedState = serde_json::from_slice(&plaintext).context("Dữ liệu decrypted không đúng định dạng JSON")?;
        Ok(state)
    }
}

pub fn remove_file_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(anyhow::anyhow!("Không thể xóa file {}: {}", path.display(), err)),
    }
}
