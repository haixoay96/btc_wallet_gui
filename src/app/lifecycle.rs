use std::time::Duration;

use iced::Task;
use secrecy::{ExposeSecret, SecretString};

use crate::app::structure::{App, AppMessage, AppState};
use crate::core::wallet::WalletSecretsVault;
use crate::infra::price_api::BtcPriceData;
use crate::infra::storage::{
    AddressBook, AppTheme, PersistedState, RuntimeState, Storage, UserProfile,
};
use crate::ui::components::language_selector::LanguageSelector;
use crate::ui::components::network_status::DashboardNetworkMessage;
use crate::ui::components::price_widget::structure::PriceWidgetMessage;
use crate::ui::components::ToastManager;
use crate::ui::i18n::{set_current_language, t, AppLanguage};
use crate::ui::views::{
    dashboard::{DashboardMessage, DashboardView},
    history::HistoryView,
    login::{LoginMode, LoginView},
    onboarding::OnboardingView,
    receive::ReceiveView,
    send::SendView,
    settings::SettingsView,
    sidebar::{NavItem, Sidebar},
    wallets::WalletsView,
};
use crate::utils::normalize_nickname;

impl App {
    pub fn new() -> (Self, Task<AppMessage>) {
        let fallback_language = AppLanguage::English;
        let (
            initial_language,
            has_existing_state,
            initial_theme,
            initial_high_contrast,
            initial_font_scale,
            onboarding_completed,
            initial_btc_price,
        ) = match Storage::new() {
            Ok(storage) => {
                let language = storage
                    .load_language_preference()
                    .unwrap_or(fallback_language);
                let theme = storage.load_theme().unwrap_or(AppTheme::Dark);
                let high_contrast = storage.load_high_contrast().unwrap_or(false);
                let font_scale = storage.load_font_scale().unwrap_or(1.0);
                let onboarding = storage.load_onboarding_completed().unwrap_or(false);
                // Load cached BTC price
                let btc_price = storage
                    .load_cached_btc_price()
                    .ok()
                    .flatten()
                    .zip(
                        storage
                            .load_preferences()
                            .ok()
                            .and_then(|p| p.cached_btc_change_24h),
                    )
                    .map(|(price, change)| BtcPriceData {
                        price_usd: price,
                        change_24h: change,
                    });
                // Set global states
                crate::ui::theme::set_high_contrast(high_contrast);
                crate::ui::theme::set_font_scale(font_scale);
                (
                    language,
                    storage.has_existing_state(),
                    theme,
                    high_contrast,
                    font_scale,
                    onboarding,
                    btc_price,
                )
            }
            Err(_) => (
                fallback_language,
                false,
                AppTheme::Dark,
                false,
                1.0,
                false,
                None,
            ),
        };
        set_current_language(initial_language);

        let mut login_view = LoginView::new();
        login_view.set_can_create_new_passphrase(!has_existing_state);
        if !has_existing_state {
            login_view.set_mode(LoginMode::NewWallet);
        }

        (
            Self {
                state: AppState::Login,
                storage_passphrase: None,
                language: initial_language,
                theme: initial_theme,
                high_contrast: initial_high_contrast,
                font_scale: initial_font_scale,
                user_nickname: None,
                wallets: Vec::new(),
                wallet_vault: WalletSecretsVault::new(),
                selected_wallet: 0,
                btc_price: initial_btc_price,
                is_fetching_price: false,
                login_view,
                sidebar: Sidebar::new(),
                dashboard: DashboardView::new(),
                wallets_view: WalletsView::new(),
                send_view: SendView::new(),
                receive_view: ReceiveView::new(),
                history_view: HistoryView::new(),
                settings_view: SettingsView::new(),
                language_selector: LanguageSelector::new(),
                current_page: NavItem::Dashboard,
                error: None,
                is_refreshing: false,
                is_estimating_fee: false,
                is_calculating_max: false,
                is_sending: false,
                revealed_mnemonic_session_id: 0,
                toast_manager: ToastManager::new(),
                last_copied_address: None,
                last_copied_time: None,
                show_shortcuts_help: false,
                focus_search_history: false,
                focus_paste_send: false,
                show_onboarding: !has_existing_state && !onboarding_completed,
                onboarding_view: OnboardingView::new(),
                address_book: AddressBook::load().unwrap_or_default(),
            },
            // Start toast cleanup + initial network check + price fetch
            Task::batch([
                Task::perform(
                    async move {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    },
                    |_| AppMessage::ToastCleanup,
                ),
                Task::done(AppMessage::DashboardMessage(DashboardMessage::Network(
                    DashboardNetworkMessage::CheckConnection,
                ))),
                // Trigger initial price fetch on startup
                Task::done(AppMessage::DashboardMessage(DashboardMessage::PriceWidget(
                    PriceWidgetMessage::RefreshPrice,
                ))),
            ]),
        )
    }

    pub fn title(&self) -> String {
        t("Ví Bitcoin", "Bitcoin Wallet").to_string()
    }

    pub fn save_language_preference(&mut self) {
        let result =
            Storage::new().and_then(|storage| storage.save_language_preference(self.language));
        if let Err(err) = result {
            let _path = match Storage::new() {
                Ok(s) => s.file_path().display().to_string(),
                Err(_) => "unknown".to_string(),
            };
            self.add_error_toast(format!(
                "{}: {err}",
                t(
                    "Không thể lưu cài đặt ngôn ngữ",
                    "Could not save language preference"
                )
            ));
        }
    }

    pub fn save_state(&mut self) -> bool {
        let passphrase = match &self.storage_passphrase {
            Some(value) => value,
            None => return false,
        };

        match Storage::new() {
            Ok(storage) => {
                let state = match self.persisted_state() {
                    Ok(state) => state,
                    Err(err) => {
                        self.add_error_toast(format!(
                            "{}: {err}",
                            t(
                                "Không thể gom dữ liệu ví",
                                "Failed to assemble wallet state"
                            )
                        ));
                        return false;
                    }
                };
                if let Err(err) = storage.save_state(&state, passphrase.expose_secret()) {
                    self.add_error_toast(format!(
                        "{}: {err}",
                        t("Không thể lưu trạng thái", "Failed to save app state")
                    ));
                    false
                } else {
                    // Also save wallet selection
                    let _ = storage
                        .save_wallet_selection(self.selected_wallet, self.current_page.title());
                    true
                }
            }
            Err(err) => {
                self.add_error_toast(format!(
                    "{}: {err}",
                    t("Không thể khởi tạo storage", "Failed to initialize storage")
                ));
                false
            }
        }
    }

    pub fn reset_to_login(&mut self, allow_create_passphrase: bool) {
        self.state = AppState::Login;
        self.storage_passphrase = None;
        self.user_nickname = None;
        self.wallets.clear();
        self.wallet_vault.clear();
        self.selected_wallet = 0;
        self.current_page = NavItem::Dashboard;
        self.is_refreshing = false;
        self.is_estimating_fee = false;
        self.is_calculating_max = false;
        self.is_sending = false;
        self.revealed_mnemonic_session_id = 0;
        self.toast_manager = ToastManager::new();
        self.last_copied_address = None;
        self.last_copied_time = None;
        self.show_shortcuts_help = false;

        self.login_view = LoginView::new();
        self.login_view
            .set_can_create_new_passphrase(allow_create_passphrase);
        self.sidebar = Sidebar::new();
        self.dashboard = DashboardView::new();
        self.wallets_view = WalletsView::new();
        self.send_view = SendView::new();
        self.receive_view = ReceiveView::new();
        self.history_view = HistoryView::new();
        self.settings_view = SettingsView::new();
        self.language_selector = LanguageSelector::new();
    }

    pub fn persisted_state(&self) -> anyhow::Result<PersistedState> {
        PersistedState::from_runtime(
            UserProfile {
                nickname: self.user_nickname.clone(),
                language: self.language,
            },
            &self.wallets,
            &self.wallet_vault,
        )
    }

    pub fn restore_runtime_state(&mut self, state: RuntimeState) {
        self.user_nickname = normalize_nickname(state.profile.nickname.as_deref());
        self.language = state.profile.language;
        set_current_language(self.language);
        self.save_language_preference();
        self.wallets = state.wallets;
        self.wallet_vault = state.wallet_vault;
        self.state = AppState::Main;

        // Restore persistent wallet selection if available
        if let Ok(storage) = Storage::new() {
            if let Ok((Some(wallet_idx), Some(page_title))) = storage.load_wallet_selection() {
                if wallet_idx < self.wallets.len() {
                    self.selected_wallet = wallet_idx;
                    // Restore page
                    self.current_page = match page_title.as_str() {
                        "Tổng quan" | "Dashboard" => NavItem::Dashboard,
                        "Ví" | "Wallets" => NavItem::Wallets,
                        "Gửi" | "Send" => NavItem::Send,
                        "Nhận" | "Receive" => NavItem::Receive,
                        "Lịch sử" | "History" => NavItem::History,
                        "Cài đặt" | "Settings" => NavItem::Settings,
                        _ => NavItem::Dashboard,
                    };
                    self.sidebar.set_active(self.current_page);
                }
            }
        }

        self.selected_wallet = self
            .selected_wallet
            .min(self.wallets.len().saturating_sub(1));
        self.update_dashboard();
        self.login_view.clear_error();
    }

    pub fn current_passphrase(&self) -> Option<&SecretString> {
        self.storage_passphrase.as_ref()
    }
}
