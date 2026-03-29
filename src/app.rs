use std::{
    env,
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use iced::{
    clipboard,
    widget::{column, container, row, text, Space},
    Element, Length, Task,
};
use printpdf::{BuiltinFont, Mm, PdfDocument};

use crate::i18n::{set_current_language, t, AppLanguage};
use crate::storage::{PersistedState, Storage, UserProfile};
use crate::views::{
    dashboard::{DashboardMessage, DashboardView},
    history::{HistoryMessage, HistoryView, HistoryEvent},
    login::{LoginMessage, LoginMode, LoginView, LoginEvent},
    receive::{ReceiveMessage, ReceiveView, ReceiveEvent},
    send::{SendMessage, SendView, SendEvent},
    settings::{SettingsMessage, SettingsView, SettingsEvent},
    sidebar::{NavItem, Sidebar, SidebarMessage, SidebarEvent},
    wallets::{WalletsMessage, WalletsView, WalletsEvent},
};
use crate::wallet::{
    ChangeStrategy, FeeMode, InputSource, TxBuildOptions, Wallet, WalletEntry, WalletNetwork,
};

#[derive(Debug, Clone)]
pub enum AppState {
    Login,
    Main,
}

#[derive(Debug, Clone)]
pub struct SendRequest {
    pub to_address: String,
    pub amount_sat: Option<u64>,
    pub fee_mode: FeeMode,
    pub use_all_funds: bool,
    pub input_source: InputSource,
    pub change_strategy: ChangeStrategy,
    pub broadcast: bool,
}

pub struct App {
    state: AppState,
    storage_passphrase: Option<String>,
    language: AppLanguage,
    user_nickname: Option<String>,
    wallets: Vec<WalletEntry>,
    selected_wallet: usize,

    login_view: LoginView,
    sidebar: Sidebar,
    dashboard: DashboardView,
    wallets_view: WalletsView,
    send_view: SendView,
    receive_view: ReceiveView,
    history_view: HistoryView,
    settings_view: SettingsView,

    current_page: NavItem,
    status: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    Login {
        passphrase: String,
        nickname: Option<String>,
        creating_new: bool,
    },
    InitialImportBackup {
        backup_path: String,
        passphrase: String,
    },
    PickImportBackupPath,
    PickExportBackupPath(String),
    LoginMessage(LoginMessage),

    Navigate(NavItem),
    SidebarMessage(SidebarMessage),
    DashboardMessage(DashboardMessage),
    WalletsMessage(WalletsMessage),
    SendMessage(SendMessage),
    ReceiveMessage(ReceiveMessage),
    HistoryMessage(HistoryMessage),
    SettingsMessage(SettingsMessage),
    ChangeLanguage(AppLanguage),

    CreateWallet(String, WalletNetwork),
    ImportWalletFromMnemonic {
        name: String,
        network: WalletNetwork,
        mnemonic: String,
    },
    ImportWalletFromSlip39 {
        name: String,
        network: WalletNetwork,
        shares: Vec<String>,
        slip39_passphrase: String,
    },
    DeleteWallet(usize),
    SelectWallet(usize),
    RefreshHistory,
    DeriveAddresses(u32),
    RevealMnemonic {
        wallet_index: usize,
        passphrase: String,
    },
    VerifyMnemonicBackup {
        wallet_index: usize,
        checks: Vec<(usize, String)>,
    },
    ExportMnemonicPdf(usize),
    ExportWalletSlip39 {
        wallet_index: usize,
        threshold: u8,
        share_count: u8,
        slip39_passphrase: String,
    },

    EstimateSendFee {
        amount_sat: u64,
        input_source: InputSource,
    },
    SendTransaction(SendRequest),
    CopyAddress(String),

    ChangePassphrase {
        current: String,
        new_passphrase: String,
    },
    ExportWalletBackup(String),
    ClearAllData(String),
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
                current_page: NavItem::Dashboard,
                status: None,
                error: None,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        t("Ví Bitcoin", "Bitcoin Wallet").to_string()
    }

    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::Login {
                passphrase,
                nickname,
                creating_new,
            } => {
                self.status = None;
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
                            self.error = Some(message.clone());
                            self.login_view.set_error(message);
                            return Task::none();
                        }

                        match storage.load_state(&passphrase) {
                            Ok(mut state) => {
                                if !had_existing_state {
                                    if !creating_new {
                                        let message = t(
                                            "Chưa có dữ liệu. Vui lòng tạo passphrase mới hoặc dùng Import backup ở màn hình này.",
                                            "No existing data found. Please create a new passphrase or import backup on this screen.",
                                        )
                                        .to_string();
                                        self.error = Some(message.clone());
                                        self.login_view.set_error(message);
                                        return Task::none();
                                    }

                                    let normalized_nickname =
                                        normalize_nickname(nickname.as_deref()).ok_or_else(|| {
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
                                                storage.save_state(&state, &passphrase)
                                            {
                                                let message = format!(
                                                    "{}: {err}",
                                                    t(
                                                        "Không thể khởi tạo dữ liệu mới",
                                                        "Failed to initialize new app data",
                                                    )
                                                );
                                                self.error = Some(message.clone());
                                                self.login_view.set_error(message);
                                                return Task::none();
                                            }
                                        }
                                        Err(message) => {
                                            self.error = Some(message.clone());
                                            self.login_view.set_error(message);
                                            return Task::none();
                                        }
                                    }
                                }

                                self.user_nickname =
                                    normalize_nickname(state.profile.nickname.as_deref());
                                self.language = state.profile.language;
                                set_current_language(self.language);
                                self.save_language_preference();
                                self.storage_passphrase = Some(passphrase);
                                self.wallets = state.wallets;
                                self.state = AppState::Main;
                                self.selected_wallet = self
                                    .selected_wallet
                                    .min(self.wallets.len().saturating_sub(1));
                                self.update_dashboard();
                                self.login_view.clear_error();

                                if had_existing_state {
                                    self.status = Some(format!(
                                        "{} {}, {} {}",
                                        t("Chào mừng quay lại,", "Welcome back,"),
                                        self.display_name(),
                                        t("đã tải", "loaded"),
                                        wallet_count_text(self.wallets.len())
                                    ));
                                } else {
                                    self.status = Some(format!(
                                        "{} {}! {}",
                                        t("Xin chào,", "Welcome,"),
                                        self.display_name(),
                                        t(
                                            "Hãy tạo ví đầu tiên của bạn.",
                                            "Create your first wallet."
                                        ),
                                    ));
                                }
                            }
                            Err(err) => {
                                let message =
                                    format!("{}: {err}", t("Đăng nhập thất bại", "Login failed"));
                                self.error = Some(message.clone());
                                self.login_view.set_error(message);
                            }
                        }
                    }
                    Err(err) => {
                        self.error = Some(format!(
                            "{}: {err}",
                            t("Không thể khởi tạo storage", "Failed to initialize storage")
                        ));
                    }
                }

                Task::none()
            }

            AppMessage::InitialImportBackup {
                backup_path,
                passphrase,
            } => {
                self.status = None;
                self.error = None;

                if passphrase.trim().is_empty() {
                    let message = t(
                        "Passphrase không được để trống",
                        "Passphrase must not be empty",
                    )
                    .to_string();
                    self.error = Some(message.clone());
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
                            self.error = Some(message.clone());
                            self.login_view.set_error(message);
                            return Task::none();
                        }

                        match storage.import_backup(&import_path, &passphrase) {
                            Ok(state) => {
                                if let Err(err) = storage.save_state(&state, &passphrase) {
                                    let message = format!(
                                        "{}: {err}",
                                        t(
                                            "Không thể lưu dữ liệu backup vào app",
                                            "Failed to save imported backup into app storage",
                                        )
                                    );
                                    self.error = Some(message.clone());
                                    self.login_view.set_error(message);
                                    return Task::none();
                                }

                                self.user_nickname =
                                    normalize_nickname(state.profile.nickname.as_deref());
                                self.language = state.profile.language;
                                set_current_language(self.language);
                                self.save_language_preference();
                                self.storage_passphrase = Some(passphrase);
                                self.wallets = state.wallets;
                                self.state = AppState::Main;
                                self.selected_wallet = self
                                    .selected_wallet
                                    .min(self.wallets.len().saturating_sub(1));
                                self.update_dashboard();
                                self.login_view.clear_error();
                                self.status = Some(format!(
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
                                self.error = Some(message.clone());
                                self.login_view.set_error(message);
                            }
                        }
                    }
                    Err(err) => {
                        let message = format!(
                            "{}: {err}",
                            t("Không thể khởi tạo storage", "Failed to initialize storage")
                        );
                        self.error = Some(message.clone());
                        self.login_view.set_error(message);
                    }
                }

                Task::none()
            }

            AppMessage::PickImportBackupPath => {
                if let Some(path) = pick_import_backup_path() {
                    self.login_view
                        .set_backup_path(path.to_string_lossy().to_string());
                }
                Task::none()
            }

            AppMessage::PickExportBackupPath(current_path) => {
                if let Some(path) = pick_export_backup_path(&current_path) {
                    self.settings_view
                        .set_export_path(path.to_string_lossy().to_string());
                }
                Task::none()
            }

            AppMessage::LoginMessage(msg) => {
                if let Some(event) = self.login_view.update(msg) {
                    match event {
                        LoginEvent::ChangeLanguage(language) => {
                            self.language = language;
                            set_current_language(language);
                            self.save_language_preference();
                        }
                        LoginEvent::BrowseBackupPath => {
                            if let Some(path) = pick_import_backup_path() {
                                self.login_view.set_backup_path(path.to_string_lossy().to_string());
                            }
                        }
                        LoginEvent::SubmitExisting { passphrase } => {
                            return self.update(AppMessage::Login {
                                passphrase,
                                nickname: None,
                                creating_new: false,
                            });
                        }
                        LoginEvent::SubmitNew { passphrase, nickname } => {
                            return self.update(AppMessage::Login {
                                passphrase,
                                nickname: Some(nickname),
                                creating_new: true,
                            });
                        }
                        LoginEvent::SubmitImport { backup_path, passphrase } => {
                            return self.update(AppMessage::InitialImportBackup { backup_path, passphrase });
                        }
                    }
                }
                Task::none()
            }

            AppMessage::Navigate(page) => {
                self.current_page = page;
                self.sidebar.set_active(page);
                Task::none()
            }

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

            AppMessage::DashboardMessage(msg) => {
                match msg {
                    DashboardMessage::Refresh => self.refresh_all_wallets(),
                }
                Task::none()
            }

            AppMessage::WalletsMessage(msg) => {
                if let Some(event) = self.wallets_view.update(msg) {
                    match event {
                        WalletsEvent::CreateWallet(name, network) => {
                            return self.update(AppMessage::CreateWallet(name, network));
                        }
                        WalletsEvent::ImportWalletFromMnemonic { name, network, mnemonic } => {
                            return self.update(AppMessage::ImportWalletFromMnemonic { name, network, mnemonic });
                        }
                        WalletsEvent::ImportWalletFromSlip39 { name, network, shares, slip39_passphrase } => {
                            return self.update(AppMessage::ImportWalletFromSlip39 { name, network, shares, slip39_passphrase });
                        }
                        WalletsEvent::SelectWallet(index) => {
                            return self.update(AppMessage::SelectWallet(index));
                        }
                        WalletsEvent::DeleteWallet(index) => {
                            return self.update(AppMessage::DeleteWallet(index));
                        }
                        WalletsEvent::RevealMnemonic { wallet_index, passphrase } => {
                            return self.update(AppMessage::RevealMnemonic { wallet_index, passphrase });
                        }
                        WalletsEvent::VerifyMnemonicBackup { wallet_index, checks } => {
                            return self.update(AppMessage::VerifyMnemonicBackup { wallet_index, checks });
                        }
                        WalletsEvent::ExportMnemonicPdf(index) => {
                            return self.update(AppMessage::ExportMnemonicPdf(index));
                        }
                        WalletsEvent::ExportWalletSlip39 { wallet_index, threshold, share_count, slip39_passphrase } => {
                            return self.update(AppMessage::ExportWalletSlip39 { wallet_index, threshold, share_count, slip39_passphrase });
                        }
                    }
                }
                Task::none()
            }

            AppMessage::SendMessage(msg) => {
                if let Some(event) = self.send_view.update(msg) {
                    match event {
                        SendEvent::SelectWallet(index) => return self.update(AppMessage::SelectWallet(index)),
                        SendEvent::EstimateSendFee { amount_sat, input_source } => return self.update(AppMessage::EstimateSendFee { amount_sat, input_source }),
                        SendEvent::SendTransaction(req) => return self.update(AppMessage::SendTransaction(req)),
                    }
                }
                Task::none()
            }

            AppMessage::ReceiveMessage(msg) => {
                if let Some(event) = self.receive_view.update(msg) {
                    match event {
                        ReceiveEvent::SelectWallet(index) => return self.update(AppMessage::SelectWallet(index)),
                        ReceiveEvent::CopyAddress(addr) => return self.update(AppMessage::CopyAddress(addr)),
                        ReceiveEvent::DeriveAddresses(count) => return self.update(AppMessage::DeriveAddresses(count)),
                    }
                }
                Task::none()
            }

            AppMessage::HistoryMessage(msg) => {
                if let Some(event) = self.history_view.update(msg) {
                    match event {
                        HistoryEvent::Refresh => return self.update(AppMessage::RefreshHistory),
                    }
                }
                Task::none()
            }

            AppMessage::SettingsMessage(msg) => {
                if let Some(event) = self.settings_view.update(msg) {
                    match event {
                        SettingsEvent::ChangeLanguage(language) => return self.update(AppMessage::ChangeLanguage(language)),
                        SettingsEvent::BrowseExportPath => {
                            if let Some(path) = pick_export_backup_path("") {
                                self.settings_view.set_export_path(path.to_string_lossy().to_string());
                            }
                        }
                        SettingsEvent::ChangePassphrase { current, new_passphrase } => return self.update(AppMessage::ChangePassphrase { current, new_passphrase }),
                        SettingsEvent::ExportWallet(path) => return self.update(AppMessage::ExportWalletBackup(path)),
                        SettingsEvent::ClearAllData(passphrase) => return self.update(AppMessage::ClearAllData(passphrase)),
                    }
                }
                Task::none()
            }

            AppMessage::ChangeLanguage(language) => {
                self.language = language;
                set_current_language(language);
                self.save_language_preference();
                if matches!(self.state, AppState::Main) {
                    self.settings_view.set_success(t(
                        "Đã đổi ngôn ngữ ứng dụng",
                        "Application language updated",
                    ));
                }
                self.save_state();
                Task::none()
            }

            AppMessage::CreateWallet(name, network) => {
                match Wallet::new(&name, network) {
                    Ok(wallet) => {
                        self.wallets.push(wallet.entry);
                        self.selected_wallet = self.wallets.len() - 1;
                        self.save_state();
                        self.update_dashboard();
                        self.wallets_view = WalletsView::new();
                        self.wallets_view.set_info(t(
                            "Ví mới đã tạo. Hãy backup mnemonic ngay và hoàn thành bài test.",
                            "New wallet created. Please back up the mnemonic now and complete the backup test.",
                        ));
                        self.status = Some(format!(
                            "{} '{name}'. {}",
                            t("Đã tạo ví thành công", "Wallet created successfully"),
                            t("Cần backup mnemonic.", "Mnemonic backup is required.")
                        ));
                        self.error = None;
                    }
                    Err(err) => {
                        self.error = Some(format!(
                            "{}: {err}",
                            t("Tạo ví thất bại", "Failed to create wallet")
                        ));
                    }
                }
                Task::none()
            }

            AppMessage::ImportWalletFromMnemonic {
                name,
                network,
                mnemonic,
            } => {
                match Wallet::from_mnemonic(&name, network, &mnemonic) {
                    Ok(wallet) => {
                        self.wallets.push(wallet.entry);
                        self.selected_wallet = self.wallets.len() - 1;
                        self.save_state();
                        self.update_dashboard();
                        self.wallets_view = WalletsView::new();
                        self.wallets_view.set_info(t(
                            "Import mnemonic thành công. Ví này đã được đánh dấu backup.",
                            "Mnemonic import succeeded. This wallet has been marked as backed up.",
                        ));
                        self.status = Some(format!(
                            "{} '{name}' {}",
                            t("Đã import ví", "Imported wallet"),
                            t("từ mnemonic", "from mnemonic")
                        ));
                        self.error = None;
                    }
                    Err(err) => {
                        let message = format!(
                            "{}: {err}",
                            t("Import mnemonic thất bại", "Mnemonic import failed")
                        );
                        self.wallets_view.set_error(message.clone());
                        self.error = Some(message);
                    }
                }
                Task::none()
            }

            AppMessage::ImportWalletFromSlip39 {
                name,
                network,
                shares,
                slip39_passphrase,
            } => {
                match Wallet::from_slip39_shares(&name, network, &shares, &slip39_passphrase) {
                    Ok(wallet) => {
                        self.wallets.push(wallet.entry);
                        self.selected_wallet = self.wallets.len() - 1;
                        self.save_state();
                        self.update_dashboard();
                        self.wallets_view = WalletsView::new();
                        self.wallets_view.set_info(t(
                            "Import SLIP-0039 thành công. Ví này đã được đánh dấu backup.",
                            "SLIP-0039 import succeeded. This wallet has been marked as backed up.",
                        ));
                        self.status = Some(format!(
                            "{} '{name}' {}",
                            t("Đã import ví", "Imported wallet"),
                            t("từ SLIP-0039", "from SLIP-0039")
                        ));
                        self.error = None;
                    }
                    Err(err) => {
                        let message = format!(
                            "{}: {err}",
                            t("Import SLIP-0039 thất bại", "SLIP-0039 import failed")
                        );
                        self.wallets_view.set_error(message.clone());
                        self.error = Some(message);
                    }
                }
                Task::none()
            }

            AppMessage::SelectWallet(index) => {
                if index < self.wallets.len() {
                    self.selected_wallet = index;
                    self.status = Some(format!(
                        "{}: {}",
                        t("Đã chọn ví", "Selected wallet"),
                        self.wallets[index].name
                    ));
                    self.error = None;
                }
                Task::none()
            }

            AppMessage::DeleteWallet(index) => {
                if index < self.wallets.len() {
                    let name = self.wallets[index].name.clone();
                    self.wallets.remove(index);

                    if self.wallets.is_empty() {
                        self.selected_wallet = 0;
                    } else if self.selected_wallet >= self.wallets.len() {
                        self.selected_wallet = self.wallets.len() - 1;
                    }

                    self.save_state();
                    self.update_dashboard();
                    self.status = Some(format!("{} '{name}'", t("Đã xóa ví", "Deleted wallet")));
                    self.error = None;
                }
                Task::none()
            }

            AppMessage::RefreshHistory => {
                self.refresh_all_wallets();
                Task::none()
            }

            AppMessage::DeriveAddresses(count) => {
                if let Some(wallet_entry) = self.wallets.get_mut(self.selected_wallet) {
                    let mut wallet = Wallet {
                        entry: wallet_entry.clone(),
                    };
                    match wallet.derive_next_addresses(count) {
                        Ok(addresses) => {
                            *wallet_entry = wallet.entry;
                            self.save_state();
                            self.status = Some(format!(
                                "{} {}",
                                t("Đã tạo", "Derived"),
                                address_count_text(addresses.len())
                            ));
                            self.error = None;
                        }
                        Err(err) => {
                            self.error = Some(format!(
                                "{}: {err}",
                                t(
                                    "Không thể tạo địa chỉ mới",
                                    "Could not derive new addresses"
                                )
                            ));
                        }
                    }
                } else {
                    self.error = Some(t("Chưa chọn ví", "No wallet selected").to_string());
                }
                Task::none()
            }

            AppMessage::RevealMnemonic {
                wallet_index,
                passphrase,
            } => {
                let active_passphrase = match &self.storage_passphrase {
                    Some(value) => value.clone(),
                    None => {
                        self.wallets_view.set_error(t(
                            "Không có session đăng nhập hợp lệ",
                            "No active login session found",
                        ));
                        return Task::none();
                    }
                };

                if wallet_index >= self.wallets.len() {
                    self.wallets_view
                        .set_error(t("Ví không tồn tại", "Wallet does not exist"));
                    return Task::none();
                }

                if passphrase != active_passphrase {
                    self.wallets_view.set_error(t(
                        "Passphrase không đúng, không thể hiển thị mnemonic",
                        "Incorrect passphrase, cannot reveal mnemonic",
                    ));
                    return Task::none();
                }

                let wallet_name = self.wallets[wallet_index].name.clone();
                if self.wallets[wallet_index].mnemonic.is_none() {
                    self.wallets_view.set_error(t(
                        "Ví này không có mnemonic để hiển thị",
                        "This wallet has no mnemonic to reveal",
                    ));
                    return Task::none();
                }

                self.wallets_view.mark_mnemonic_revealed(wallet_index);
                self.status = Some(format!(
                    "{} '{wallet_name}'",
                    t("Đã mở khóa mnemonic cho ví", "Mnemonic unlocked for wallet")
                ));
                self.error = None;
                Task::none()
            }

            AppMessage::VerifyMnemonicBackup {
                wallet_index,
                checks,
            } => {
                if wallet_index >= self.wallets.len() {
                    self.wallets_view
                        .set_error(t("Ví không tồn tại", "Wallet does not exist"));
                    return Task::none();
                }

                let verification = {
                    let wallet = &self.wallets[wallet_index];
                    let mnemonic = match &wallet.mnemonic {
                        Some(value) => value,
                        None => {
                            self.wallets_view.set_error(t(
                                "Ví này không có mnemonic để xác thực backup",
                                "This wallet has no mnemonic for backup verification",
                            ));
                            return Task::none();
                        }
                    };

                    let words: Vec<&str> = mnemonic.split_whitespace().collect();
                    if words.is_empty() {
                        self.wallets_view
                            .set_error(t("Mnemonic không hợp lệ", "Invalid mnemonic"));
                        return Task::none();
                    }

                    if checks.is_empty() {
                        self.wallets_view.set_error(t(
                            "Thiếu dữ liệu bài test backup",
                            "Missing backup test data",
                        ));
                        return Task::none();
                    }

                    let mut wrong_positions = Vec::new();
                    for (position, input_word) in &checks {
                        let pos = *position;
                        if pos == 0 || pos > words.len() {
                            self.wallets_view.set_error(t(
                                "Vị trí từ trong bài test không hợp lệ",
                                "Invalid word position in backup test",
                            ));
                            return Task::none();
                        }

                        let expected = words[pos - 1];
                        if !expected.eq_ignore_ascii_case(input_word.trim()) {
                            wrong_positions.push(pos);
                        }
                    }

                    if wrong_positions.is_empty() {
                        Ok(())
                    } else {
                        Err(wrong_positions)
                    }
                };

                match verification {
                    Ok(()) => {
                        let wallet_name = self.wallets[wallet_index].name.clone();
                        if let Some(wallet) = self.wallets.get_mut(wallet_index) {
                            wallet.mnemonic_backed_up = true;
                        }

                        self.save_state();
                        self.wallets_view.mark_backup_verified(wallet_index);
                        self.status = Some(format!(
                            "{} '{wallet_name}'",
                            t(
                                "Ví đã vượt qua bài test backup mnemonic",
                                "Wallet passed mnemonic backup test",
                            )
                        ));
                        self.error = None;
                    }
                    Err(wrong_positions) => {
                        self.wallets_view.set_error(format!(
                            "{}: {}",
                            t(
                                "Bài test chưa đúng ở vị trí",
                                "Backup test is incorrect at positions"
                            ),
                            wrong_positions
                                .iter()
                                .map(usize::to_string)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }

                Task::none()
            }

            AppMessage::ExportMnemonicPdf(wallet_index) => {
                if wallet_index >= self.wallets.len() {
                    self.wallets_view
                        .set_error(t("Ví không tồn tại", "Wallet does not exist"));
                    return Task::none();
                }

                let wallet = &self.wallets[wallet_index];
                let mnemonic = match wallet.mnemonic.as_deref() {
                    Some(value) => value,
                    None => {
                        self.wallets_view.set_error(t(
                            "Ví này không có mnemonic để export PDF",
                            "This wallet has no mnemonic to export as PDF",
                        ));
                        return Task::none();
                    }
                };

                let default_name = default_mnemonic_pdf_filename(&wallet.name);
                let Some(raw_path) = pick_mnemonic_pdf_path(&default_name) else {
                    return Task::none();
                };
                let export_path = ensure_pdf_extension(raw_path);

                match export_mnemonic_to_pdf(
                    &export_path,
                    &wallet.name,
                    wallet.network.as_str(),
                    mnemonic,
                ) {
                    Ok(_) => {
                        let message = format!(
                            "{}: {}",
                            t("Đã export mnemonic PDF", "Exported mnemonic PDF"),
                            export_path.display()
                        );
                        self.wallets_view.set_info(message.clone());
                        self.status = Some(message);
                        self.error = None;
                    }
                    Err(err) => {
                        self.wallets_view.set_error(format!(
                            "{}: {err}",
                            t(
                                "Export mnemonic PDF thất bại",
                                "Failed to export mnemonic PDF"
                            )
                        ));
                    }
                }
                Task::none()
            }

            AppMessage::ExportWalletSlip39 {
                wallet_index,
                threshold,
                share_count,
                slip39_passphrase,
            } => {
                if wallet_index >= self.wallets.len() {
                    self.wallets_view
                        .set_error(t("Ví không tồn tại", "Wallet does not exist"));
                    return Task::none();
                }

                let wallet = &self.wallets[wallet_index];
                let mnemonic = match wallet.mnemonic.as_deref() {
                    Some(value) => value,
                    None => {
                        self.wallets_view.set_error(t(
                            "Ví này không có mnemonic để export SLIP-0039",
                            "This wallet has no mnemonic to export as SLIP-0039",
                        ));
                        return Task::none();
                    }
                };

                let shares = match Wallet::split_mnemonic_to_slip39_shares(
                    mnemonic,
                    threshold,
                    share_count,
                    &slip39_passphrase,
                ) {
                    Ok(value) => value,
                    Err(err) => {
                        self.wallets_view.set_error(format!(
                            "{}: {err}",
                            t("Không thể tách SLIP-0039", "Could not split to SLIP-0039")
                        ));
                        return Task::none();
                    }
                };

                let default_dir_name =
                    default_slip39_directory_name(&wallet.name, threshold, share_count);
                let Some(base_directory) = pick_slip39_export_directory() else {
                    return Task::none();
                };

                match export_slip39_shares_to_pdf_directory(
                    &base_directory,
                    &default_dir_name,
                    &wallet.name,
                    wallet.network.as_str(),
                    threshold,
                    share_count,
                    !slip39_passphrase.trim().is_empty(),
                    &shares,
                ) {
                    Ok(export_directory) => {
                        let message = format!(
                            "{}: {}",
                            t(
                                "Đã export SLIP-0039 shares PDF tại",
                                "Exported SLIP-0039 shares PDF to",
                            ),
                            export_directory.display()
                        );
                        self.wallets_view.set_info(message.clone());
                        self.status = Some(message);
                        self.error = None;
                    }
                    Err(err) => {
                        self.wallets_view.set_error(format!(
                            "{}: {err}",
                            t("Export SLIP-0039 thất bại", "Failed to export SLIP-0039")
                        ));
                    }
                }
                Task::none()
            }

            AppMessage::EstimateSendFee {
                amount_sat,
                input_source,
            } => {
                if let Some(wallet_entry) = self.wallets.get(self.selected_wallet) {
                    let wallet = Wallet {
                        entry: wallet_entry.clone(),
                    };
                    match wallet.estimate_auto_fee_for_amount(amount_sat, &input_source) {
                        Ok(fee) => {
                            self.send_view.set_estimated_fee(fee);
                            self.status =
                                Some(format!("{}: {fee} sat", t("Phí ước tính", "Estimated fee")));
                            self.error = None;
                        }
                        Err(err) => {
                            self.send_view.set_error(err.to_string());
                            self.error = Some(format!(
                                "{}: {err}",
                                t("Ước tính phí thất bại", "Fee estimation failed")
                            ));
                        }
                    }
                } else {
                    let message = t("Chưa chọn ví", "No wallet selected").to_string();
                    self.send_view.set_error(message.clone());
                    self.error = Some(message);
                }
                Task::none()
            }

            AppMessage::SendTransaction(request) => {
                if let Some(wallet_entry) = self.wallets.get_mut(self.selected_wallet) {
                    let mut wallet = Wallet {
                        entry: wallet_entry.clone(),
                    };

                    let tx_options = TxBuildOptions {
                        broadcast: request.broadcast,
                        input_source: request.input_source.clone(),
                        change_strategy: request.change_strategy.clone(),
                    };

                    let result = if request.use_all_funds {
                        wallet.create_send_all_transaction_with_options(
                            &request.to_address,
                            request.fee_mode,
                            tx_options,
                        )
                    } else {
                        let amount_sat = match request.amount_sat {
                            Some(value) if value > 0 => value,
                            _ => {
                                self.send_view.set_error(t(
                                    "Số lượng không hợp lệ cho giao dịch thường",
                                    "Invalid amount for regular transaction",
                                ));
                                return Task::none();
                            }
                        };

                        let fee_sat = match request.fee_mode {
                            FeeMode::Auto => match wallet
                                .estimate_auto_fee_for_amount(amount_sat, &request.input_source)
                            {
                                Ok(value) => value,
                                Err(err) => {
                                    self.send_view.set_error(format!(
                                        "{}: {err}",
                                        t(
                                            "Không thể ước tính phí tự động",
                                            "Could not estimate auto fee",
                                        )
                                    ));
                                    return Task::none();
                                }
                            },
                            FeeMode::FixedSat(value) => value,
                        };

                        wallet.create_transaction_with_options(
                            &request.to_address,
                            amount_sat,
                            fee_sat,
                            tx_options,
                        )
                    };

                    match result {
                        Ok(tx_result) => {
                            *wallet_entry = wallet.entry;
                            self.save_state();
                            self.update_dashboard();

                            let short_txid = short_txid(&tx_result.txid);
                            let send_message = if tx_result.broadcasted {
                                format!(
                                    "{}: {short_txid}",
                                    t("Đã broadcast giao dịch", "Transaction broadcasted")
                                )
                            } else {
                                format!(
                                    "{}: {short_txid}",
                                    t(
                                        "Đã tạo giao dịch (chưa broadcast)",
                                        "Transaction created (not broadcast)",
                                    )
                                )
                            };
                            self.send_view.set_success(send_message.clone());
                            self.status = Some(send_message);
                            self.error = None;
                        }
                        Err(err) => {
                            self.send_view.set_error(err.to_string());
                            self.error = Some(format!(
                                "{}: {err}",
                                t("Gửi giao dịch thất bại", "Send transaction failed")
                            ));
                        }
                    }
                } else {
                    let message = t("Chưa chọn ví", "No wallet selected").to_string();
                    self.send_view.set_error(message.clone());
                    self.error = Some(message);
                }

                Task::none()
            }

            AppMessage::CopyAddress(address) => {
                self.status = Some(
                    t(
                        "Đã copy địa chỉ vào clipboard",
                        "Copied address to clipboard",
                    )
                    .to_string(),
                );
                self.error = None;
                clipboard::write(address)
            }

            AppMessage::ChangePassphrase {
                current,
                new_passphrase,
            } => {
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

            AppMessage::ExportWalletBackup(raw_path) => {
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

            AppMessage::ClearAllData(passphrase) => {
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
    }

    pub fn view(&self) -> Element<'_, AppMessage> {
        match self.state {
            AppState::Login => container(self.login_view.view().map(AppMessage::LoginMessage))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
            AppState::Main => {
                let sidebar = self.sidebar.view().map(AppMessage::SidebarMessage);
                let selected_wallet = self.wallets.get(self.selected_wallet);

                let main_content = match self.current_page {
                    NavItem::Dashboard => self.dashboard.view().map(AppMessage::DashboardMessage),
                    NavItem::Wallets => self
                        .wallets_view
                        .view(&self.wallets, self.selected_wallet)
                        .map(AppMessage::WalletsMessage),
                    NavItem::Send => self
                        .send_view
                        .view(&self.wallets, self.selected_wallet)
                        .map(AppMessage::SendMessage),
                    NavItem::Receive => self
                        .receive_view
                        .view(&self.wallets, self.selected_wallet)
                        .map(AppMessage::ReceiveMessage),
                    NavItem::History => self
                        .history_view
                        .view(selected_wallet)
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

                let greeting_bar = container(
                    text(format!(
                        "{} {}",
                        t("Xin chào,", "Hello,"),
                        self.display_name()
                    ))
                    .size(14)
                    .style(crate::theme::text_color(
                        crate::theme::Colors::TEXT_SECONDARY,
                    )),
                )
                .padding(8);

                row![
                    sidebar,
                    column![greeting_bar, status_bar, error_bar, main_content,].width(Length::Fill)
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        }
    }

    fn update_dashboard(&mut self) {
        let total: i64 = self
            .wallets
            .iter()
            .map(|wallet| wallet.history.iter().map(|tx| tx.amount_sat).sum::<i64>())
            .sum();

        let confirmed: i64 = self
            .wallets
            .iter()
            .map(|wallet| {
                wallet
                    .history
                    .iter()
                    .filter(|tx| tx.confirmed)
                    .map(|tx| tx.amount_sat)
                    .sum::<i64>()
            })
            .sum();

        self.dashboard
            .update_balances(total, confirmed, self.wallets.len());
    }

    fn refresh_all_wallets(&mut self) {
        if self.wallets.is_empty() {
            self.status = Some(t("Không có ví để làm mới", "No wallets to refresh").to_string());
            return;
        }

        let mut refreshed_wallets = 0usize;
        let mut refreshed_txs = 0usize;
        let mut errors = Vec::new();

        for wallet_entry in &mut self.wallets {
            let mut wallet = Wallet {
                entry: wallet_entry.clone(),
            };
            match wallet.refresh_history() {
                Ok(count) => {
                    *wallet_entry = wallet.entry;
                    refreshed_wallets += 1;
                    refreshed_txs += count;
                }
                Err(err) => {
                    errors.push(format!("{}: {}", wallet_entry.name, err));
                }
            }
        }

        self.save_state();
        self.update_dashboard();

        self.status = Some(format!(
            "{} {}, {} {}",
            t("Đã làm mới", "Refreshed"),
            wallet_count_text(refreshed_wallets),
            refreshed_txs,
            t("giao dịch", "transaction(s)")
        ));

        if !errors.is_empty() {
            self.error = Some(format!(
                "{}: {}",
                t("Một số ví làm mới lỗi", "Some wallets failed to refresh",),
                errors.join(" | ")
            ));
        } else {
            self.error = None;
        }
    }

    fn save_language_preference(&mut self) {
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

    fn save_state(&mut self) {
        let passphrase = match &self.storage_passphrase {
            Some(value) => value.clone(),
            None => return,
        };

        match Storage::new() {
            Ok(storage) => {
                let state = PersistedState {
                    profile: UserProfile {
                        nickname: self.user_nickname.clone(),
                        language: self.language,
                    },
                    wallets: self.wallets.clone(),
                };
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

    fn reset_to_login(&mut self, allow_create_passphrase: bool) {
        self.state = AppState::Login;
        self.storage_passphrase = None;
        self.user_nickname = None;
        self.wallets.clear();
        self.selected_wallet = 0;
        self.current_page = NavItem::Dashboard;
        self.status = None;
        self.error = None;

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
    }

    fn display_name(&self) -> &str {
        self.user_nickname.as_deref().unwrap_or(t("bạn", "friend"))
    }
}

fn short_txid(txid: &str) -> String {
    let prefix = txid.get(..12).unwrap_or(txid);
    format!("{prefix}...")
}

fn wallet_count_text(count: usize) -> String {
    format!("{} {}", count, t("ví", "wallet(s)"))
}

fn address_count_text(count: usize) -> String {
    format!("{} {}", count, t("địa chỉ mới", "new address(es)"))
}

fn resolve_user_path(raw_path: &str) -> PathBuf {
    let trimmed = raw_path.trim();
    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return Path::new(&home).join(rest);
        }
    }

    PathBuf::from(trimmed)
}

fn pick_import_backup_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t(
            "Chọn file backup để import",
            "Choose backup file to import",
        ))
        .add_filter(t("File backup", "Backup files"), &["enc", "json"])
        .pick_file()
}

fn pick_export_backup_path(current_path: &str) -> Option<PathBuf> {
    let resolved = resolve_user_path(current_path);

    let mut dialog = rfd::FileDialog::new()
        .set_title(t("Chọn nơi lưu backup", "Choose where to save backup"))
        .add_filter(t("Backup mã hóa", "Encrypted backup"), &["enc"]);

    if let Some(parent) = resolved.parent() {
        dialog = dialog.set_directory(parent);
    }

    if let Some(file_name) = resolved.file_name().and_then(|name| name.to_str()) {
        dialog = dialog.set_file_name(file_name);
    } else {
        dialog = dialog.set_file_name("wallet_backup.enc");
    }

    dialog.save_file()
}

fn pick_mnemonic_pdf_path(default_file_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t("Lưu mnemonic ra PDF", "Save mnemonic as PDF"))
        .add_filter(t("File PDF", "PDF file"), &["pdf"])
        .set_file_name(default_file_name)
        .save_file()
}

fn pick_slip39_export_directory() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t(
            "Chọn thư mục chứa backup SLIP-0039",
            "Choose folder for SLIP-0039 backup",
        ))
        .pick_folder()
}

fn default_mnemonic_pdf_filename(wallet_name: &str) -> String {
    format!("{}_mnemonic_backup.pdf", sanitize_filename(wallet_name))
}

fn default_slip39_directory_name(wallet_name: &str, threshold: u8, share_count: u8) -> String {
    format!(
        "{}_slip39_{}of{}",
        sanitize_filename(wallet_name),
        threshold,
        share_count
    )
}

fn sanitize_filename(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            result.push(ch);
        } else if ch.is_whitespace() {
            result.push('_');
        }
    }

    let trimmed = result.trim_matches('_');
    if trimmed.is_empty() {
        "wallet".to_string()
    } else {
        trimmed.to_string()
    }
}

fn ensure_pdf_extension(mut path: PathBuf) -> PathBuf {
    let has_pdf = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false);

    if !has_pdf {
        path.set_extension("pdf");
    }

    path
}

fn export_mnemonic_to_pdf(
    path: &Path,
    wallet_name: &str,
    network: &str,
    mnemonic: &str,
) -> Result<(), String> {
    let (doc, page, layer) =
        PdfDocument::new("Mnemonic Backup", Mm(210.0), Mm(297.0), "Mnemonic Layer");
    let current_layer = doc.get_page(page).get_layer(layer);

    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|err| {
            format!(
                "{}: {err}",
                t("Không tải được font PDF", "Could not load PDF font")
            )
        })?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|err| {
            format!(
                "{}: {err}",
                t("Không tải được font PDF", "Could not load PDF font")
            )
        })?;

    current_layer.use_text(
        "Bitcoin Wallet - Mnemonic Backup",
        18.0,
        Mm(18.0),
        Mm(280.0),
        &font_bold,
    );
    current_layer.use_text(
        format!("Wallet: {wallet_name}"),
        12.0,
        Mm(18.0),
        Mm(268.0),
        &font_regular,
    );
    current_layer.use_text(
        format!("Network: {network}"),
        12.0,
        Mm(18.0),
        Mm(260.0),
        &font_regular,
    );
    current_layer.use_text(
        "Keep this file offline and private. Anyone with these words can spend your funds.",
        10.0,
        Mm(18.0),
        Mm(250.0),
        &font_regular,
    );

    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    for (idx, word) in words.iter().enumerate() {
        let row = idx / 2;
        let col = idx % 2;
        let x = if col == 0 { 18.0 } else { 110.0 };
        let y = 236.0 - (row as f32 * 10.0);

        current_layer.use_text(
            format!("{:02}. {}", idx + 1, word),
            12.0,
            Mm(x),
            Mm(y),
            &font_regular,
        );
    }

    let file = File::create(path).map_err(|err| {
        format!(
            "{} {}: {err}",
            t("Không tạo được file PDF", "Could not create PDF file"),
            path.display()
        )
    })?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer).map_err(|err| {
        format!(
            "{}: {err}",
            t("Không ghi được nội dung PDF", "Could not write PDF content")
        )
    })?;

    Ok(())
}

fn export_slip39_shares_to_pdf_directory(
    base_directory: &Path,
    directory_name: &str,
    wallet_name: &str,
    network: &str,
    threshold: u8,
    share_count: u8,
    has_slip39_passphrase: bool,
    shares: &[String],
) -> Result<PathBuf, String> {
    if shares.is_empty() {
        return Err(t(
            "Không có SLIP-0039 share nào để export",
            "No SLIP-0039 shares available to export",
        )
        .to_string());
    }

    let export_dir = create_unique_export_directory(base_directory, directory_name)?;

    for (index, share) in shares.iter().enumerate() {
        let file_name = format!("share_{:02}_of_{:02}.pdf", index + 1, shares.len());
        let share_path = export_dir.join(file_name);

        export_slip39_share_to_pdf(
            &share_path,
            wallet_name,
            network,
            threshold,
            share_count,
            has_slip39_passphrase,
            index + 1,
            shares.len(),
            share,
        )?;
    }

    Ok(export_dir)
}

fn create_unique_export_directory(
    base_directory: &Path,
    directory_name: &str,
) -> Result<PathBuf, String> {
    if !base_directory.exists() {
        return Err(format!(
            "{}: {}",
            t(
                "Thư mục đích không tồn tại",
                "Destination directory does not exist"
            ),
            base_directory.display()
        ));
    }

    for attempt in 0..1000 {
        let candidate_name = if attempt == 0 {
            directory_name.to_string()
        } else {
            format!("{directory_name}_{attempt}")
        };
        let candidate = base_directory.join(candidate_name);

        if !candidate.exists() {
            fs::create_dir_all(&candidate).map_err(|err| {
                format!(
                    "{} {}: {err}",
                    t(
                        "Không thể tạo thư mục export SLIP-0039",
                        "Could not create SLIP-0039 export directory",
                    ),
                    candidate.display()
                )
            })?;
            return Ok(candidate);
        }
    }

    Err(t(
        "Không thể tạo thư mục export SLIP-0039 (đã thử quá nhiều lần)",
        "Could not create SLIP-0039 export directory (too many attempts)",
    )
    .to_string())
}

fn export_slip39_share_to_pdf(
    path: &Path,
    wallet_name: &str,
    network: &str,
    threshold: u8,
    share_count: u8,
    has_slip39_passphrase: bool,
    share_index: usize,
    share_total: usize,
    share_phrase: &str,
) -> Result<(), String> {
    let (doc, page, layer) = PdfDocument::new(
        "SLIP-0039 Share Backup",
        Mm(210.0),
        Mm(297.0),
        "Share Layer",
    );
    let current_layer = doc.get_page(page).get_layer(layer);

    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|err| {
            format!(
                "{}: {err}",
                t("Không tải được font PDF", "Could not load PDF font")
            )
        })?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|err| {
            format!(
                "{}: {err}",
                t("Không tải được font PDF", "Could not load PDF font")
            )
        })?;

    current_layer.use_text(
        "Bitcoin Wallet - SLIP-0039 Share",
        18.0,
        Mm(18.0),
        Mm(280.0),
        &font_bold,
    );
    current_layer.use_text(
        format!("Wallet: {wallet_name}"),
        12.0,
        Mm(18.0),
        Mm(268.0),
        &font_regular,
    );
    current_layer.use_text(
        format!("Network: {network}"),
        12.0,
        Mm(18.0),
        Mm(260.0),
        &font_regular,
    );
    current_layer.use_text(
        format!("Scheme: {threshold}-of-{share_count}"),
        12.0,
        Mm(18.0),
        Mm(252.0),
        &font_regular,
    );
    current_layer.use_text(
        format!("Share: {share_index}/{share_total}"),
        12.0,
        Mm(18.0),
        Mm(244.0),
        &font_regular,
    );
    current_layer.use_text(
        format!(
            "SLIP39 passphrase: {}",
            if has_slip39_passphrase {
                "SET (required for restore)"
            } else {
                "EMPTY"
            }
        ),
        11.0,
        Mm(18.0),
        Mm(236.0),
        &font_regular,
    );
    current_layer.use_text(
        "Keep this PDF offline. Whoever has enough shares can recover your wallet.",
        10.0,
        Mm(18.0),
        Mm(228.0),
        &font_regular,
    );

    let words: Vec<&str> = share_phrase.split_whitespace().collect();
    for (idx, word) in words.iter().enumerate() {
        let row = idx / 2;
        let col = idx % 2;
        let x = if col == 0 { 18.0 } else { 110.0 };
        let y = 214.0 - (row as f32 * 10.0);

        current_layer.use_text(
            format!("{:02}. {}", idx + 1, word),
            12.0,
            Mm(x),
            Mm(y),
            &font_regular,
        );
    }

    let file = File::create(path).map_err(|err| {
        format!(
            "{} {}: {err}",
            t("Không tạo được file PDF", "Could not create PDF file"),
            path.display()
        )
    })?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer).map_err(|err| {
        format!(
            "{}: {err}",
            t("Không ghi được nội dung PDF", "Could not write PDF content")
        )
    })?;

    Ok(())
}

fn normalize_nickname(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
