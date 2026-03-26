use std::fs;

use anyhow::{Context, Result, anyhow};


mod encryption;
mod legacy;
mod paths;

pub use self::legacy::PersistedState;
use self::encryption::{EncryptedEnvelope, encrypt_blob, decrypt_blob};
use self::paths::StoragePaths;

#[derive(Debug)]
pub struct Storage {
    paths: StoragePaths,
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

        if let Some(legacy_path) = self.paths.first_existing_legacy_path() {
            let state = self.load_plain_state(legacy_path)?;
            self.save_encrypted_state(&self.paths.encrypted_state_file, &state, passphrase)?;
            self.archive_legacy_file(legacy_path)?;
            return Ok(state);
        }

        Ok(PersistedState::default())
    }

    pub fn save_state(&self, state: &PersistedState, passphrase: &str) -> Result<()> {
        self.save_encrypted_state(&self.paths.encrypted_state_file, state, passphrase)
    }

    pub fn encrypted_state_exists(&self) -> bool {
        self.paths.encrypted_state_file.exists()
    }

    pub fn rotate_passphrase(&self, old_pass: &str, new_pass: &str) -> Result<()> {
        let state = self.load_state(old_pass)?;
        self.save_state(&state, new_pass)
    }

    fn load_plain_state(&self, path: &std::path::Path) -> Result<PersistedState> {
        legacy::load_plain_state(path)
    }

    fn save_encrypted_state(&self, path: &std::path::Path, state: &PersistedState, passphrase: &str) -> Result<()> {
        let json = serde_json::to_vec_pretty(state).context("Không serialize được wallet state")?;
        let envelope = encrypt_blob(&json, passphrase)?;
        let encoded =
            serde_json::to_vec_pretty(&envelope).context("Không serialize encrypted payload")?;

        let parent = path
            .parent()
            .ok_or_else(|| anyhow!("Đường dẫn file state không hợp lệ"))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("Không tạo được thư mục dữ liệu: {}", parent.display()))?;

        let tmp_path = path.with_extension("enc.tmp");
        fs::write(&tmp_path, encoded)
            .with_context(|| format!("Không ghi được file tạm: {}", tmp_path.display()))?;

        fs::rename(&tmp_path, path)
            .with_context(|| format!("Không đổi tên file tạm sang file đích: {}", path.display()))?;

        Ok(())
    }

    fn load_encrypted_state(&self, path: &std::path::Path, passphrase: &str) -> Result<PersistedState> {
        let content = fs::read(path)
            .with_context(|| format!("Không đọc được file encrypted: {}", path.display()))?;

        let envelope: EncryptedEnvelope = serde_json::from_slice(&content)
            .with_context(|| format!("File encrypted không đúng định dạng: {}", path.display()))?;

        let plaintext = decrypt_blob(&envelope, passphrase)?;
        let state: PersistedState = serde_json::from_slice(&plaintext)
            .context("Dữ liệu decrypted không đúng định dạng JSON")?;

        Ok(state)
    }

    fn archive_legacy_file(&self, path: &std::path::Path) -> Result<()> {
        legacy::archive_legacy_file(path)
    }
}

// Re-export for backward compatibility
pub fn load_state(passphrase: &str) -> Result<PersistedState> {
    let storage = Storage::new()?;
    storage.load_state(passphrase)
}

pub fn save_state(state: &PersistedState, passphrase: &str) -> Result<()> {
    let storage = Storage::new()?;
    storage.save_state(state, passphrase)
}

pub fn encrypted_state_exists() -> Result<bool> {
    let storage = Storage::new()?;
    Ok(storage.encrypted_state_exists())
}