// Re-export từ ui/views/ và ui/components/ để backward compatibility
pub use crate::ui::views::*;
pub use crate::ui::views::transfer::{send, receive, history};
pub use crate::ui::views::wallet;
pub use crate::ui::views::wallet as wallets;  // Alias for backward compatibility
pub use crate::ui::components::{language_selector, wallet_picker};
pub use crate::ui::components::{LanguageSelector, WalletChoice, wallet_choices, selected_wallet_choice};
