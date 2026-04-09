use crate::views::login::LoginMode;
use iced::Task;
use secrecy::{ExposeSecret, SecretString};

use crate::i18n::{set_current_language, t, AppLanguage};
use crate::storage::{AppTheme, Storage};
use crate::utils::{pick_export_backup_path, resolve_user_path};
use crate::views::settings::{SettingsEvent, SettingsMessage};

use super::*;

impl App {
    pub fn handle_settings_message(&mut self, msg: SettingsMessage) -> Task<AppMessage> {
        if let Some(event) = self.settings_view.update(msg) {
            match event {
                SettingsEvent::ChangePassphrase {
                    current,
                    new_passphrase,
                } => {
                    let task = self.handle_change_passphrase(current, new_passphrase);
                    self.settings_view.clear_sensitive_inputs();
                    return task;
                }
                SettingsEvent::ExportWallet => {
                    if let Some(path) = pick_export_backup_path("") {
                        return self
                            .handle_export_wallet_backup(path.to_string_lossy().to_string());
                    }
                }
                SettingsEvent::ClearAllData(passphrase) => {
                    let task = self.handle_clear_all_data(passphrase);
                    self.settings_view.clear_sensitive_inputs();
                    return task;
                }
                SettingsEvent::ThemeChanged(theme) => {
                    return self.handle_change_theme(theme);
                }
                SettingsEvent::ShowOnboardingTour => {
                    self.show_onboarding = true;
                    if let Ok(storage) = Storage::new() {
                        let _ = storage.save_onboarding_completed(false);
                    }
                    self.add_info_toast(t(
                        "Đang mở hướng dẫn...",
                        "Opening onboarding tour...",
                    ).to_string());
                }
                // Accessibility events
                SettingsEvent::FontScaleChanged(scale) => {
                    return self.handle_font_scale_changed(scale);
                }
                SettingsEvent::HighContrastToggled(enabled) => {
                    return self.handle_toggle_high_contrast(enabled);
                }
                // Network events
                SettingsEvent::EsploraEndpointChanged(endpoint) => {
                    if let Ok(storage) = Storage::new() {
                        let _ = storage.save_esplora_endpoint(endpoint);
                    }
                }
                SettingsEvent::TimeoutSecsChanged(secs) => {
                    if let Ok(storage) = Storage::new() {
                        let _ = storage.save_timeout_secs(secs);
                    }
                }
                SettingsEvent::TestConnection => {
                    let endpoint = self.settings_view.esplora_endpoint.clone();
                    let timeout = self.settings_view.timeout_secs;
                    // Reset testing state
                    self.settings_view.testing_connection = true;
                    self.settings_view.connection_test_result = None;
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                crate::wallet::esplora::EsploraClient::test_connection(&endpoint, timeout)
                            })
                            .await
                            .unwrap_or(Err(anyhow::anyhow!("Task failed")))
                        },
                        |result| {
                            match result {
                                Ok(height) => {
                                    AppMessage::SettingsMessage(SettingsMessage::TestConnectionSuccess(format!("✅ Connected! Block height: {}", height)))
                                }
                                Err(e) => {
                                    AppMessage::SettingsMessage(SettingsMessage::TestConnectionFailed(format!("❌ Failed: {}", e)))
                                }
                            }
                        },
                    );
                }
                // Advanced events
                SettingsEvent::DebugLoggingToggled(enabled) => {
                    if let Ok(storage) = Storage::new() {
                        let _ = storage.save_enable_debug(enabled);
                    }
                }
                SettingsEvent::AutoRefreshToggled(enabled) => {
                    if let Ok(storage) = Storage::new() {
                        let _ = storage.save_auto_refresh(enabled);
                    }
                }
                SettingsEvent::ShowSatoshisToggled(enabled) => {
                    self.show_satoshis = enabled;
                    if let Ok(storage) = Storage::new() {
                        let _ = storage.save_show_satoshis(enabled);
                    }
                }
                SettingsEvent::CompactModeToggled(enabled) => {
                    self.compact_mode = enabled;
                    if let Ok(storage) = Storage::new() {
                        let _ = storage.save_compact_mode(enabled);
                    }
                }
                // Reset settings
                SettingsEvent::ResetAllSettings => {
                    if let Ok(storage) = Storage::new() {
                        let _ = storage.reset_preferences();
                        self.add_success_toast(t("Đã đặt lại cài đặt!", "Settings reset!").to_string());
                    }
                }
            }
        }
        Task::none()
    }

    pub fn handle_change_language(&mut self, language: AppLanguage) -> Task<AppMessage> {
        self.language = language;
        set_current_language(language);
        self.save_language_preference();
        if matches!(self.state, super::AppState::Main) {
            self.settings_view.set_success(t(
                "Đã đổi ngôn ngữ ứng dụng",
                "Application language updated",
            ));
        }
        self.save_state();
        Task::none()
    }

    pub fn handle_change_theme(&mut self, theme: AppTheme) -> Task<AppMessage> {
        self.theme = theme;
        if let Ok(storage) = Storage::new() {
            let _ = storage.save_theme(theme);
        }
        self.settings_view.set_success(t(
            "Đã đổi giao diện",
            "Theme updated successfully",
        ));
        Task::none()
    }

    pub fn handle_toggle_high_contrast(&mut self, enabled: bool) -> Task<AppMessage> {
        self.high_contrast = enabled;
        crate::theme::set_high_contrast(enabled);
        if let Ok(storage) = Storage::new() {
            let _ = storage.save_high_contrast(enabled);
        }
        Task::none()
    }

    pub fn handle_font_scale_changed(&mut self, scale: f64) -> Task<AppMessage> {
        self.font_scale = scale.clamp(0.8, 1.5);
        crate::theme::set_font_scale(self.font_scale);
        if let Ok(storage) = Storage::new() {
            let _ = storage.save_font_scale(self.font_scale);
        }
        Task::none()
    }

    pub fn handle_change_passphrase(
        &mut self,
        current: String,
        new_passphrase: String,
    ) -> Task<AppMessage> {
        let current = SecretString::from(current);
        let active_passphrase = match self.current_passphrase() {
            Some(value) => value,
            None => {
                self.settings_view.set_error(t(
                    "Không có session đăng nhập hợp lệ",
                    "No active login session found",
                ));
                return Task::none();
            }
        };
        let new_passphrase = SecretString::from(new_passphrase);

        if current.expose_secret() != active_passphrase.expose_secret() {
            self.settings_view.set_error(t(
                "Passphrase hiện tại không đúng",
                "Current passphrase is incorrect",
            ));
            return Task::none();
        }

        match Storage::new() {
            Ok(storage) => {
                match storage
                    .rotate_passphrase(current.expose_secret(), new_passphrase.expose_secret())
                {
                    Ok(_) => {
                        self.storage_passphrase = Some(new_passphrase);
                        self.settings_view.clear_sensitive_inputs();
                        self.settings_view.set_success(t(
                            "Đổi passphrase thành công",
                            "Passphrase updated successfully",
                        ));
                        self.add_info_toast(
                            t(
                                "Đổi passphrase thành công",
                                "Passphrase updated successfully",
                            )
                            .to_string(),
                        );
                        self.error = None;
                    }
                    Err(err) => {
                        self.settings_view.set_error(format!(
                            "{}: {err}",
                            t("Đổi passphrase thất bại", "Failed to update passphrase")
                        ));
                    }
                }
            }
            Err(err) => {
                self.settings_view.set_error(format!(
                    "{}: {err}",
                    t("Không thể mở storage", "Could not open storage")
                ));
            }
        }
        Task::none()
    }

    pub fn handle_export_wallet_backup(&mut self, raw_path: String) -> Task<AppMessage> {
        let passphrase = match self.current_passphrase() {
            Some(value) => value,
            None => {
                self.settings_view.set_error(t(
                    "Không có session đăng nhập hợp lệ",
                    "No active login session found",
                ));
                return Task::none();
            }
        };

        let export_path = resolve_user_path(&raw_path);
        let state = match self.persisted_state() {
            Ok(state) => state,
            Err(err) => {
                self.settings_view.set_error(format!(
                    "{}: {err}",
                    t(
                        "Không thể gom dữ liệu ví",
                        "Failed to assemble wallet state"
                    )
                ));
                return Task::none();
            }
        };

        match Storage::new() {
            Ok(storage) => {
                match storage.export_encrypted_backup(
                    &state,
                    passphrase.expose_secret(),
                    &export_path,
                ) {
                    Ok(_) => {
                        let message = format!(
                            "{} {}",
                            t(
                                "Đã export backup mã hóa tới",
                                "Exported encrypted backup to"
                            ),
                            export_path.display()
                        );
                        self.settings_view.set_success(message.clone());
                        self.add_info_toast(message);
                        self.error = None;
                    }
                    Err(err) => {
                        self.settings_view.set_error(format!(
                            "{}: {err}",
                            t("Export backup thất bại", "Backup export failed")
                        ));
                    }
                }
            }
            Err(err) => {
                self.settings_view.set_error(format!(
                    "{}: {err}",
                    t("Không thể mở storage", "Could not open storage")
                ));
            }
        }

        Task::none()
    }

    pub fn handle_clear_all_data(&mut self, passphrase: String) -> Task<AppMessage> {
        let passphrase = SecretString::from(passphrase);
        let active_passphrase = match self.current_passphrase() {
            Some(value) => value,
            None => {
                self.settings_view.set_error(t(
                    "Không có session đăng nhập hợp lệ",
                    "No active login session found",
                ));
                return Task::none();
            }
        };

        if passphrase.expose_secret() != active_passphrase.expose_secret() {
            self.settings_view.set_error(t(
                "Passphrase hiện tại không đúng",
                "Current passphrase is incorrect",
            ));
            return Task::none();
        }

        match Storage::new() {
            Ok(storage) => match storage.clear_all_data() {
                Ok(_) => {
                    self.reset_to_login(true);
                    self.login_view.set_mode(LoginMode::NewWallet);
                    self.login_view.set_error(t(
                        "Đã xóa toàn bộ dữ liệu cũ. Hãy tạo passphrase mới.",
                        "All old data has been deleted. Please create a new passphrase.",
                    ));
                }
                Err(err) => {
                    self.settings_view.set_error(format!(
                        "{}: {err}",
                        t("Không thể xóa dữ liệu", "Could not clear data")
                    ));
                }
            },
            Err(err) => {
                self.settings_view.set_error(format!(
                    "{}: {err}",
                    t("Không thể mở storage", "Could not open storage")
                ));
            }
        }
        Task::none()
    }
}
