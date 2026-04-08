use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub(crate) contacts: HashMap<String, ContactEntry>,
    /// Auto-incrementing ID counter
    pub(crate) next_id: u64,
}

impl AddressBook {
    /// Create a new empty address book
    pub fn new() -> Self {
        Self {
            contacts: HashMap::new(),
            next_id: 1,
        }
    }
}
