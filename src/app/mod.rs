mod auth;
mod transfer_ops;
mod settings_ops;
mod wallet_ops;

mod structure;
mod lifecycle;
mod helpers;
mod subscription;
mod update;
mod view;
mod refresh;

pub use structure::*;

use std::time::Duration;

use chrono::Local;
use iced::{
    clipboard, keyboard,
    widget::{column, container, row, text, Space},
    Element, Length, Subscription, Task, Theme,
};
use secrecy::{ExposeSecret, SecretString};

use crate::components::{Toast, ToastManager, shortcuts_help_popup, modal, error_card};
use crate::error::AppError;
use crate::i18n::{set_current_language, t, AppLanguage};
use crate::storage::{AppTheme, PersistedState, RuntimeState, Storage, UserProfile, AddressBook};
use crate::theme::{
    screen_background_style, text_color, Colors,
    get_theme_colors,
};
use crate::utils::{normalize_nickname, wallet_count_text};
use crate::views::{
    dashboard::{DashboardMessage, DashboardView},
    history::{HistoryEvent, HistoryMessage, HistoryView},
    language_selector::LanguageSelector,
    login::{LoginMessage, LoginMode, LoginView},
    onboarding::{OnboardingMessage, OnboardingView},
    receive::{ReceiveMessage, ReceiveView},
    send::{SendMessage, SendView},
    settings::{SettingsMessage, SettingsView},
    sidebar::{NavItem, Sidebar, SidebarEvent, SidebarMessage},
    wallets::{WalletsMessage, WalletsView},
};
use crate::wallet::{Wallet, WalletSecretsRef, WalletSecretsVault};

const REVEALED_MNEMONIC_TTL_SECS: u64 = 60;
