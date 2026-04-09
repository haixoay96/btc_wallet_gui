use std::time::Duration;

use iced::Task;

use crate::app::structure::*;
use crate::i18n::t;
use crate::wallet::{Wallet, WalletSecretsRef};
use crate::utils::wallet_count_text;
use crate::error::AppError;
use crate::components::Toast;

impl App {
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

    pub fn add_success_toast(&mut self, message: String) {
        self.toast_manager.add_toast(Toast::success(message));
    }

    pub fn add_info_toast(&mut self, message: String) {
        self.toast_manager.add_toast(Toast::info(message));
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

