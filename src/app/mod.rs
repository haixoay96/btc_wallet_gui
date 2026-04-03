mod login;
mod send;
mod settings;
mod wallet;

use iced::{
    clipboard,
    widget::{column, container, row, text, Space},
    Element, Length, Task,
};

use crate::i18n::{set_current_language, t, AppLanguage};
use crate::storage::{PersistedState, Storage, UserProfile};
use crate::utils::wallet_count_text;
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
use crate::wallet::Wallet;

#[derive(Debug, Clone)]
pub enum AppState {
    Login,
    Main,
}

pub struct App {
    pub state: AppState,
    pub storage_passphrase: Option<String>,
    pub language: AppLanguage,
    pub user_nickname: Option<String>,
    pub wallets: Vec<Wallet>,
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
            },
            Task::none(),
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
        }
    }

    pub fn view(&self) -> Element<'_, AppMessage> {
        match self.state {
            AppState::Login => container(self.login_view.view().map(AppMessage::LoginMessage))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            AppState::Main => {
                let sidebar = self.sidebar.view().map(AppMessage::SidebarMessage);
                let _selected_wallet = self.wallets.get(self.selected_wallet);

                let main_content = match self.current_page {
                    NavItem::Dashboard => self
                        .dashboard
                        .view(self.is_refreshing)
                        .map(AppMessage::DashboardMessage),
                    NavItem::Wallets => self
                        .wallets_view
                        .view(&self.wallets, self.selected_wallet)
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

                let status_bar = if let Some(status) = &self.status {
                    container(
                        text(status.as_str())
                            .size(12)
                            .style(crate::theme::text_color(crate::theme::Colors::SUCCESS)),
                    )
                    .padding(8)
                } else {
                    container(Space::with_height(0))
                };

                let error_bar = if let Some(error) = &self.error {
                    container(
                        text(error.as_str())
                            .size(12)
                            .style(crate::theme::text_color(crate::theme::Colors::ERROR)),
                    )
                    .padding(8)
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
                        .style(crate::theme::text_color(
                            crate::theme::Colors::TEXT_SECONDARY,
                        )),
                        Space::with_width(Length::Fill),
                        language_picker,
                    ]
                    .align_y(iced::Alignment::Center),
                )
                .padding(8);

                row![
                    sidebar,
                    column![header_bar, status_bar, error_bar, main_content,].width(Length::Fill)
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
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

        self.dashboard
            .update_balances(total, confirmed, self.wallets.len());
    }

    pub fn refresh_all_wallets(&mut self) -> Task<AppMessage> {
        if self.wallets.is_empty() {
            self.status = Some(t("Không có ví để làm mới", "No wallets to refresh").to_string());
            return Task::none();
        }

        if self.is_refreshing {
            return Task::none();
        }

        self.is_refreshing = true;
        self.status =
            Some(t("Đang làm mới dữ liệu ví...", "Refreshing wallet data...").to_string());
        self.error = None;

        let wallets = self.wallets.clone();
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let mut wallets = wallets;
                    let mut refreshed_wallets = 0usize;
                    let mut refreshed_txs = 0usize;
                    let mut errors = Vec::new();

                    for wallet in &mut wallets {
                        match wallet.refresh_history() {
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

    pub fn save_state(&mut self) {
        let passphrase = match &self.storage_passphrase {
            Some(value) => value.clone(),
            None => return,
        };

        match Storage::new() {
            Ok(storage) => {
                let state = self.persisted_state();
                if let Err(err) = storage.save_state(&state, &passphrase) {
                    self.error = Some(format!(
                        "{}: {err}",
                        t("Không thể lưu trạng thái", "Failed to save app state")
                    ));
                }
            }
            Err(err) => {
                self.error = Some(format!(
                    "{}: {err}",
                    t("Không thể khởi tạo storage", "Failed to initialize storage")
                ));
            }
        }
    }

    pub fn reset_to_login(&mut self, allow_create_passphrase: bool) {
        self.state = AppState::Login;
        self.storage_passphrase = None;
        self.user_nickname = None;
        self.wallets.clear();
        self.selected_wallet = 0;
        self.current_page = NavItem::Dashboard;
        self.status = None;
        self.error = None;
        self.is_refreshing = false;
        self.is_estimating_fee = false;
        self.is_calculating_max = false;
        self.is_sending = false;

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

    pub fn persisted_state(&self) -> PersistedState {
        PersistedState {
            profile: UserProfile {
                nickname: self.user_nickname.clone(),
                language: self.language,
            },
            wallets: self.wallets.clone(),
        }
    }

    fn handle_refresh_wallets_finished(
        &mut self,
        result: Result<RefreshWalletsResult, String>,
    ) -> Task<AppMessage> {
        self.is_refreshing = false;

        match result {
            Ok(payload) => {
                self.wallets = payload.wallets;
                self.save_state();
                self.update_dashboard();
                self.status = Some(format!(
                    "{} {}, {} {}",
                    t("Đã làm mới", "Refreshed"),
                    wallet_count_text(payload.refreshed_wallets),
                    payload.refreshed_txs,
                    t("giao dịch", "transaction(s)")
                ));
                self.error = if payload.errors.is_empty() {
                    None
                } else {
                    Some(format!(
                        "{}: {}",
                        t("Một số ví làm mới lỗi", "Some wallets failed to refresh"),
                        payload.errors.join(" | ")
                    ))
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
}
