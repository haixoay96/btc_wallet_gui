use secrecy::SecretString;

use crate::core::error::AppError;
use crate::core::wallet::{Wallet, WalletSecretsVault};
use crate::infra::storage::{AddressBook, AppTheme};
use crate::ui::components::language_selector::LanguageSelector;
use crate::ui::components::ToastManager;
use crate::ui::i18n::AppLanguage;
use crate::ui::views::{
    dashboard::{DashboardMessage, DashboardView},
    history::{HistoryMessage, HistoryView},
    login::{LoginMessage, LoginView},
    onboarding::{OnboardingMessage, OnboardingView},
    receive::{ReceiveMessage, ReceiveView},
    send::{SendMessage, SendView},
    settings::{SettingsMessage, SettingsView},
    sidebar::{NavItem, Sidebar, SidebarMessage},
    wallets::{WalletsMessage, WalletsView},
};

/// TTL for revealed mnemonic in seconds
#[allow(dead_code)]
const REVEALED_MNEMONIC_TTL_SECS: u64 = 60;

pub enum AppState {
    Login,
    Main,
}

pub struct App {
    pub state: AppState,
    pub storage_passphrase: Option<SecretString>,
    pub language: AppLanguage,
    pub theme: AppTheme,
    pub high_contrast: bool,
    pub font_scale: f64,
    pub user_nickname: Option<String>,
    pub wallets: Vec<Wallet>,
    pub wallet_vault: WalletSecretsVault,
    pub selected_wallet: usize,
    pub btc_price: Option<crate::infra::price_api::BtcPriceData>,
    pub is_fetching_price: bool,

    pub login_view: LoginView,
    pub sidebar: Sidebar,
    pub dashboard: DashboardView,
    pub wallets_view: WalletsView,
    pub send_view: SendView,
    pub receive_view: ReceiveView,
    pub history_view: HistoryView,
    pub settings_view: SettingsView,
    pub language_selector: LanguageSelector,

    pub current_page: NavItem,
    pub error: Option<AppError>,
    pub is_refreshing: bool,
    pub is_estimating_fee: bool,
    pub is_calculating_max: bool,
    pub is_sending: bool,
    pub revealed_mnemonic_session_id: u64,

    // Toast notification system
    pub toast_manager: ToastManager,

    // Copy tracking
    pub last_copied_address: Option<String>,
    pub last_copied_time: Option<String>,

    // Keyboard shortcuts help popup
    pub show_shortcuts_help: bool,

    // Keyboard focus tracking
    pub focus_search_history: bool,
    pub focus_paste_send: bool,

    // Onboarding
    pub show_onboarding: bool,
    pub onboarding_view: OnboardingView,

    // Address Book / Contact Book
    pub address_book: AddressBook,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    LoginMessage(LoginMessage),
    SidebarMessage(SidebarMessage),
    DashboardMessage(DashboardMessage),
    WalletsMessage(WalletsMessage),
    SendMessage(SendMessage),
    ReceiveMessage(ReceiveMessage),
    HistoryMessage(HistoryMessage),
    SettingsMessage(SettingsMessage),
    LanguageChanged(AppLanguage),
    RefreshWalletsFinished(Result<RefreshWalletsResult, String>),
    EstimateSendFeeFinished(Result<u64, String>),
    MaxAmountFinished(Result<(u64, u64), String>),
    SendTransactionFinished(Result<SendExecutionResult, String>),
    RevealedMnemonicExpired(u64),
    ToastCleanup,
    ToggleShortcutsHelp,
    ExportFinished(Result<(), String>),
    GlobalEscKey,
    DismissStatus,
    KeyboardCopy,
    KeyboardPaste,
    KeyboardSubmitForm,
    KeyboardSaveState,
    KeyboardFocusSearch,
    AutoRefreshConfirmations,
    OnboardingMessage(OnboardingMessage),
}

#[derive(Debug, Clone)]
pub struct RefreshWalletsResult {
    pub wallets: Vec<Wallet>,
    pub refreshed_wallets: usize,
    pub refreshed_txs: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SendExecutionResult {
    pub wallet_id: String,
    pub wallet: Wallet,
    pub txid: String,
    pub broadcasted: bool,
}
