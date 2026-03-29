use iced::Task;

use crate::i18n::{set_current_language, t, AppLanguage};
use crate::storage::{PersistedState, Storage, UserProfile};
use crate::views::settings::{SettingsEvent, SettingsMessage};

use super::*;

impl App {
    pub fn handle_settings_message(&mut self, msg: SettingsMessage) -> Task<AppMessage> {
        if let Some(event) = self.settings_view.update(msg) {
            match event {
                SettingsEvent::ChangeLanguage(language) => {
                    return self.handle_change_language(language)
                }
                SettingsEvent::BrowseExportPath => {
                    if let Some(path) = super::pick_export_backup_path("") {
                        self.settings_view
                            .set_export_path(path.to_string_lossy().to_string());
                    }
                }
                SettingsEvent::ChangePassphrase {
                    current,
                    new_passphrase,
                } => {
                    return self.handle_change_passphrase(current, new_passphrase)
                }
                SettingsEvent::ExportWallet(path) => {
                    return self.handle_export_wallet_backup(path)
                }
                SettingsEvent::ClearAllData(passphrase) => {
                    return self.handle_clear_all_data(passphrase)
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

    pub fn handle_change_passphrase(
        &mut self,
        current: String,
        new_passphrase: String,
    ) -> Task<AppMessage> {
        let active_passphrase = match &self.storage_passphrase {
            Some(value) => value.clone(),
            None => {
                self.settings_view.set_error(t(
                    "Không có session đăng nhập hợp lệ",
                    "No active login session found",
                ));
                return Task::none();
            }
        };

        if current != active_passphrase {
            self.settings_view.set_error(t(
                "Passphrase hiện tại không đúng",
                "Current passphrase is incorrect",
            ));
            return Task::none();
        }

        match Storage::new() {
            Ok(storage) => match storage.rotate_passphrase(&current, &new_passphrase) {
                Ok(_) => {
                    self.storage_passphrase = Some(new_passphrase);
                    self.settings_view.clear_sensitive_inputs();
                    self.settings_view.set_success(t(
                        "Đổi passphrase thành công",
                        "Passphrase updated successfully",
                    ));
                    self.status = Some(
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

    pub fn handle_export_wallet_backup(&mut self, raw_path: String) -> Task<AppMessage> {
        let passphrase = match &self.storage_passphrase {
            Some(value) => value.clone(),
            None => {
                self.settings_view.set_error(t(
                    "Không có session đăng nhập hợp lệ",
                    "No active login session found",
                ));
                return Task::none();
            }
        };

        let export_path = resolve_user_path(&raw_path);
        let state = PersistedState {
            profile: UserProfile {
                nickname: self.user_nickname.clone(),
                language: self.language,
            },
            wallets: self.wallets.clone(),
        };

        match Storage::new() {
            Ok(storage) => {
                match storage.export_encrypted_backup(&state, &passphrase, &export_path) {
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
                        self.status = Some(message);
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
        let active_passphrase = match &self.storage_passphrase {
            Some(value) => value.clone(),
            None => {
                self.settings_view.set_error(t(
                    "Không có session đăng nhập hợp lệ",
                    "No active login session found",
                ));
                return Task::none();
            }
        };

        if passphrase != active_passphrase {
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