// Re-export từ infra/storage/ để backward compatibility
pub use crate::infra::paths::{data_directory_path, StoragePaths};
pub use crate::infra::storage::{
    decrypt_blob, encrypt_blob, AppPreferences, AppTheme, EncryptedEnvelope, PersistedState,
    RuntimeState, Storage, UserProfile, WalletSortField,
};

// Address book vẫn giữ nguyên (sẽ refactor sau)
pub mod address_book;
pub use self::address_book::AddressBook;

// Encryption, paths modules giữ lại cho compatibility
pub mod encryption;
pub mod paths;
