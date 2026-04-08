use std::path::PathBuf;

use crate::core::contact::structure::AddressBook;
use crate::infra::paths::data_directory_path;

// ─── File Persistence ────────────────────────────────────────────────────

impl AddressBook {
    /// Get the file path for address book storage
    pub fn file_path() -> PathBuf {
        let data_dir = data_directory_path().unwrap_or_else(|_| {
            std::env::current_dir().unwrap_or_default()
        });
        data_dir.join("address_book.json")
    }

    /// Save address book to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::file_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create data directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize address book: {}", e))?;

        let temp_path = path.with_extension("json.tmp");
        std::fs::write(&temp_path, &json)
            .map_err(|e| format!("Failed to write address book file: {}", e))?;

        std::fs::rename(&temp_path, &path)
            .map_err(|e| format!("Failed to save address book file: {}", e))?;

        Ok(())
    }

    /// Load address book from disk
    pub fn load() -> Result<Self, String> {
        let path = Self::file_path();

        if !path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read address book file: {}", e))?;

        let book: AddressBook = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse address book file: {}", e))?;

        Ok(book)
    }
}
