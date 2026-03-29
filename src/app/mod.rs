mod login;
mod send;
mod settings;
mod wallet;

use std::{
    env,
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use iced::{
    widget::{column, container, row, text, Space},
    Element, Length, Task,
};
use printpdf::{BuiltinFont, Mm, PdfDocument};

use crate::i18n::{set_current_language, t, AppLanguage};
use crate::storage::{PersistedState, Storage, UserProfile};
use crate::views::{
    dashboard::{DashboardMessage, DashboardView},
    history::{HistoryMessage, HistoryView, HistoryEvent},
    login::{LoginMessage, LoginMode, LoginView},
    receive::{ReceiveMessage, ReceiveView},
    send::{SendMessage, SendView},
    settings::{SettingsMessage, SettingsView},
    sidebar::{NavItem, Sidebar, SidebarMessage, SidebarEvent},
    wallets::{WalletsMessage, WalletsView},
};
use crate::wallet::{
    ChangeStrategy, FeeMode, InputSource, Wallet, WalletEntry,
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
    pub state: AppState,
    pub storage_passphrase: Option<String>,
    pub language: AppLanguage,
    pub user_nickname: Option<String>,
    pub wallets: Vec<WalletEntry>,
    pub selected_wallet: usize,

    pub login_view: LoginView,
    pub sidebar: Sidebar,
    pub dashboard: DashboardView,
    pub wallets_view: WalletsView,
    pub send_view: SendView,
    pub receive_view: ReceiveView,
    pub history_view: HistoryView,
    pub settings_view: SettingsView,

    pub current_page: NavItem,
    pub status: Option<String>,
    pub error: Option<String>,
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

            AppMessage::DashboardMessage(msg) => {
                match msg {
                    DashboardMessage::Refresh => self.refresh_all_wallets(),
                }
                Task::none()
            }

            AppMessage::WalletsMessage(msg) => self.handle_wallets_message(msg),
            AppMessage::SendMessage(msg) => self.handle_send_message(msg),
            AppMessage::ReceiveMessage(msg) => self.handle_receive_message(msg),

            AppMessage::HistoryMessage(msg) => {
                if let Some(event) = self.history_view.update(msg) {
                    match event {
                        HistoryEvent::Refresh => self.refresh_all_wallets(),
                    }
                }
                Task::none()
            }

            AppMessage::SettingsMessage(msg) => self.handle_settings_message(msg),
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

    pub fn update_dashboard(&mut self) {
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

    pub fn refresh_all_wallets(&mut self) {
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

    pub fn reset_to_login(&mut self, allow_create_passphrase: bool) {
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

    pub fn display_name(&self) -> &str {
        self.user_nickname.as_deref().unwrap_or(t("bạn", "friend"))
    }
}

pub fn short_txid(txid: &str) -> String {
    let prefix = txid.get(..12).unwrap_or(txid);
    format!("{prefix}...")
}

pub fn wallet_count_text(count: usize) -> String {
    format!("{} {}", count, t("ví", "wallet(s)"))
}

pub fn address_count_text(count: usize) -> String {
    format!("{} {}", count, t("địa chỉ mới", "new address(es)"))
}

pub fn resolve_user_path(raw_path: &str) -> PathBuf {
    let trimmed = raw_path.trim();
    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return Path::new(&home).join(rest);
        }
    }

    PathBuf::from(trimmed)
}

pub fn pick_import_backup_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t(
            "Chọn file backup để import",
            "Choose backup file to import",
        ))
        .add_filter(t("File backup", "Backup files"), &["enc", "json"])
        .pick_file()
}

pub fn pick_export_backup_path(current_path: &str) -> Option<PathBuf> {
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

pub fn pick_mnemonic_pdf_path(default_file_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t("Lưu mnemonic ra PDF", "Save mnemonic as PDF"))
        .add_filter(t("File PDF", "PDF file"), &["pdf"])
        .set_file_name(default_file_name)
        .save_file()
}

pub fn pick_slip39_export_directory() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t(
            "Chọn thư mục chứa backup SLIP-0039",
            "Choose folder for SLIP-0039 backup",
        ))
        .pick_folder()
}

pub fn default_mnemonic_pdf_filename(wallet_name: &str) -> String {
    format!("{}_mnemonic_backup.pdf", sanitize_filename(wallet_name))
}

pub fn default_slip39_directory_name(wallet_name: &str, threshold: u8, share_count: u8) -> String {
    format!(
        "{}_slip39_{}of{}",
        sanitize_filename(wallet_name),
        threshold,
        share_count
    )
}

pub fn sanitize_filename(raw: &str) -> String {
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

pub fn ensure_pdf_extension(mut path: PathBuf) -> PathBuf {
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

pub fn export_mnemonic_to_pdf(
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

pub fn export_slip39_shares_to_pdf_directory(
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

pub fn normalize_nickname(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}