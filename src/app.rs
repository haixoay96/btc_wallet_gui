use iced::{
    widget::{column, container, row, text, Space},
    Element, Length, Subscription, Task, Theme,
};
use anyhow::Result;

use crate::storage::{Storage, PersistedState};
use crate::wallet::{Wallet, WalletEntry, WalletNetwork};
use crate::views::{
    login::{LoginView, LoginMessage},
    sidebar::{Sidebar, SidebarMessage, NavItem},
    dashboard::{DashboardView, DashboardMessage},
    wallets::{WalletsView, WalletsMessage},
    send::{SendView, SendMessage},
    receive::{ReceiveView, ReceiveMessage},
    history::{HistoryView, HistoryMessage},
    settings::{SettingsView, SettingsMessage},
};

#[derive(Debug, Clone)]
pub enum AppState {
    Login,
    Main,
}

pub struct App {
    state: AppState,
    storage_passphrase: Option<String>,
    wallets: Vec<WalletEntry>,
    selected_wallet: usize,
    
    // Views
    login_view: LoginView,
    sidebar: Sidebar,
    dashboard: DashboardView,
    wallets_view: WalletsView,
    send_view: SendView,
    receive_view: ReceiveView,
    history_view: HistoryView,
    settings_view: SettingsView,
    
    // Current page
    current_page: NavItem,
    
    // Status messages
    status: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    // Login
    Login(String),
    LoginMessage(LoginMessage),
    
    // Navigation
    Navigate(NavItem),
    SidebarMessage(SidebarMessage),
    DashboardMessage(DashboardMessage),
    WalletsMessage(WalletsMessage),
    SendMessage(SendMessage),
    ReceiveMessage(ReceiveMessage),
    HistoryMessage(HistoryMessage),
    SettingsMessage(SettingsMessage),
    
    // Wallet operations
    CreateWallet(String, WalletNetwork),
    DeleteWallet(usize),
    SelectWallet(usize),
    RefreshHistory,
    DeriveAddresses(u32),
    
    // System
    Loaded(Result<PersistedState, String>),
    Saved(Result<(), String>),
    Error(String),
}

impl App {
    pub fn new() -> (Self, Task<AppMessage>) {
        let app = Self {
            state: AppState::Login,
            storage_passphrase: None,
            wallets: Vec::new(),
            selected_wallet: 0,
            login_view: LoginView::new(),
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
        };
        
        (app, Task::none())
    }

    pub fn title(&self) -> String {
        "Bitcoin Wallet - Exodus Style".to_string()
    }

    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::Login(passphrase) => {
                self.storage_passphrase = Some(passphrase.clone());
                
                // Load state synchronously - always succeeds (creates empty state if needed)
                match Storage::new() {
                    Ok(storage) => {
                        match storage.load_state(&passphrase) {
                            Ok(state) => {
                                self.wallets = state.wallets;
                                self.state = AppState::Main;
                                self.update_dashboard();
                                if self.wallets.is_empty() {
                                    self.status = Some("Welcome! Create your first wallet.".to_string());
                                } else {
                                    self.status = Some(format!("Loaded {} wallet(s)", self.wallets.len()));
                                }
                            }
                            Err(e) => {
                                // Even if there's an error, allow login with empty state
                                // This handles first-time users
                                self.wallets = Vec::new();
                                self.state = AppState::Main;
                                self.update_dashboard();
                                self.status = Some("Welcome! Create your first wallet.".to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Error initializing storage: {}", e));
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
            
            AppMessage::Loaded(result) => {
                match result {
                    Ok(state) => {
                        self.wallets = state.wallets;
                        self.state = AppState::Main;
                        self.update_dashboard();
                        self.status = Some("Wallet loaded successfully".to_string());
                    }
                    Err(e) => {
                        self.error = Some(format!("Error loading wallet: {}", e));
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
                let app_msg = self.sidebar.update(msg);
                self.update(app_msg)
            }
            
            AppMessage::DashboardMessage(msg) => {
                match msg {
                    DashboardMessage::Refresh => {
                        self.refresh_all_wallets();
                    }
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
                let wallet = self.wallets.get(self.selected_wallet);
                if let Some(_app_msg) = self.send_view.update(msg) {
                    // TODO: Handle send transaction
                }
                Task::none()
            }
            
            AppMessage::ReceiveMessage(msg) => {
                let wallet = self.wallets.get(self.selected_wallet);
                if let Some(_app_msg) = self.receive_view.update(msg) {
                    // TODO: Handle derive addresses
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
                if let Some(_app_msg) = self.settings_view.update(msg) {
                    // TODO: Handle settings
                }
                Task::none()
            }
            
            AppMessage::CreateWallet(name, network) => {
                match Wallet::new(&name, network) {
                    Ok(wallet) => {
                        self.wallets.push(wallet.entry);
                        self.save_state();
                        self.update_dashboard();
                        self.status = Some(format!("Created wallet '{}'", name));
                    }
                    Err(e) => {
                        self.error = Some(format!("Error creating wallet: {}", e));
                    }
                }
                Task::none()
            }
            
            AppMessage::SelectWallet(index) => {
                if index < self.wallets.len() {
                    self.selected_wallet = index;
                    self.status = Some(format!("Selected wallet: {}", self.wallets[index].name));
                }
                Task::none()
            }
            
            AppMessage::DeleteWallet(index) => {
                if index < self.wallets.len() {
                    let name = self.wallets[index].name.clone();
                    self.wallets.remove(index);
                    
                    if self.selected_wallet >= self.wallets.len() && !self.wallets.is_empty() {
                        self.selected_wallet = self.wallets.len() - 1;
                    }
                    
                    self.save_state();
                    self.update_dashboard();
                    self.status = Some(format!("Deleted wallet '{}'", name));
                }
                Task::none()
            }
            
            AppMessage::RefreshHistory => {
                self.refresh_all_wallets();
                Task::none()
            }
            
            AppMessage::DeriveAddresses(count) => {
                if let Some(wallet_entry) = self.wallets.get_mut(self.selected_wallet) {
                    let mut wallet = Wallet { entry: wallet_entry.clone() };
                    if let Ok(addrs) = wallet.derive_next_addresses(count) {
                        *wallet_entry = wallet.entry;
                        self.save_state();
                        self.status = Some(format!("Derived {} new address(es)", addrs.len()));
                    }
                }
                Task::none()
            }
            
            AppMessage::Saved(result) => {
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        self.error = Some(format!("Error saving: {}", e));
                    }
                }
                Task::none()
            }
            
            AppMessage::Error(msg) => {
                self.error = Some(msg);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<AppMessage> {
        match self.state {
            AppState::Login => {
                container(self.login_view.view().map(AppMessage::LoginMessage))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            }
            AppState::Main => {
                let sidebar = self.sidebar.view().map(AppMessage::SidebarMessage);
                
                let selected_wallet = self.wallets.get(self.selected_wallet);
                
                let main_content = match self.current_page {
                    NavItem::Dashboard => {
                        self.dashboard.view().map(AppMessage::DashboardMessage)
                    }
                    NavItem::Wallets => {
                        self.wallets_view.view(&self.wallets, self.selected_wallet)
                            .map(AppMessage::WalletsMessage)
                    }
                    NavItem::Send => {
                        self.send_view.view(selected_wallet)
                            .map(AppMessage::SendMessage)
                    }
                    NavItem::Receive => {
                        self.receive_view.view(selected_wallet)
                            .map(AppMessage::ReceiveMessage)
                    }
                    NavItem::History => {
                        self.history_view.view(selected_wallet)
                            .map(AppMessage::HistoryMessage)
                    }
                    NavItem::Settings => {
                        self.settings_view.view()
                            .map(AppMessage::SettingsMessage)
                    }
                };
                
                let status_bar = if let Some(status) = &self.status {
                    container(
                        text(status.as_str())
                            .size(12)
                            .style(crate::theme::text_color(crate::theme::Colors::SUCCESS))
                    )
                    .padding(8)
                } else {
                    container(Space::with_height(0))
                };
                
                let error_bar = if let Some(error) = &self.error {
                    container(
                        text(error.as_str())
                            .size(12)
                            .style(crate::theme::text_color(crate::theme::Colors::ERROR))
                    )
                    .padding(8)
                } else {
                    container(Space::with_height(0))
                };
                
                row![
                    sidebar,
                    column![
                        status_bar,
                        error_bar,
                        main_content,
                    ]
                    .width(Length::Fill)
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        }
    }

    fn update_dashboard(&mut self) {
        let total: i64 = self.wallets.iter().map(|w| {
            w.history.iter().map(|tx| tx.amount_sat).sum::<i64>()
        }).sum();
        
        let confirmed: i64 = self.wallets.iter().map(|w| {
            w.history.iter()
                .filter(|tx| tx.confirmed)
                .map(|tx| tx.amount_sat)
                .sum::<i64>()
        }).sum();
        
        self.dashboard.update_balances(total, confirmed, self.wallets.len());
    }

    fn refresh_all_wallets(&mut self) {
        for wallet_entry in &mut self.wallets {
            let mut wallet = Wallet { entry: wallet_entry.clone() };
            if let Ok(count) = wallet.refresh_history() {
                *wallet_entry = wallet.entry;
                self.status = Some(format!("Refreshed {} transactions", count));
            }
        }
        self.save_state();
        self.update_dashboard();
    }

    fn save_state(&self) {
        let passphrase = match &self.storage_passphrase {
            Some(p) => p.clone(),
            None => return,
        };
        
        let wallets = self.wallets.clone();
        
        tokio::spawn(async move {
            if let Ok(storage) = Storage::new() {
                let state = PersistedState { wallets };
                if let Err(e) = storage.save_state(&state, &passphrase) {
                    eprintln!("Error saving state: {}", e);
                }
            }
        });
    }
}