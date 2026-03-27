use std::{
    env,
    path::{Path, PathBuf},
};

use iced::{
    clipboard,
    widget::{column, container, row, text, Space},
    Element, Length, Task,
};

use crate::storage::{PersistedState, Storage};
use crate::views::{
    dashboard::{DashboardMessage, DashboardView},
    history::{HistoryMessage, HistoryView},
    login::{LoginMessage, LoginMode, LoginView},
    receive::{ReceiveMessage, ReceiveView},
    send::{SendMessage, SendView},
    settings::{SettingsMessage, SettingsView},
    sidebar::{NavItem, Sidebar, SidebarMessage},
    wallets::{WalletsMessage, WalletsView},
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
    Login(String),
    LoginMessage(LoginMessage),

    Navigate(NavItem),
    SidebarMessage(SidebarMessage),
    DashboardMessage(DashboardMessage),
    WalletsMessage(WalletsMessage),
    SendMessage(SendMessage),
    ReceiveMessage(ReceiveMessage),
    HistoryMessage(HistoryMessage),
    SettingsMessage(SettingsMessage),

    CreateWallet(String, WalletNetwork),
    DeleteWallet(usize),
    SelectWallet(usize),
    RefreshHistory,
    DeriveAddresses(u32),

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
    ImportWalletBackup(String),
    ClearAllData,
}

impl App {
    pub fn new() -> (Self, Task<AppMessage>) {
        let has_existing_state = Storage::new()
            .map(|storage| storage.has_existing_state())
            .unwrap_or(false);
        let mut login_view = LoginView::new();
        login_view.set_can_create_new_passphrase(!has_existing_state);
        if !has_existing_state {
            login_view.set_mode(LoginMode::NewWallet);
        }

        (
            Self {
                state: AppState::Login,
                storage_passphrase: None,
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
        "Bitcoin Wallet - Exodus Style".to_string()
    }

    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::Login(passphrase) => {
                self.status = None;
                self.error = None;

                match Storage::new() {
                    Ok(storage) => {
                        let had_existing_state = storage.has_existing_state();
                        match storage.load_state(&passphrase) {
                            Ok(state) => {
                                self.storage_passphrase = Some(passphrase);
                                self.wallets = state.wallets;
                                self.state = AppState::Main;
                                self.selected_wallet =
                                    self.selected_wallet.min(self.wallets.len().saturating_sub(1));
                                self.update_dashboard();
                                self.login_view.clear_error();

                                if had_existing_state {
                                    self.status =
                                        Some(format!("Loaded {} wallet(s)", self.wallets.len()));
                                } else {
                                    self.status =
                                        Some("Welcome! Create your first wallet.".to_string());
                                }
                            }
                            Err(err) => {
                                let message = format!("Đăng nhập thất bại: {err}");
                                self.error = Some(message.clone());
                                self.login_view.set_error(message);
                            }
                        }
                    }
                    Err(err) => {
                        self.error = Some(format!("Error initializing storage: {err}"));
                    }
                }

                Task::none()
            }

            AppMessage::LoginMessage(msg) => {
                if let Some(app_msg) = self.login_view.update(msg) {
                    return self.update(app_msg);
                }
                Task::none()
            }

            AppMessage::Navigate(page) => {
                self.current_page = page;
                self.sidebar.set_active(page);
                Task::none()
            }

            AppMessage::SidebarMessage(msg) => {
                let app_msg = self.sidebar.update(msg);
                self.update(app_msg)
            }

            AppMessage::DashboardMessage(msg) => {
                match msg {
                    DashboardMessage::Refresh => self.refresh_all_wallets(),
                }
                Task::none()
            }

            AppMessage::WalletsMessage(msg) => {
                if let Some(app_msg) = self.wallets_view.update(msg) {
                    return self.update(app_msg);
                }
                Task::none()
            }

            AppMessage::SendMessage(msg) => {
                if let Some(app_msg) = self.send_view.update(msg) {
                    return self.update(app_msg);
                }
                Task::none()
            }

            AppMessage::ReceiveMessage(msg) => {
                if let Some(app_msg) = self.receive_view.update(msg) {
                    return self.update(app_msg);
                }
                Task::none()
            }

            AppMessage::HistoryMessage(msg) => {
                if let Some(app_msg) = self.history_view.update(msg) {
                    return self.update(app_msg);
                }
                Task::none()
            }

            AppMessage::SettingsMessage(msg) => {
                if let Some(app_msg) = self.settings_view.update(msg) {
                    return self.update(app_msg);
                }
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
                        self.status = Some(format!("Created wallet '{name}' successfully"));
                        self.error = None;
                    }
                    Err(err) => {
                        self.error = Some(format!("Error creating wallet: {err}"));
                    }
                }
                Task::none()
            }

            AppMessage::SelectWallet(index) => {
                if index < self.wallets.len() {
                    self.selected_wallet = index;
                    self.status = Some(format!("Selected wallet: {}", self.wallets[index].name));
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
                    self.status = Some(format!("Deleted wallet '{name}'"));
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
                            self.status = Some(format!("Derived {} new address(es)", addresses.len()));
                            self.error = None;
                        }
                        Err(err) => {
                            self.error = Some(format!("Không thể derive địa chỉ: {err}"));
                        }
                    }
                } else {
                    self.error = Some("No wallet selected".to_string());
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
                            self.status = Some(format!("Estimated fee: {fee} sat"));
                            self.error = None;
                        }
                        Err(err) => {
                            self.send_view.set_error(err.to_string());
                            self.error = Some(format!("Estimate fee failed: {err}"));
                        }
                    }
                } else {
                    self.send_view.set_error("No wallet selected");
                    self.error = Some("No wallet selected".to_string());
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
                                self.send_view
                                    .set_error("Amount không hợp lệ cho giao dịch thường");
                                return Task::none();
                            }
                        };

                        let fee_sat = match request.fee_mode {
                            FeeMode::Auto => match wallet
                                .estimate_auto_fee_for_amount(amount_sat, &request.input_source)
                            {
                                Ok(value) => value,
                                Err(err) => {
                                    self.send_view
                                        .set_error(format!("Không thể estimate fee tự động: {err}"));
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
                                format!("Transaction broadcasted: {short_txid}")
                            } else {
                                format!("Transaction created (not broadcast): {short_txid}")
                            };
                            self.send_view.set_success(send_message.clone());
                            self.status = Some(send_message);
                            self.error = None;
                        }
                        Err(err) => {
                            self.send_view.set_error(err.to_string());
                            self.error = Some(format!("Send failed: {err}"));
                        }
                    }
                } else {
                    self.send_view.set_error("No wallet selected");
                    self.error = Some("No wallet selected".to_string());
                }

                Task::none()
            }

            AppMessage::CopyAddress(address) => {
                self.status = Some("Copied address to clipboard".to_string());
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
                        self.settings_view
                            .set_error("Không có session đăng nhập hợp lệ");
                        return Task::none();
                    }
                };

                if current != active_passphrase {
                    self.settings_view
                        .set_error("Passphrase hiện tại không đúng");
                    return Task::none();
                }

                match Storage::new() {
                    Ok(storage) => match storage.rotate_passphrase(&current, &new_passphrase) {
                        Ok(_) => {
                            self.storage_passphrase = Some(new_passphrase);
                            self.settings_view.clear_sensitive_inputs();
                            self.settings_view
                                .set_success("Đổi passphrase thành công");
                            self.status = Some("Passphrase updated successfully".to_string());
                            self.error = None;
                        }
                        Err(err) => {
                            self.settings_view
                                .set_error(format!("Đổi passphrase thất bại: {err}"));
                        }
                    },
                    Err(err) => {
                        self.settings_view
                            .set_error(format!("Không thể mở storage: {err}"));
                    }
                }
                Task::none()
            }

            AppMessage::ExportWalletBackup(raw_path) => {
                let passphrase = match &self.storage_passphrase {
                    Some(value) => value.clone(),
                    None => {
                        self.settings_view
                            .set_error("Không có session đăng nhập hợp lệ");
                        return Task::none();
                    }
                };

                let export_path = resolve_user_path(&raw_path);
                let state = PersistedState {
                    wallets: self.wallets.clone(),
                };

                match Storage::new() {
                    Ok(storage) => {
                        match storage.export_encrypted_backup(&state, &passphrase, &export_path) {
                            Ok(_) => {
                                let message =
                                    format!("Exported encrypted backup to {}", export_path.display());
                                self.settings_view.set_success(message.clone());
                                self.status = Some(message);
                                self.error = None;
                            }
                            Err(err) => {
                                self.settings_view
                                    .set_error(format!("Export thất bại: {err}"));
                            }
                        }
                    }
                    Err(err) => {
                        self.settings_view
                            .set_error(format!("Không thể mở storage: {err}"));
                    }
                }

                Task::none()
            }

            AppMessage::ImportWalletBackup(raw_path) => {
                let passphrase = match &self.storage_passphrase {
                    Some(value) => value.clone(),
                    None => {
                        self.settings_view
                            .set_error("Không có session đăng nhập hợp lệ");
                        return Task::none();
                    }
                };

                let import_path = resolve_user_path(&raw_path);

                match Storage::new() {
                    Ok(storage) => match storage.import_backup(&import_path, &passphrase) {
                        Ok(state) => {
                            self.wallets = state.wallets;
                            self.selected_wallet =
                                self.selected_wallet.min(self.wallets.len().saturating_sub(1));
                            self.save_state();
                            self.update_dashboard();
                            let message = format!(
                                "Imported {} wallet(s) from {}",
                                self.wallets.len(),
                                import_path.display()
                            );
                            self.settings_view.set_success(message.clone());
                            self.status = Some(message);
                            self.error = None;
                        }
                        Err(err) => {
                            self.settings_view
                                .set_error(format!("Import thất bại: {err}"));
                        }
                    },
                    Err(err) => {
                        self.settings_view
                            .set_error(format!("Không thể mở storage: {err}"));
                    }
                }

                Task::none()
            }

            AppMessage::ClearAllData => {
                match Storage::new() {
                    Ok(storage) => match storage.clear_all_data() {
                        Ok(_) => {
                            self.reset_to_login(true);
                            self.login_view
                                .set_mode(LoginMode::NewWallet);
                            self.login_view
                                .set_error("Đã xóa toàn bộ dữ liệu cũ. Hãy tạo passphrase mới.");
                        }
                        Err(err) => {
                            self.settings_view
                                .set_error(format!("Không thể xóa dữ liệu: {err}"));
                        }
                    },
                    Err(err) => {
                        self.settings_view
                            .set_error(format!("Không thể mở storage: {err}"));
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
                    NavItem::Send => self.send_view.view(selected_wallet).map(AppMessage::SendMessage),
                    NavItem::Receive => self
                        .receive_view
                        .view(selected_wallet)
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

                row![
                    sidebar,
                    column![status_bar, error_bar, main_content,].width(Length::Fill)
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
            self.status = Some("Không có ví để refresh".to_string());
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
            "Refreshed {refreshed_wallets} wallet(s), {refreshed_txs} transaction(s)"
        ));

        if !errors.is_empty() {
            self.error = Some(format!("Một số ví refresh lỗi: {}", errors.join(" | ")));
        } else {
            self.error = None;
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
                    wallets: self.wallets.clone(),
                };
                if let Err(err) = storage.save_state(&state, &passphrase) {
                    self.error = Some(format!("Error saving state: {err}"));
                }
            }
            Err(err) => {
                self.error = Some(format!("Error initializing storage: {err}"));
            }
        }
    }

    fn reset_to_login(&mut self, allow_create_passphrase: bool) {
        self.state = AppState::Login;
        self.storage_passphrase = None;
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
}

fn short_txid(txid: &str) -> String {
    let prefix = txid.get(..12).unwrap_or(txid);
    format!("{prefix}...")
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
