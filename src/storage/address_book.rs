use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::i18n::t;

/// Contact entry in the address book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactEntry {
    pub id: String,
    pub name: String,
    pub address: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub created_at: u64,
}

/// Address Book / Contact Book storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddressBook {
    /// Map of contact_id -> ContactEntry
    contacts: HashMap<String, ContactEntry>,
    /// Auto-incrementing ID counter
    next_id: u64,
}

impl AddressBook {
    pub fn new() -> Self {
        Self {
            contacts: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a new contact
    pub fn add_contact(&mut self, name: &str, address: &str, note: &str) -> String {
        let id = format!("contact_{}", self.next_id);
        self.next_id += 1;
        
        let contact = ContactEntry {
            id: id.clone(),
            name: name.trim().to_string(),
            address: address.trim().to_string(),
            note: note.trim().to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        self.contacts.insert(id.clone(), contact);
        id
    }

    /// Update an existing contact
    pub fn update_contact(&mut self, id: &str, name: &str, address: &str, note: &str) {
        if let Some(contact) = self.contacts.get_mut(id) {
            contact.name = name.trim().to_string();
            contact.address = address.trim().to_string();
            contact.note = note.trim().to_string();
        }
    }

    /// Delete a contact
    pub fn delete_contact(&mut self, id: &str) {
        self.contacts.remove(id);
    }

    /// Get a contact by ID
    pub fn get_contact(&self, id: &str) -> Option<&ContactEntry> {
        self.contacts.get(id)
    }

    /// Find contact by exact address match
    pub fn find_by_address(&self, address: &str) -> Option<&ContactEntry> {
        self.contacts.values().find(|c| c.address == address)
    }

    /// Get all contacts
    pub fn get_all_contacts(&self) -> Vec<&ContactEntry> {
        let mut contacts: Vec<&ContactEntry> = self.contacts.values().collect();
        contacts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        contacts
    }

    /// Search contacts by name, address, or note (case-insensitive)
    pub fn search(&self, query: &str) -> Vec<&ContactEntry> {
        if query.trim().is_empty() {
            return self.get_all_contacts();
        }
        let query_lower = query.to_lowercase();
        let mut results: Vec<&ContactEntry> = self.contacts
            .values()
            .filter(|c| {
                c.name.to_lowercase().contains(&query_lower)
                    || c.address.to_lowercase().contains(&query_lower)
                    || c.note.to_lowercase().contains(&query_lower)
            })
            .collect();
        results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        results
    }

    /// Get contact count
    pub fn count(&self) -> usize {
        self.contacts.len()
    }

    /// Export contacts to JSON string
    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.contacts)
            .map_err(|e| format!("Failed to export contacts: {}", e))
    }

    /// Import contacts from JSON string (merges with existing)
    pub fn import_json(&mut self, json: &str) -> Result<usize, String> {
        let imported: HashMap<String, ContactEntry> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to import contacts: {}", e))?;
        
        let count = imported.len();
        self.contacts.extend(imported);
        Ok(count)
    }

    /// Clear all contacts
    pub fn clear_all(&mut self) {
        self.contacts.clear();
        self.next_id = 1;
    }
}

/// Address Book file persistence
impl AddressBook {
    /// Get the file path for address book storage
    pub fn file_path() -> std::path::PathBuf {
        let data_dir = crate::storage::paths::data_directory_path().unwrap_or_else(|_| {
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

    /// Delete address book file
    pub fn delete() -> Result<(), String> {
        let path = Self::file_path();
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete address book file: {}", e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_contact() {
        let mut book = AddressBook::new();
        let id = book.add_contact("Alice", "bc1qabc", "Friend");
        assert!(book.get_contact(&id).is_some());
        assert_eq!(book.count(), 1);
    }

    #[test]
    fn test_update_contact() {
        let mut book = AddressBook::new();
        let id = book.add_contact("Alice", "bc1qabc", "");
        book.update_contact(&id, "Alice Smith", "bc1qnew", "Updated");
        let contact = book.get_contact(&id).unwrap();
        assert_eq!(contact.name, "Alice Smith");
        assert_eq!(contact.address, "bc1qnew");
    }

    #[test]
    fn test_delete_contact() {
        let mut book = AddressBook::new();
        let id = book.add_contact("Bob", "bc1qbob", "");
        book.delete_contact(&id);
        assert_eq!(book.count(), 0);
    }

    #[test]
    fn test_search_contacts() {
        let mut book = AddressBook::new();
        book.add_contact("Alice", "bc1qalice", "Binance");
        book.add_contact("Bob", "bc1qbob", "Coinbase");
        book.add_contact("Charlie", "bc1qcharlie", "Binance Deposit");
        
        let results = book.search("binance");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_export_import() {
        let mut book = AddressBook::new();
        book.add_contact("Test", "bc1qtest", "Note");
        
        let json = book.export_json().unwrap();
        let mut new_book = AddressBook::new();
        new_book.import_json(&json).unwrap();
        
        assert_eq!(new_book.count(), 1);
    }
}
