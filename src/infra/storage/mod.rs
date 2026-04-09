mod encryption;
mod operations;
mod preferences;
mod structure;

pub use encryption::{decrypt_blob, encrypt_blob, EncryptedEnvelope};
pub use operations::remove_file_if_exists;
pub use structure::{
    AppPreferences, AppTheme, PersistedState, RuntimeState, Storage, UserProfile, WalletSortField,
};

// Re-export AddressBook from core/contact for backward compatibility
pub use crate::core::contact::{AddressBook, ContactEntry};
