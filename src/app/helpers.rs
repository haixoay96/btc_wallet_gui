use chrono::Local;
use iced::{clipboard, Task};
use secrecy::ExposeSecret;

use crate::app::structure::{App, AppMessage, RefreshWalletsResult};
use crate::core::wallet::{Wallet, WalletSecretsRef};
use crate::error::AppError;
use crate::i18n::t;
use crate::ui::components::{Toast, ToastManager};
use crate::ui::views::sidebar::NavItem;
use crate::ui::views::wallets::WalletsView;

impl App {
    pub fn display_name(&self) -> &str {
        self.user_nickname.as_deref().unwrap_or(t("bạn", "friend"))
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
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                session_id
            },
            AppMessage::RevealedMnemonicExpired,
        )
    }

    pub fn handle_refresh_wallets_finished(
        &mut self,
        result: Result<RefreshWalletsResult, String>,
    ) -> Task<AppMessage> {
        self.is_refreshing = false;

        match result {
            Ok(payload) => {
                self.wallets = payload.wallets;
                let save_error: Option<AppError> = if self.save_state() {
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
                    crate::utils::wallet_count_text(payload.refreshed_wallets),
                    payload.refreshed_txs,
                    t("giao dịch", "transaction(s)")
                ));
                let refresh_error: Option<AppError> = if payload.errors.is_empty() {
                    None
                } else {
                    Some(AppError::api_with_status(
                        "refresh_errors",
                        500,
                        &format!(
                            "{}: {}",
                            t("Một số ví làm mới lỗi", "Some wallets failed to refresh"),
                            payload.errors.join(" | ")
                        ),
                    ))
                };
                self.error = match (save_error, refresh_error) {
                    (Some(save_error), Some(refresh_error)) => Some(AppError::unknown(&format!(
                        "{} | {}",
                        save_error.user_message(),
                        refresh_error.user_message()
                    ))),
                    (Some(save_error), None) => Some(save_error),
                    (None, Some(refresh_error)) => Some(refresh_error),
                    (None, None) => None,
                };
            }
            Err(err) => {
                self.error = Some(AppError::api_with_status(
                    "wallet_refresh",
                    500,
                    &format!(
                        "{}: {err}",
                        t("Làm mới ví thất bại", "Wallet refresh failed")
                    ),
                ));
            }
        }

        Task::none()
    }

    pub fn handle_revealed_mnemonic_expired(&mut self, session_id: u64) -> Task<AppMessage> {
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
            60,
            t("giây", "seconds")
        ));
        self.error = None;

        Task::none()
    }

    pub fn current_theme(&self) -> iced::Theme {
        match self.theme {
            crate::infra::storage::AppTheme::Dark => iced::Theme::Dark,
            crate::infra::storage::AppTheme::Light => iced::Theme::Light,
            crate::infra::storage::AppTheme::System => iced::Theme::Dark,
        }
    }
}
