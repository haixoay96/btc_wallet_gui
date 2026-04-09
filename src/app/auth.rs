use crate::error::AppError;
use iced::Task;
use secrecy::{ExposeSecret, SecretString};

use crate::i18n::{set_current_language, t};
use crate::infra::storage::Storage;
use crate::ui::views::login::{LoginEvent, LoginMessage};
use crate::utils::{
    normalize_nickname, pick_import_backup_path, resolve_user_path, wallet_count_text,
};

use super::*;

impl App {
    pub fn handle_login(
        &mut self,
        passphrase: String,
        nickname: Option<String>,
        creating_new: bool,
    ) -> Task<AppMessage> {
        let passphrase = SecretString::from(passphrase);
        self.error = None;

        match Storage::new() {
            Ok(storage) => {
                let had_existing_state = storage.has_existing_state();
                if had_existing_state && creating_new {
                    let message = t(
                        "Ứng dụng đã có dữ liệu. Vui lòng đăng nhập bằng passphrase hiện tại.",
                        "Application data already exists. Please login with your current passphrase.",
                    )
                    .to_string();
                    self.error = Some(AppError::crypto("login", &message));
                    self.login_view.set_error(message);
                    return Task::none();
                }

                match storage.load_state(passphrase.expose_secret()) {
                    Ok(mut state) => {
                        if !had_existing_state {
                            if !creating_new {
                                let message = t(
                                    "Chưa có dữ liệu. Vui lòng tạo passphrase mới hoặc dùng Import backup ở màn hình này.",
                                    "No existing data found. Please create a new passphrase or import backup on this screen.",
                                )
                                .to_string();
                                self.error = Some(AppError::crypto("login", &message));
                                self.login_view.set_error(message);
                                return Task::none();
                            }

                            let normalized_nickname = normalize_nickname(nickname.as_deref())
                                .ok_or_else(|| {
                                    t(
                                        "Vui lòng nhập nickname hợp lệ",
                                        "Please enter a valid nickname",
                                    )
                                    .to_string()
                                });

                            match normalized_nickname {
                                Ok(value) => {
                                    state.profile.nickname = Some(value);
                                    state.profile.language = self.language;
                                    if let Err(err) =
                                        storage.save_state(&state, passphrase.expose_secret())
                                    {
                                        let message = format!(
                                            "{}: {err}",
                                            t(
                                                "Không thể khởi tạo dữ liệu mới",
                                                "Failed to initialize new app data",
                                            )
                                        );
                                        self.error = Some(AppError::crypto("login", &message));
                                        self.login_view.set_error(message);
                                        return Task::none();
                                    }
                                }
                                Err(message) => {
                                    self.error = Some(AppError::crypto("login", &message));
                                    self.login_view.set_error(message);
                                    return Task::none();
                                }
                            }
                        }

                        match state.into_runtime() {
                            Ok(runtime_state) => {
                                self.storage_passphrase = Some(passphrase);
                                self.restore_runtime_state(runtime_state);
                            }
                            Err(err) => {
                                let message = format!(
                                    "{}: {err}",
                                    t(
                                        "Không thể khôi phục secret wallet",
                                        "Failed to restore wallet secrets",
                                    )
                                );
                                self.error = Some(AppError::crypto("login", &message));
                                self.login_view.set_error(message);
                                return Task::none();
                            }
                        }

                        if had_existing_state {
                            self.add_success_toast(format!(
                                "{} {}, {} {}",
                                t("Chào mừng quay lại,", "Welcome back,"),
                                self.display_name(),
                                t("đã tải", "loaded"),
                                wallet_count_text(self.wallets.len())
                            ));
                        } else {
                            self.add_success_toast(format!(
                                "{} {}! {}",
                                t("Xin chào,", "Welcome,"),
                                self.display_name(),
                                t("Hãy tạo ví đầu tiên của bạn.", "Create your first wallet."),
                            ));
                        }
                    }
                    Err(err) => {
                        let message = format!("{}: {err}", t("Đăng nhập thất bại", "Login failed"));
                        self.error = Some(AppError::crypto("login", &message));
                        self.login_view.set_error(message);
                    }
                }
            }
            Err(err) => {
                self.error = Some(AppError::storage(
                    "init_storage",
                    &format!(
                        "{}: {err}",
                        t("Không thể khởi tạo storage", "Failed to initialize storage")
                    ),
                ));
            }
        }

        Task::none()
    }

    pub fn handle_initial_import_backup(
        &mut self,
        backup_path: String,
        passphrase: String,
    ) -> Task<AppMessage> {
        let passphrase = SecretString::from(passphrase);
        self.error = None;

        if passphrase.expose_secret().trim().is_empty() {
            let message = t(
                "Passphrase không được để trống",
                "Passphrase must not be empty",
            )
            .to_string();
            self.error = Some(AppError::crypto("login", &message));
            self.login_view.set_error(message);
            return Task::none();
        }

        let import_path = resolve_user_path(&backup_path);

        match Storage::new() {
            Ok(storage) => {
                if storage.has_existing_state() {
                    let message = t(
                        "Ứng dụng đã có dữ liệu. Chỉ import backup từ màn hình này khi chưa tạo passphrase.",
                        "Application data already exists. Import backup here is only allowed before creating a passphrase.",
                    )
                    .to_string();
                    self.error = Some(AppError::crypto("login", &message));
                    self.login_view.set_error(message);
                    return Task::none();
                }

                match storage.import_backup(&import_path, passphrase.expose_secret()) {
                    Ok(state) => {
                        if let Err(err) = storage.save_state(&state, passphrase.expose_secret()) {
                            let message = format!(
                                "{}: {err}",
                                t(
                                    "Không thể lưu dữ liệu backup vào app",
                                    "Failed to save imported backup into app storage",
                                )
                            );
                            self.error = Some(AppError::crypto("login", &message));
                            self.login_view.set_error(message);
                            return Task::none();
                        }

                        match state.into_runtime() {
                            Ok(runtime_state) => {
                                self.storage_passphrase = Some(passphrase);
                                self.restore_runtime_state(runtime_state);
                            }
                            Err(err) => {
                                let message = format!(
                                    "{}: {err}",
                                    t(
                                        "Không thể khôi phục secret wallet",
                                        "Failed to restore wallet secrets",
                                    )
                                );
                                self.error = Some(AppError::crypto("login", &message));
                                self.login_view.set_error(message);
                                return Task::none();
                            }
                        }
                        self.add_success_toast(format!(
                            "{} {} {} {}",
                            t("Đã import", "Imported"),
                            wallet_count_text(self.wallets.len()),
                            t("từ", "from"),
                            import_path.display()
                        ));
                        self.error = None;
                    }
                    Err(err) => {
                        let message = format!(
                            "{}: {err}",
                            t("Import backup thất bại", "Backup import failed")
                        );
                        self.error = Some(AppError::crypto("login", &message));
                        self.login_view.set_error(message);
                    }
                }
            }
            Err(err) => {
                let message = format!(
                    "{}: {err}",
                    t("Không thể khởi tạo storage", "Failed to initialize storage")
                );
                self.error = Some(AppError::crypto("login", &message));
                self.login_view.set_error(message);
            }
        }

        Task::none()
    }

    pub fn handle_login_message(&mut self, msg: LoginMessage) -> Task<AppMessage> {
        if let Some(event) = self.login_view.update(msg) {
            match event {
                LoginEvent::ChangeLanguage(language) => {
                    self.language = language;
                    set_current_language(language);
                    self.save_language_preference();
                }
                LoginEvent::BrowseBackupPath => {
                    if let Some(path) = pick_import_backup_path() {
                        self.login_view
                            .set_backup_path(path.to_string_lossy().to_string());
                    }
                }
                LoginEvent::SubmitExisting { passphrase } => {
                    let task = self.handle_login(passphrase, None, false);
                    self.login_view.clear_sensitive_inputs();
                    return task;
                }
                LoginEvent::SubmitNew {
                    passphrase,
                    nickname,
                } => {
                    let task = self.handle_login(passphrase, Some(nickname), true);
                    self.login_view.clear_sensitive_inputs();
                    return task;
                }
                LoginEvent::SubmitImport {
                    backup_path,
                    passphrase,
                } => {
                    let task = self.handle_initial_import_backup(backup_path, passphrase);
                    self.login_view.clear_sensitive_inputs();
                    return task;
                }
            }
        }
        Task::none()
    }
}
