// Re-export từ ui/views/ và ui/components/ để backward compatibility
pub use crate::ui::components::{language_selector, wallet_picker};
pub use crate::ui::components::{
    selected_wallet_choice, wallet_choices, LanguageSelector, WalletChoice,
};
pub use crate::ui::views::transfer::{history, receive, send};
pub use crate::ui::views::wallet;
pub use crate::ui::views::wallet as wallets; // Alias for backward compatibility
pub use crate::ui::views::*;
