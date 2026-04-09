pub mod dashboard;
pub mod login;
pub mod onboarding;
pub mod settings;
pub mod sidebar;
pub mod transfer;
pub mod wallet;

// Re-export transfer submodules for backward compatibility
pub use transfer::{history, receive, send};

// Re-export wallet as wallets for backward compatibility
pub use wallet as wallets;
