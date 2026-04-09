use std::time::Duration;

use iced::{clipboard, Task};

use crate::app::structure::*;
use crate::core::wallet::{Wallet, WalletSecretsRef, WalletSecretsVault};
use crate::error::AppError;
use crate::i18n::t;
use crate::infra::storage::Storage;
use crate::ui::components::language_selector::LanguageSelector;
use crate::ui::components::{Toast, ToastManager};
use crate::ui::views::{
    dashboard::{DashboardMessage, DashboardView},
    history::{HistoryEvent, HistoryMessage, HistoryView},
    login::{LoginMessage, LoginView},
    onboarding::{OnboardingMessage, OnboardingView},
    receive::{ReceiveMessage, ReceiveView},
    send::{SendMessage, SendView},
    settings::{SettingsMessage, SettingsView},
    sidebar::{NavItem, Sidebar, SidebarEvent, SidebarMessage},
    wallets::{WalletsMessage, WalletsView},
};

impl App {
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
                            self.settings_view.esplora_endpoint =
                                if let Ok(storage) = Storage::new() {
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
                        self.error = Some(AppError::unknown(&format!(
                            "{}: {}",
                            t("Không thể mở trình duyệt", "Cannot open browser"),
                            e
                        )));
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
                                                        crate::core::wallet::TxDirection::Incoming => "IN",
                                                        crate::core::wallet::TxDirection::Outgoing => "OUT",
                                                        crate::core::wallet::TxDirection::SelfTransfer => "SELF",
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
                                                    .set_title(t(
                                                        "Lưu lịch sử PDF",
                                                        "Save history as PDF",
                                                    ))
                                                    .add_filter(t("File PDF", "PDF file"), &["pdf"])
                                                    .set_file_name("history_export.pdf")
                                                    .save_file()
                                            })
                                            .await
                                            .unwrap_or(None)
                                            .and_then(
                                                |path| {
                                                    use printpdf::{BuiltinFont, Mm, PdfDocument};
                                                    let (doc, page, layer) = PdfDocument::new(
                                                        "Transaction History",
                                                        Mm(210.0),
                                                        Mm(297.0),
                                                        "History Layer",
                                                    );
                                                    let current_layer =
                                                        doc.get_page(page).get_layer(layer);
                                                    let font_regular = doc
                                                        .add_builtin_font(BuiltinFont::Helvetica)
                                                        .ok()?;
                                                    let font_bold = doc
                                                        .add_builtin_font(
                                                            BuiltinFont::HelveticaBold,
                                                        )
                                                        .ok()?;
                                                    current_layer.use_text(
                                                        format!(
                                                            "Bitcoin Wallet - {}",
                                                            wallet_clone.name
                                                        ),
                                                        18.0,
                                                        Mm(15.0),
                                                        Mm(280.0),
                                                        &font_bold,
                                                    );
                                                    let mut y_pos = 260.0;
                                                    for tx in &wallet_clone.history {
                                                        if y_pos < 15.0 {
                                                            break;
                                                        }
                                                        let date_str =
                                                            if let Some(ts) = tx.block_time {
                                                                let dt =
                                                                chrono::DateTime::from_timestamp(
                                                                    ts as i64, 0,
                                                                )
                                                                .unwrap_or_default();
                                                                dt.format("%d/%m/%Y").to_string()
                                                            } else {
                                                                "Pending".to_string()
                                                            };
                                                        let amount_btc =
                                                            tx.amount_sat as f64 / 100_000_000.0;
                                                        current_layer.use_text(
                                                            date_str,
                                                            8.0,
                                                            Mm(15.0),
                                                            Mm(y_pos),
                                                            &font_regular,
                                                        );
                                                        current_layer.use_text(
                                                            format!("{:.8}", amount_btc),
                                                            8.0,
                                                            Mm(65.0),
                                                            Mm(y_pos),
                                                            &font_regular,
                                                        );
                                                        current_layer.use_text(
                                                            &tx.txid[..16.min(tx.txid.len())],
                                                            8.0,
                                                            Mm(115.0),
                                                            Mm(y_pos),
                                                            &font_regular,
                                                        );
                                                        y_pos -= 10.0;
                                                    }
                                                    if let Ok(file) = std::fs::File::create(&path) {
                                                        let mut writer =
                                                            std::io::BufWriter::new(file);
                                                        doc.save(&mut writer).ok()
                                                    } else {
                                                        None
                                                    }
                                                },
                                            )
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
                    self.wallets_view
                        .update(WalletsMessage::DismissWalletNotice);
                } else if self.settings_view.show_clear_data_confirm {
                    self.settings_view
                        .update(SettingsMessage::ToggleClearDataConfirm);
                }
                Task::none()
            }
            AppMessage::ExportFinished(result) => {
                match result {
                    Ok(_) => {
                        self.add_success_toast(
                            t("Xuất file thành công!", "Export successful!").to_string(),
                        );
                    }
                    Err(e) => {
                        self.error = Some(AppError::storage(
                            "export",
                            &format!("{}: {}", t("Lỗi export", "Export error"), e),
                        ));
                    }
                }
                Task::none()
            }
            AppMessage::DismissStatus => Task::none(),
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
                            self.last_copied_time =
                                Some(chrono::Local::now().format("%H:%M:%S").to_string());
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

            AppMessage::AutoRefreshConfirmations => {
                // Only auto-refresh if auto_refresh is enabled AND we have pending transactions
                if let Ok(storage) = Storage::new() {
                    if !storage.load_auto_refresh().unwrap_or(false) {
                        return Task::none();
                    }
                }
                let has_pending = self.wallets.iter().any(|w| {
                    w.history
                        .iter()
                        .any(|tx| !tx.confirmed || tx.confirmations < 6)
                });
                if has_pending {
                    return self.refresh_all_wallets();
                }
                Task::none()
            }
            AppMessage::OnboardingMessage(msg) => {
                use crate::ui::views::onboarding::OnboardingEvent;
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
        }
    }
}
