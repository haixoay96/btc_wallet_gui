mod login;
mod send;
mod settings;
mod wallet;

use std::time::Duration;

use chrono::Local;
use iced::{
    clipboard, keyboard,
    widget::{column, container, row, text, Space},
    Element, Length, Subscription, Task, Theme,
};
use secrecy::{ExposeSecret, SecretString};

use crate::components::{Toast, ToastManager, shortcuts_help_popup, modal, error_card, HelpDismissals};
use crate::error::AppError;
use crate::i18n::{set_current_language, t, AppLanguage};
use crate::storage::{AppTheme, PersistedState, RuntimeState, Storage, UserProfile, AddressBook};
use crate::theme::{
    notice_style, screen_background_style, secondary_button_style, text_color, Colors, NoticeTone,
    get_theme_colors,
};
use crate::utils::{normalize_nickname, wallet_count_text, export::{export_history_csv, export_history_pdf}};
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
use std::path::PathBuf;

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
    pub theme: AppTheme,
    pub high_contrast: bool,
    pub font_scale: f64,
    pub show_satoshis: bool,
    pub compact_mode: bool,
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
    
    // Help system
    pub help_dismissals: HelpDismissals,

    // Onboarding
    pub show_onboarding: bool,
    pub onboarding_view: OnboardingView,
    pub onboarding_delay_timer: bool,

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
    ThemeChanged(AppTheme),
    HighContrastToggled(bool),
    FontScaleChanged(f64),
    RefreshWalletsFinished(Result<RefreshWalletsResult, String>),
    EstimateSendFeeFinished(Result<u64, String>),
    MaxAmountFinished(Result<(u64, u64), String>),
    SendTransactionFinished(Result<SendExecutionResult, String>),
    RevealedMnemonicExpired(u64),
    ToastCleanup,
    ToggleShortcutsHelp,
    PickExportCsvPath,
    PickExportPdfPath,
    ExportFinished(Result<(), String>),
    GlobalEscKey,
    DismissStatus,
    DismissError,
    KeyboardCopy,
    KeyboardPaste,
    KeyboardSubmitForm,
    KeyboardSaveState,
    KeyboardFocusSearch,
    ResetHelpDismissals,
    AutoRefreshConfirmations,
    OnboardingMessage(OnboardingMessage),
    StartOnboardingDelay,
    ShowOnboarding,
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
        let (initial_language, has_existing_state, initial_theme, initial_high_contrast, initial_font_scale, onboarding_completed, initial_show_satoshis, initial_compact_mode) = match Storage::new() {
            Ok(storage) => {
                let language = storage
                    .load_language_preference()
                    .unwrap_or(fallback_language);
                let theme = storage.load_theme().unwrap_or(AppTheme::Dark);
                let high_contrast = storage.load_high_contrast().unwrap_or(false);
                let font_scale = storage.load_font_scale().unwrap_or(1.0);
                let onboarding = storage.load_onboarding_completed().unwrap_or(false);
                let show_satoshis = storage.load_show_satoshis().unwrap_or(false);
                let compact_mode = storage.load_compact_mode().unwrap_or(false);
                // Set global states
                crate::theme::set_high_contrast(high_contrast);
                crate::theme::set_font_scale(font_scale);
                (language, storage.has_existing_state(), theme, high_contrast, font_scale, onboarding, show_satoshis, compact_mode)
            }
            Err(_) => (fallback_language, false, AppTheme::Dark, false, 1.0, false, false, false),
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
                show_satoshis: initial_show_satoshis,
                compact_mode: initial_compact_mode,
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
                help_dismissals: HelpDismissals::new(),
                show_onboarding: false, // Start hidden, show after delay
                onboarding_view: OnboardingView::new(),
                onboarding_delay_timer: true, // Always check onboarding on fresh start
                address_book: AddressBook::load().unwrap_or_default(),
            },
            // Start toast cleanup task + onboarding delay if needed
            {
                let mut tasks = vec![Task::perform(
                    async move {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    },
                    |_| AppMessage::ToastCleanup,
                )];
                // Always show onboarding delay on fresh app start
                tasks.push(Task::perform(
                    async move {
                        tokio::time::sleep(Duration::from_millis(1200)).await;
                    },
                    |_| AppMessage::ShowOnboarding,
                ));
                Task::batch(tasks)
            },
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
                        if page == NavItem::Settings {
                            // Only sync once when navigating TO Settings
                            // Don't sync font_scale/high_contrast here to allow real-time slider updates
                            self.settings_view.esplora_endpoint = if let Ok(storage) = Storage::new() {
                                storage.load_esplora_endpoint().unwrap_or_default()
                            } else {
                                "https://blockstream.info/api".to_string()
                            };
                            self.settings_view.timeout_secs = if let Ok(storage) = Storage::new() {
                                storage.load_timeout_secs().unwrap_or(15)
                            } else {
                                15
                            };
                            self.settings_view.debug_logging = if let Ok(storage) = Storage::new() {
                                storage.load_enable_debug().unwrap_or(false)
                            } else {
                                false
                            };
                            self.settings_view.auto_refresh = if let Ok(storage) = Storage::new() {
                                storage.load_auto_refresh().unwrap_or(false)
                            } else {
                                false
                            };
                            self.settings_view.show_satoshis = self.show_satoshis;
                            self.settings_view.compact_mode = if let Ok(storage) = Storage::new() {
                                storage.load_compact_mode().unwrap_or(false)
                            } else {
                                false
                            };
                            self.settings_view.load_data_folder_info();
                        }
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
                        self.error = Some(AppError::unknown(&format!("{}: {}", t("Không thể mở trình duyệt", "Cannot open browser"), e)));
                    }
                    Task::none()
                }
                _ => {
                    if let Some(event) = self.history_view.update(msg) {
                        match event {
                            HistoryEvent::Refresh => return self.refresh_all_wallets(),
                            HistoryEvent::ExportCsv => {
                                if let Some(wallet) = self.wallets.get(self.selected_wallet) {
                                    let wallet_clone = wallet.clone();
                                    return Task::perform(
                                        async move {
                                            tokio::task::spawn_blocking(move || {
                                                rfd::FileDialog::new()
                                                    .set_title(t("Lưu lịch sử CSV", "Save history as CSV"))
                                                    .add_filter(t("File CSV", "CSV file"), &["csv"])
                                                    .set_file_name("history_export.csv")
                                                    .save_file()
                                            })
                                            .await
                                            .unwrap_or(None)
                                            .and_then(|path| {
                                                let mut wtr = String::new();
                                                wtr.push_str("\u{FEFF}");
                                                wtr.push_str("Date,Time,Type,Amount BTC,Amount Sat,Confirmations,TxID\n");
                                                for tx in &wallet_clone.history {
                                                    let date_str = if let Some(ts) = tx.block_time {
                                                        let dt = chrono::DateTime::from_timestamp(ts as i64, 0).unwrap_or_default();
                                                        format!("{},{}", dt.format("%Y-%m-%d"), dt.format("%H:%M:%S"))
                                                    } else {
                                                        "Pending,Pending".to_string()
                                                    };
                                                    let type_str = match tx.direction {
                                                        crate::wallet::TxDirection::Incoming => "IN",
                                                        crate::wallet::TxDirection::Outgoing => "OUT",
                                                        crate::wallet::TxDirection::SelfTransfer => "SELF",
                                                    };
                                                    let amount_btc = tx.amount_sat as f64 / 100_000_000.0;
                                                    wtr.push_str(&format!("{},{},{:.8},{},{},{}\n",
                                                        date_str, type_str, amount_btc,
                                                        tx.amount_sat, tx.confirmations, tx.txid));
                                                }
                                                std::fs::write(&path, wtr).ok()
                                            })
                                        },
                                        |result| {
                                            if result.is_some() {
                                                AppMessage::ExportFinished(Ok(()))
                                            } else {
                                                AppMessage::DismissStatus
                                            }
                                        },
                                    );
                                }
                                return Task::none();
                            }
                            HistoryEvent::ExportPdf => {
                                if let Some(wallet) = self.wallets.get(self.selected_wallet) {
                                    let wallet_clone = wallet.clone();
                                    return Task::perform(
                                        async move {
                                            tokio::task::spawn_blocking(move || {
                                                rfd::FileDialog::new()
                                                    .set_title(t("Lưu lịch sử PDF", "Save history as PDF"))
                                                    .add_filter(t("File PDF", "PDF file"), &["pdf"])
                                                    .set_file_name("history_export.pdf")
                                                    .save_file()
                                            })
                                            .await
                                            .unwrap_or(None)
                                            .and_then(|path| {
                                                use printpdf::{BuiltinFont, Mm, PdfDocument};
                                                let (doc, page, layer) = PdfDocument::new(
                                                    "Transaction History",
                                                    Mm(210.0),
                                                    Mm(297.0),
                                                    "History Layer",
                                                );
                                                let current_layer = doc.get_page(page).get_layer(layer);
                                                let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica).ok()?;
                                                let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).ok()?;
                                                current_layer.use_text(
                                                    format!("Bitcoin Wallet - {}", wallet_clone.name),
                                                    18.0, Mm(15.0), Mm(280.0), &font_bold,
                                                );
                                                let mut y_pos = 260.0;
                                                for tx in &wallet_clone.history {
                                                    if y_pos < 15.0 {
                                                        break;
                                                    }
                                                    let date_str = if let Some(ts) = tx.block_time {
                                                        let dt = chrono::DateTime::from_timestamp(ts as i64, 0).unwrap_or_default();
                                                        dt.format("%d/%m/%Y").to_string()
                                                    } else {
                                                        "Pending".to_string()
                                                    };
                                                    let amount_btc = tx.amount_sat as f64 / 100_000_000.0;
                                                    current_layer.use_text(date_str, 8.0, Mm(15.0), Mm(y_pos), &font_regular);
                                                    current_layer.use_text(format!("{:.8}", amount_btc), 8.0, Mm(65.0), Mm(y_pos), &font_regular);
                                                    current_layer.use_text(&tx.txid[..16.min(tx.txid.len())], 8.0, Mm(115.0), Mm(y_pos), &font_regular);
                                                    y_pos -= 10.0;
                                                }
                                                if let Ok(file) = std::fs::File::create(&path) {
                                                    let mut writer = std::io::BufWriter::new(file);
                                                    doc.save(&mut writer).ok()
                                                } else {
                                                    None
                                                }
                                            })
                                        },
                                        |result| {
                                            if result.is_some() {
                                                AppMessage::ExportFinished(Ok(()))
                                            } else {
                                                AppMessage::DismissStatus
                                            }
                                        },
                                    );
                                }
                                return Task::none();
                            }
                        }
                    }
                    Task::none()
                }
            },

            AppMessage::SettingsMessage(msg) => self.handle_settings_message(msg),

            AppMessage::LanguageChanged(language) => self.handle_change_language(language),
            AppMessage::ThemeChanged(theme) => self.handle_change_theme(theme),
            AppMessage::HighContrastToggled(enabled) => self.handle_toggle_high_contrast(enabled),
            AppMessage::FontScaleChanged(scale) => self.handle_font_scale_changed(scale),
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
            AppMessage::ToggleShortcutsHelp => {
                self.show_shortcuts_help = !self.show_shortcuts_help;
                Task::none()
            }
            AppMessage::GlobalEscKey => {
                // Close popups in priority order
                if self.show_shortcuts_help {
                    self.show_shortcuts_help = false;
                } else if self.send_view.show_confirm {
                    self.send_view.update(SendMessage::CancelSend);
                } else if self.receive_view.show_qr {
                    self.receive_view.update(ReceiveMessage::CloseQrPopup);
                } else if self.wallets_view.confirm_delete_index.is_some() {
                    self.wallets_view.update(WalletsMessage::CancelDelete);
                } else if self.wallets_view.notice_wallet_index.is_some() {
                    self.wallets_view.update(WalletsMessage::DismissWalletNotice);
                } else if self.settings_view.show_clear_data_confirm {
                    self.settings_view.update(SettingsMessage::ToggleClearDataConfirm);
                }
                Task::none()
            }
            AppMessage::ExportFinished(result) => {
                match result {
                    Ok(_) => {
                        self.add_success_toast(t("Xuất file thành công!", "Export successful!").to_string());
                    }
                    Err(e) => {
                        self.error = Some(AppError::storage("export", &format!("{}: {}", t("Lỗi export", "Export error"), e)));
                    }
                }
                Task::none()
            }
            AppMessage::PickExportCsvPath | AppMessage::PickExportPdfPath => {
                // These are handled in the async task
                Task::none()
            }
            AppMessage::DismissStatus => {
                Task::none()
            }
            AppMessage::DismissError => {
                self.error = None;
                Task::none()
            }
            
            // Keyboard shortcut handlers
            AppMessage::KeyboardCopy => {
                // Copy from current context based on current page
                match self.current_page {
                    NavItem::Receive => {
                        // Copy current receive address
                        if let Some(addr) = self.receive_view.get_current_address() {
                            self.last_copied_address = Some(addr.clone());
                            self.last_copied_time = Some(chrono::Local::now().format("%H:%M:%S").to_string());
                            clipboard::write(addr)
                        } else {
                            Task::none()
                        }
                    }
                    _ => Task::none(),
                }
            }
            
            AppMessage::KeyboardPaste => {
                // Only paste on Send screen - set focus flag to paste when text input is ready
                if self.current_page == NavItem::Send {
                    self.focus_paste_send = true;
                }
                Task::none()
            }
            
            AppMessage::KeyboardSubmitForm => {
                // Submit form based on current context
                if matches!(self.state, AppState::Main) && self.current_page == NavItem::Send {
                    // Trigger send confirmation if form is valid
                    if self.send_view.can_submit() {
                        return self.handle_send_message(SendMessage::ConfirmSend);
                    }
                }
                Task::none()
            }
            
            AppMessage::KeyboardSaveState => {
                // Manual save trigger
                if matches!(self.state, AppState::Main) {
                    self.save_state();
                    self.add_success_toast(t("Đã lưu trạng thái!", "State saved!").to_string());
                }
                Task::none()
            }
            
            AppMessage::KeyboardFocusSearch => {
                // Focus search box in history
                if self.current_page == NavItem::History {
                    self.focus_search_history = true;
                    // Reset flag after focus (view will handle this)
                    return Task::perform(
                        async move {
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        },
                        |_| AppMessage::DismissStatus,
                    );
                }
                Task::none()
            }
            
            AppMessage::ResetHelpDismissals => {
                self.help_dismissals.clear_all();
                self.add_success_toast(t("Đã khôi phục tất cả gợi ý", "Restored all help hints").to_string());
                Task::none()
            }
            
            AppMessage::AutoRefreshConfirmations => {
                // Only auto-refresh if auto_refresh is enabled AND we have pending transactions
                if let Ok(storage) = Storage::new() {
                    if !storage.load_auto_refresh().unwrap_or(false) {
                        return Task::none();
                    }
                }
                let has_pending = self.wallets.iter().any(|w| w.history.iter().any(|tx| !tx.confirmed || tx.confirmations < 6));
                if has_pending {
                    return self.refresh_all_wallets();
                }
                Task::none()
            }
            AppMessage::OnboardingMessage(msg) => {
                use crate::views::onboarding::OnboardingEvent;
                if let Some(event) = self.onboarding_view.update(msg) {
                    match event {
                        OnboardingEvent::Finished | OnboardingEvent::Skipped => {
                            self.show_onboarding = false;
                            if let Ok(storage) = Storage::new() {
                                let _ = storage.save_onboarding_completed(true);
                            }
                        }
                    }
                }
                Task::none()
            }
            AppMessage::ShowOnboarding => {
                self.show_onboarding = true;
                Task::none()
            }
            AppMessage::StartOnboardingDelay => Task::none(),
            AppMessage::SettingsMessage(SettingsMessage::ResetAllSettings) => {
                // Reset all settings to default values
                self.theme = AppTheme::Dark;
                self.font_scale = 1.0;
                self.high_contrast = false;
                self.show_satoshis = false;
                if let Ok(storage) = Storage::new() {
                    let _ = storage.reset_preferences();
                }
                self.settings_view = crate::views::settings::SettingsView::new();
                self.add_success_toast(t("Đã đặt lại cài đặt!", "Settings reset!").to_string());
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
                let sidebar = self.sidebar.view(self.wallets.len(), self.compact_mode).map(AppMessage::SidebarMessage);
                let _selected_wallet = self.wallets.get(self.selected_wallet);

                let main_content = match self.current_page {
                    NavItem::Dashboard => self
                        .dashboard
                        .view(self.is_refreshing, self.show_satoshis, self.compact_mode)
                        .map(AppMessage::DashboardMessage),
                    NavItem::Wallets => self
                        .wallets_view
                        .view(
                            &self.wallets,
                            self.selected_wallet,
                            self.selected_wallet_revealed_mnemonic(),
                            self.compact_mode,
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
                            &self.address_book,
                            self.compact_mode,
                        )
                        .map(AppMessage::SendMessage),
                    NavItem::Receive => self
                        .receive_view
                        .view(&self.wallets, self.selected_wallet, self.compact_mode)
                        .map(AppMessage::ReceiveMessage),
                    NavItem::History => self
                        .history_view
                        .view(&self.wallets, self.selected_wallet, self.is_refreshing, self.compact_mode)
                        .map(AppMessage::HistoryMessage),
                    NavItem::Settings => {
                        self.settings_view.view(
                            self.theme,
                            self.font_scale,
                            self.high_contrast,
                            self.compact_mode
                        ).map(AppMessage::SettingsMessage)
                    }
                };

                let error_bar = if let Some(app_error) = &self.error {
                    let mut detail = app_error.user_message();
                    if let Some(ctx) = app_error.context() {
                        detail.push_str(&format!("\n📋 {}", ctx));
                    }
                    container(
                        error_card(
                            app_error.title(),
                            format!("{}\n\n💡 {}", detail, app_error.suggestion()),
                            if app_error.is_retryable() {
                                Some(AppMessage::DismissError)
                            } else {
                                None
                            },
                        ),
                    )
                    .padding(10)
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

                // Shortcuts help popup overlay - Dùng component Modal mới
                if self.show_shortcuts_help {
                    return modal(
                        base_content.into(),
                        t("Phím tắt", "Keyboard Shortcuts"),
                        shortcuts_help_popup().map(|_| AppMessage::ToggleShortcutsHelp),
                        AppMessage::ToggleShortcutsHelp,
                        self.compact_mode,
                    );
                }

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

                let final_content: Element<'_, AppMessage> = if self.show_onboarding {
                    // Stack onboarding overlay on top of main content
                    iced::widget::stack![
                        base_content,
                        // Semi-transparent overlay background
                        container(Space::with_width(Length::Fill))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .style(|theme: &iced::Theme| {
                                let colors = get_theme_colors(theme);
                                iced::widget::container::Style {
                                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                                        colors.bg_primary.r,
                                        colors.bg_primary.g,
                                        colors.bg_primary.b,
                                        0.75,
                                    ))),
                                    ..Default::default()
                                }
                            }),
                        // Onboarding card
                        self.onboarding_view.view().map(AppMessage::OnboardingMessage)
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else {
                    base_content.into()
                };
                final_content.into()
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
                let path = match Storage::new() {
                    Ok(s) => s.file_path().display().to_string(),
                    Err(_) => "unknown".to_string(),
                };
                self.error = Some(AppError::storage_with_path(
                    "language_preference",
                    &path,
                    &format!("{}: {err}", t("Không thể lưu cài đặt ngôn ngữ", "Could not save language preference")),
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
                        self.error = Some(AppError::storage(
                            "assemble_state",
                            &format!("{}: {err}", t("Không thể gom dữ liệu ví", "Failed to assemble wallet state")),
                        ));
                        return false;
                    }
                };
                if let Err(err) = storage.save_state(&state, passphrase.expose_secret()) {
                    self.error = Some(AppError::storage_with_path(
                        "save_state",
                        &storage.file_path().display().to_string(),
                        &format!("{}: {err}", t("Không thể lưu trạng thái", "Failed to save app state")),
                    ));
                    false
                } else {
                    // Also save wallet selection
                    let _ = storage.save_wallet_selection(
                        self.selected_wallet,
                        &self.current_page.title(),
                    );
                    true
                }
            }
            Err(err) => {
                self.error = Some(AppError::storage_with_path(
                    "init_storage",
                    &format!("{}", err),
                    &format!("{}: {err}", t("Không thể khởi tạo storage", "Failed to initialize storage")),
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
                let refresh_error: Option<AppError> = if payload.errors.is_empty() {
                    None
                } else {
                    Some(AppError::api_with_status(
                        "refresh_errors",
                        500,
                        &format!("{}: {}", t("Một số ví làm mới lỗi", "Some wallets failed to refresh"), payload.errors.join(" | ")),
                    ))
                };
                self.error = match (save_error, refresh_error) {
                    (Some(save_error), Some(refresh_error)) => {
                        Some(AppError::unknown(&format!("{} | {}", save_error.user_message(), refresh_error.user_message())))
                    }
                    (Some(save_error), None) => Some(save_error),
                    (None, Some(refresh_error)) => Some(refresh_error),
                    (None, None) => None,
                };
            }
            Err(err) => {
                self.error = Some(AppError::api_with_status(
                    "wallet_refresh",
                    500,
                    &format!("{}: {err}", t("Làm mới ví thất bại", "Wallet refresh failed")),
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

    pub fn subscription(&self) -> Subscription<AppMessage> {
        let keyboard_sub = keyboard::on_key_press(|key_code, modifiers| {
            // Priority 1: Help shortcuts (always available)
            if modifiers.control() && key_code == keyboard::Key::Character("/".into()) {
                return Some(AppMessage::ToggleShortcutsHelp);
            }
            if key_code == keyboard::Key::Named(keyboard::key::Named::F1) {
                return Some(AppMessage::ToggleShortcutsHelp);
            }

            // Priority 2: Esc to close popups (always available)
            if key_code == keyboard::Key::Named(keyboard::key::Named::Escape) {
                return Some(AppMessage::GlobalEscKey);
            }

            // Priority 3: Form and action shortcuts (when not in modal)
            if modifiers.control() || modifiers.command() {
                match key_code {
                    // Navigation shortcuts (Ctrl+1-6)
                    keyboard::Key::Character(c) if c == "1" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(NavItem::Dashboard)));
                    }
                    keyboard::Key::Character(c) if c == "2" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(NavItem::Wallets)));
                    }
                    keyboard::Key::Character(c) if c == "3" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(NavItem::Send)));
                    }
                    keyboard::Key::Character(c) if c == "4" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(NavItem::Receive)));
                    }
                    keyboard::Key::Character(c) if c == "5" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(NavItem::History)));
                    }
                    keyboard::Key::Character(c) if c == "6" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(NavItem::Settings)));
                    }
                    // Copy shortcut (Ctrl+C)
                    keyboard::Key::Character(c) if c == "c" || c == "C" => {
                        return Some(AppMessage::KeyboardCopy);
                    }
                    // Paste shortcut (Ctrl+V)
                    keyboard::Key::Character(c) if c == "v" || c == "V" => {
                        return Some(AppMessage::KeyboardPaste);
                    }
                    // Submit form (Ctrl+Enter)
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        return Some(AppMessage::KeyboardSubmitForm);
                    }
                    // Save state (Ctrl+S)
                    keyboard::Key::Character(c) if c == "s" || c == "S" => {
                        return Some(AppMessage::KeyboardSaveState);
                    }
                    // Focus search (Ctrl+F)
                    keyboard::Key::Character(c) if c == "f" || c == "F" => {
                        return Some(AppMessage::KeyboardFocusSearch);
                    }
                    _ => {}
                }
            }

            None
        });

        // Auto-refresh confirmations - interval based on setting
        let refresh_interval_secs = if let Ok(storage) = crate::storage::Storage::new() {
            if storage.load_auto_refresh().unwrap_or(false) {
                120 // 2 minutes
            } else {
                0 // Disabled
            }
        } else {
            120
        };
        
        let refresh_sub = if refresh_interval_secs > 0 {
            iced::time::every(Duration::from_secs(refresh_interval_secs))
                .map(|_| AppMessage::AutoRefreshConfirmations)
        } else {
            Subscription::none()
        };

        Subscription::batch([keyboard_sub, refresh_sub])
    }

    pub fn current_theme(&self) -> Theme {
        match self.theme {
            crate::storage::AppTheme::Dark => Theme::Dark,
            crate::storage::AppTheme::Light => Theme::Light,
            crate::storage::AppTheme::System => Theme::Dark, // Fallback
        }
    }
}
