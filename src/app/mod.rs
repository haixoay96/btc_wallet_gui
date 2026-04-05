mod login;
mod send;
mod settings;
mod wallet;

use std::time::Duration;

use chrono::Local;
use iced::{
    clipboard,
    widget::{column, container, row, text, Space},
    Element, Length, Task,
};
use secrecy::{ExposeSecret, SecretString};

use crate::components::{Toast, ToastManager};
use crate::i18n::{set_current_language, t, AppLanguage};
use crate::storage::{PersistedState, RuntimeState, Storage, UserProfile};
use crate::theme::{
    notice_style, screen_background_style, secondary_button_style, text_color, Colors, NoticeTone,
};
use crate::utils::{normalize_nickname, wallet_count_text};
use crate::views::{
    dashboard::{DashboardMessage, DashboardView},
    history::{HistoryEvent, HistoryMessage, HistoryView},
    language_selector::LanguageSelector,
    login::{LoginMessage, LoginMode, LoginView},
    receive::{ReceiveMessage, ReceiveView},
    send::{SendMessage, SendView},
    settings::{SettingsMessage, SettingsView},
    sidebar::{NavItem, Sidebar, SidebarEvent, SidebarMessage},
    wallets::{WalletsMessage, WalletsView},
};
use crate::wallet::{Wallet, WalletSecretsRef, WalletSecretsVault};

const REVEALED_MNEMONIC_TTL_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub enum AppState {
    Login,
    Main,
}

pub struct App {
    pub state: AppState,
    pub storage_passphrase: Option<SecretString>,
    pub language: AppLanguage,
    pub user_nickname: Option<String>,
    pub wallets: Vec<Wallet>,
    pub wallet_vault: WalletSecretsVault,
    pub selected_wallet: usize,

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
    pub status: Option<String>,
    pub error: Option<String>,
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
    DismissStatus,
    DismissError,
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

impl App {
    pub fn new() -> (Self, Task<AppMessage>) {
        let fallback_language = AppLanguage::English;
        let (initial_language, has_existing_state) = match Storage::new() {
            Ok(storage) => {
                let language = storage
                    .load_language_preference()
                    .unwrap_or(fallback_language);
                (language, storage.has_existing_state())
            }
            Err(_) => (fallback_language, false),
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
                user_nickname: None,
                wallets: Vec::new(),
                wallet_vault: WalletSecretsVault::new(),
                selected_wallet: 0,
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
                status: None,
                error: None,
                is_refreshing: false,
                is_estimating_fee: false,
                is_calculating_max: false,
                is_sending: false,
                revealed_mnemonic_session_id: 0,
                toast_manager: ToastManager::new(),
                last_copied_address: None,
                last_copied_time: None,
            },
            // Start toast cleanup task
            Task::perform(
                async move {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                },
                |_| AppMessage::ToastCleanup,
            ),
        )
    }

    pub fn title(&self) -> String {
        t("Ví Bitcoin", "Bitcoin Wallet").to_string()
    }

    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::LoginMessage(msg) => self.handle_login_message(msg),

            AppMessage::SidebarMessage(msg) => {
                let event = self.sidebar.update(msg);
                match event {
                    SidebarEvent::Navigate(page) => {
                        self.current_page = page;
                        self.sidebar.set_active(page);
                    }
                }
                Task::none()
            }

            AppMessage::DashboardMessage(DashboardMessage::Refresh) => self.refresh_all_wallets(),
            AppMessage::DashboardMessage(DashboardMessage::Navigate(page)) => {
                self.current_page = page;
                self.sidebar.set_active(page);
                Task::none()
            }

            AppMessage::WalletsMessage(msg) => self.handle_wallets_message(msg),
            AppMessage::SendMessage(msg) => self.handle_send_message(msg),
            AppMessage::ReceiveMessage(msg) => self.handle_receive_message(msg),

            AppMessage::HistoryMessage(msg) => match msg {
                HistoryMessage::SelectWallet(index) => self.handle_select_wallet(index),
                HistoryMessage::CopyTxid(txid) => clipboard::write(txid),
                HistoryMessage::OpenExplorer(url) => {
                    #[cfg(target_os = "linux")]
                    let result = std::process::Command::new("xdg-open").arg(&url).spawn();

                    #[cfg(target_os = "macos")]
                    let result = std::process::Command::new("open").arg(&url).spawn();

                    #[cfg(target_os = "windows")]
                    let result = std::process::Command::new("cmd")
                        .args(["/C", "start", &url])
                        .spawn();

                    if let Err(e) = result {
                        self.error = Some(format!("Không thể mở trình duyệt: {}", e));
                    }
                    Task::none()
                }
                _ => {
                    if let Some(event) = self.history_view.update(msg) {
                        match event {
                            HistoryEvent::Refresh => return self.refresh_all_wallets(),
                        }
                    }
                    Task::none()
                }
            },

            AppMessage::SettingsMessage(msg) => self.handle_settings_message(msg),

            AppMessage::LanguageChanged(language) => self.handle_change_language(language),
            AppMessage::RefreshWalletsFinished(result) => {
                self.handle_refresh_wallets_finished(result)
            }
            AppMessage::EstimateSendFeeFinished(result) => {
                self.handle_estimate_send_fee_finished(result)
            }
            AppMessage::MaxAmountFinished(result) => self.handle_max_amount_finished(result),
            AppMessage::SendTransactionFinished(result) => {
                self.handle_send_transaction_finished(result)
            }
            AppMessage::RevealedMnemonicExpired(session_id) => {
                self.handle_revealed_mnemonic_expired(session_id)
            }
            AppMessage::ToastCleanup => {
                self.toast_manager.cleanup_expired();
                // Schedule next cleanup in 2 seconds
                Task::perform(
                    async move {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    },
                    |_| AppMessage::ToastCleanup,
                )
            }
            AppMessage::DismissStatus => {
                Task::none()
            }
            AppMessage::DismissError => {
                self.error = None;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, AppMessage> {
        match self.state {
            AppState::Login => container(self.login_view.view().map(AppMessage::LoginMessage))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            AppState::Main => {
                let sidebar = self.sidebar.view(self.wallets.len()).map(AppMessage::SidebarMessage);
                let _selected_wallet = self.wallets.get(self.selected_wallet);

                let main_content = match self.current_page {
                    NavItem::Dashboard => self
                        .dashboard
                        .view(self.is_refreshing)
                        .map(AppMessage::DashboardMessage),
                    NavItem::Wallets => self
                        .wallets_view
                        .view(
                            &self.wallets,
                            self.selected_wallet,
                            self.selected_wallet_revealed_mnemonic(),
                        )
                        .map(AppMessage::WalletsMessage),
                    NavItem::Send => self
                        .send_view
                        .view(
                            &self.wallets,
                            self.selected_wallet,
                            self.is_estimating_fee,
                            self.is_calculating_max,
                            self.is_sending,
                        )
                        .map(AppMessage::SendMessage),
                    NavItem::Receive => self
                        .receive_view
                        .view(&self.wallets, self.selected_wallet)
                        .map(AppMessage::ReceiveMessage),
                    NavItem::History => self
                        .history_view
                        .view(&self.wallets, self.selected_wallet, self.is_refreshing)
                        .map(AppMessage::HistoryMessage),
                    NavItem::Settings => self.settings_view.view().map(AppMessage::SettingsMessage),
                };

                let error_bar = if let Some(error) = &self.error {
                    container(
                        row![
                            text(error.as_str())
                                .size(12)
                                .style(text_color(Colors::TEXT_PRIMARY)),
                            Space::with_width(Length::Fill),
                            iced::widget::button(text(t("Đóng", "Close")).size(12))
                                .on_press(AppMessage::DismissError)
                                .padding(6)
                                .style(secondary_button_style()),
                        ]
                        .align_y(iced::Alignment::Center),
                    )
                    .padding(10)
                    .style(notice_style(NoticeTone::Error))
                } else {
                    container(Space::with_height(0))
                };

                let language_picker = self.language_selector.view(AppMessage::LanguageChanged);

                let header_bar = container(
                    row![
                        text(format!(
                            "{} {}",
                            t("Xin chào,", "Hello,"),
                            self.display_name()
                        ))
                        .size(14)
                        .style(text_color(Colors::TEXT_SECONDARY)),
                        Space::with_width(Length::Fill),
                        text(self.current_page.title())
                            .size(14)
                            .style(text_color(Colors::TEXT_MUTED)),
                        Space::with_width(12),
                        language_picker,
                    ]
                    .align_y(iced::Alignment::Center),
                )
                .padding(12);

                let base_content = container(row![
                    sidebar,
                    column![header_bar, error_bar, main_content,].width(Length::Fill)
                ])
                .width(Length::Fill)
                .height(Length::Fill)
                .style(screen_background_style());

                // Toast notification layer on top - centered horizontally
                if self.toast_manager.has_toasts() {
                    if let Some(toast_view) = self.toast_manager.view() {
                        use iced::widget::stack;
                        
                        // Dùng row với 2 Space ở 2 bên để đẩy toast vào giữa
                        let centered_row = row![
                            Space::with_width(Length::Fill),
                            column![
                                Space::with_height(20),
                                toast_view.map(|_| AppMessage::DismissStatus),
                            ]
                            .spacing(0),
                            Space::with_width(Length::Fill),
                        ]
                        .width(Length::Fill)
                        .spacing(0);
                        
                        let toast_overlay = container(
                            column![
                                centered_row,
                                Space::with_height(Length::Fill),
                            ]
                            .spacing(0)
                            .width(Length::Fill),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill);
                        
                        return stack![
                            base_content,
                            toast_overlay
                        ]
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into();
                    }
                }

                base_content.into()
            }
        }
    }

    pub fn update_dashboard(&mut self) {
        let total: i64 = self.wallets.iter().map(|wallet| wallet.balance()).sum();

        let confirmed: i64 = self
            .wallets
            .iter()
            .map(|wallet| wallet.confirmed_balance())
            .sum();

        let pending = total - confirmed;
        let backup_needed = self
            .wallets
            .iter()
            .filter(|wallet| wallet.has_mnemonic && !wallet.mnemonic_backed_up)
            .count();

        self.dashboard.update_balances(
            total,
            confirmed,
            pending,
            self.wallets.len(),
            backup_needed,
        );
    }

    pub fn refresh_all_wallets(&mut self) -> Task<AppMessage> {
        if self.wallets.is_empty() {
            self.add_info_toast(t("Không có ví để làm mới", "No wallets to refresh").to_string());
            return Task::none();
        }

        if self.is_refreshing {
            return Task::none();
        }

        self.is_refreshing = true;
        self.add_info_toast(t("Đang làm mới dữ liệu ví...", "Refreshing wallet data...").to_string());
        self.error = None;

        let wallets = self.wallets.clone();
        let wallet_vault = self.wallet_vault.clone();
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let mut wallets = wallets;
                    let mut refreshed_wallets = 0usize;
                    let mut refreshed_txs = 0usize;
                    let mut errors = Vec::new();

                    for wallet in &mut wallets {
                        let Some(secrets) = wallet_vault.get(wallet.wallet_id()) else {
                            errors.push(format!("{}: thiếu wallet secret", wallet.name));
                            continue;
                        };

                        match wallet.refresh_history(secrets.as_ref()) {
                            Ok(count) => {
                                refreshed_wallets += 1;
                                refreshed_txs += count;
                            }
                            Err(err) => {
                                errors.push(format!("{}: {}", wallet.name, err));
                            }
                        }
                    }

                    Ok(RefreshWalletsResult {
                        wallets,
                        refreshed_wallets,
                        refreshed_txs,
                        errors,
                    })
                })
                .await
                .unwrap_or_else(|err| Err(format!("Wallet refresh task failed: {err}")))
            },
            AppMessage::RefreshWalletsFinished,
        )
    }

    pub fn save_language_preference(&mut self) {
        let result =
            Storage::new().and_then(|storage| storage.save_language_preference(self.language));
        if let Err(err) = result {
            if self.error.is_none() {
                self.error = Some(format!(
                    "{}: {err}",
                    t(
                        "Không thể lưu cài đặt ngôn ngữ",
                        "Could not save language preference"
                    )
                ));
            }
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
                        self.error = Some(format!(
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
                    self.error = Some(format!(
                        "{}: {err}",
                        t("Không thể lưu trạng thái", "Failed to save app state")
                    ));
                    false
                } else {
                    true
                }
            }
            Err(err) => {
                self.error = Some(format!(
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
        self.error = None;
        self.is_refreshing = false;
        self.is_estimating_fee = false;
        self.is_calculating_max = false;
        self.is_sending = false;
        self.revealed_mnemonic_session_id = 0;
        self.toast_manager = ToastManager::new();
        self.last_copied_address = None;
        self.last_copied_time = None;

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

    pub fn display_name(&self) -> &str {
        self.user_nickname.as_deref().unwrap_or(t("bạn", "friend"))
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

    pub fn selected_wallet_revealed_mnemonic(&self) -> Option<&str> {
        let wallet = self.wallets.get(self.selected_wallet)?;
        if self.wallets_view.revealed_wallet_index() != Some(self.selected_wallet) {
            return None;
        }

        self.wallet_secret(wallet)
            .and_then(|secrets| secrets.mnemonic_phrase())
    }

    pub fn wallet_secret(&self, wallet: &Wallet) -> Option<&WalletSecretsRef> {
        self.wallet_vault.get(wallet.wallet_id())
    }

    pub fn wallet_secret_by_index(&self, wallet_index: usize) -> Option<WalletSecretsRef> {
        let wallet = self.wallets.get(wallet_index)?;
        self.wallet_vault.get(wallet.wallet_id()).cloned()
    }

    pub fn restore_runtime_state(&mut self, state: RuntimeState) {
        self.user_nickname = normalize_nickname(state.profile.nickname.as_deref());
        self.language = state.profile.language;
        set_current_language(self.language);
        self.save_language_preference();
        self.wallets = state.wallets;
        self.wallet_vault = state.wallet_vault;
        self.state = AppState::Main;
        self.selected_wallet = self
            .selected_wallet
            .min(self.wallets.len().saturating_sub(1));
        self.update_dashboard();
        self.login_view.clear_error();
    }

    pub fn current_passphrase(&self) -> Option<&SecretString> {
        self.storage_passphrase.as_ref()
    }

    pub fn add_toast(&mut self, toast: Toast) {
        self.toast_manager.add_toast(toast);
    }

    pub fn add_success_toast(&mut self, message: String) {
        self.toast_manager.add_toast(Toast::success(message));
    }

    pub fn add_error_toast(&mut self, message: String) {
        self.toast_manager.add_toast(Toast::error(message));
    }

    pub fn add_info_toast(&mut self, message: String) {
        self.toast_manager.add_toast(Toast::info(message));
    }

    pub fn add_warning_toast(&mut self, message: String) {
        self.toast_manager.add_toast(Toast::warning(message));
    }

    pub fn track_copy(&mut self, address: String) {
        let now = Local::now();
        self.last_copied_address = Some(address);
        self.last_copied_time = Some(now.format("%H:%M:%S").to_string());
    }

    pub fn passphrase_matches(&self, candidate: &str) -> bool {
        self.current_passphrase()
            .map(|value| value.expose_secret() == candidate)
            .unwrap_or(false)
    }

    pub fn clear_revealed_mnemonic(&mut self) {
        self.revealed_mnemonic_session_id = self.revealed_mnemonic_session_id.wrapping_add(1);
        self.wallets_view.clear_revealed_mnemonic();
    }

    pub fn schedule_revealed_mnemonic_timeout(&mut self) -> Task<AppMessage> {
        self.revealed_mnemonic_session_id = self.revealed_mnemonic_session_id.wrapping_add(1);
        let session_id = self.revealed_mnemonic_session_id;

        Task::perform(
            async move {
                tokio::time::sleep(Duration::from_secs(REVEALED_MNEMONIC_TTL_SECS)).await;
                session_id
            },
            AppMessage::RevealedMnemonicExpired,
        )
    }

    fn handle_refresh_wallets_finished(
        &mut self,
        result: Result<RefreshWalletsResult, String>,
    ) -> Task<AppMessage> {
        self.is_refreshing = false;

        match result {
            Ok(payload) => {
                self.wallets = payload.wallets;
                let save_error = if self.save_state() {
                    None
                } else {
                    self.error.clone()
                };
                self.update_dashboard();
                self.dashboard.set_last_synced_label(Some(
                    Local::now().format("%d/%m/%Y %H:%M:%S").to_string(),
                ));
                self.add_success_toast(format!(
                    "{} {}, {} {}",
                    t("Đã làm mới", "Refreshed"),
                    wallet_count_text(payload.refreshed_wallets),
                    payload.refreshed_txs,
                    t("giao dịch", "transaction(s)")
                ));
                let refresh_error = if payload.errors.is_empty() {
                    None
                } else {
                    Some(format!(
                        "{}: {}",
                        t("Một số ví làm mới lỗi", "Some wallets failed to refresh"),
                        payload.errors.join(" | ")
                    ))
                };
                self.error = match (save_error, refresh_error) {
                    (Some(save_error), Some(refresh_error)) => {
                        Some(format!("{save_error} | {refresh_error}"))
                    }
                    (Some(save_error), None) => Some(save_error),
                    (None, Some(refresh_error)) => Some(refresh_error),
                    (None, None) => None,
                };
            }
            Err(err) => {
                self.error = Some(format!(
                    "{}: {err}",
                    t("Làm mới ví thất bại", "Wallet refresh failed")
                ));
            }
        }

        Task::none()
    }

    fn handle_revealed_mnemonic_expired(&mut self, session_id: u64) -> Task<AppMessage> {
        if session_id != self.revealed_mnemonic_session_id {
            return Task::none();
        }

        if self.wallets_view.revealed_wallet_index().is_none() {
            return Task::none();
        }

        self.clear_revealed_mnemonic();
        self.add_info_toast(format!(
            "{} {} {}",
            t("Mnemonic đã tự động khóa sau", "Mnemonic auto-locked after"),
            REVEALED_MNEMONIC_TTL_SECS,
            t("giây", "seconds")
        ));
        self.error = None;

        Task::none()
    }
}
