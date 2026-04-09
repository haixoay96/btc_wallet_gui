use crate::core::contact::structure::{AddressBook, ContactEntry};

// ─── CRUD Operations ─────────────────────────────────────────────────────

impl AddressBook {
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

    /// Get all contacts (sorted by name)
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
        let mut results: Vec<&ContactEntry> = self
            .contacts
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_contact() {
        let mut book = AddressBook::new();
        let id = book.add_contact("Alice", "bc1qabc", "Friend");
        assert!(book.get_contact(&id).is_some());
        assert_eq!(book.get_all_contacts().len(), 1);
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
        assert_eq!(book.get_all_contacts().len(), 0);
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
        let contact = book.get_all_contacts()[0];

        let mut new_book = AddressBook::new();
        new_book
            .contacts
            .insert(contact.id.clone(), contact.clone());

        assert_eq!(new_book.get_all_contacts().len(), 1);
    }
}
