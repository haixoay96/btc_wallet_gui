use anyhow::anyhow;
use iced::{clipboard, Task};
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroize;

use crate::core::wallet::{Wallet, WalletNetwork, WalletSecretsRef};
use crate::infra::storage::Storage;
use crate::ui::i18n::t;
use crate::ui::views::receive::{ReceiveEvent, ReceiveMessage};
use crate::ui::views::wallets::{WalletsEvent, WalletsMessage, WalletsView};
use crate::utils::{
    address_count_text, default_mnemonic_encrypted_filename, default_mnemonic_pdf_filename,
    default_slip39_directory_name, ensure_enc_extension, ensure_pdf_extension,
    export_mnemonic_to_encrypted_file, export_mnemonic_to_pdf,
    export_slip39_shares_to_pdf_directory, load_encrypted_secret_export,
    pick_encrypted_export_path, pick_encrypted_secret_import_path, pick_mnemonic_pdf_path,
    pick_slip39_export_directory, resolve_user_path, DecryptedSecretExport, Slip39PdfExport,
};

use super::{App, AppMessage};

impl App {
    pub fn handle_wallets_message(&mut self, msg: WalletsMessage) -> Task<AppMessage> {
        if let Some(event) = self.wallets_view.update(msg) {
            match event {
                WalletsEvent::CreateWallet(name, network) => {
                    return self.handle_create_wallet(name, network);
                }
                WalletsEvent::ImportWalletFromMnemonic {
                    name,
                    network,
                    mnemonic,
                } => {
                    let task = self.handle_import_wallet_from_mnemonic(name, network, mnemonic);
                    self.wallets_view.clear_import_sensitive_inputs();
                    return task;
                }
                WalletsEvent::ImportWalletFromSlip39 {
                    name,
                    network,
                    shares,
                    slip39_passphrase,
                } => {
                    let task = self.handle_import_wallet_from_slip39(
                        name,
                        network,
                        shares,
                        slip39_passphrase,
                    );
                    self.wallets_view.clear_import_sensitive_inputs();
                    return task;
                }
                WalletsEvent::BrowseImportEncryptedPath => {
                    if let Some(path) = pick_encrypted_secret_import_path() {
                        self.wallets_view
                            .set_import_encrypted_path(path.to_string_lossy().to_string());
                    }
                    return Task::none();
                }
                WalletsEvent::ImportWalletFromEncrypted {
                    path,
                    passphrase,
                    name_override,
                } => {
                    let task =
                        self.handle_import_wallet_from_encrypted(path, passphrase, name_override);
                    self.wallets_view.clear_import_sensitive_inputs();
                    return task;
                }
                WalletsEvent::SelectWallet(index) => {
                    return self.handle_select_wallet(index);
                }
                WalletsEvent::DeleteWallet(index) => {
                    return self.handle_delete_wallet(index);
                }
                WalletsEvent::RevealMnemonic {
                    wallet_index,
                    passphrase,
                } => {
                    let task = self.handle_reveal_mnemonic(wallet_index, passphrase);
                    self.wallets_view.clear_reveal_sensitive_inputs();
                    return task;
                }
                WalletsEvent::VerifyMnemonicBackup {
                    wallet_index,
                    checks,
                } => {
                    let task = self.handle_verify_mnemonic_backup(wallet_index, checks);
                    self.wallets_view.clear_backup_test_inputs();
                    return task;
                }
                WalletsEvent::ExportMnemonicPdf(index) => {
                    return self.handle_export_mnemonic_pdf(index);
                }
                WalletsEvent::ExportMnemonicEncrypted(index) => {
                    return self.handle_export_mnemonic_encrypted(index);
                }
                WalletsEvent::ExportWalletSlip39 {
                    wallet_index,
                    threshold,
                    share_count,
                    slip39_passphrase,
                } => {
                    let task = self.handle_export_wallet_slip39(
                        wallet_index,
                        threshold,
                        share_count,
                        slip39_passphrase,
                    );
                    self.wallets_view.clear_export_sensitive_inputs();
                    return task;
                }
                WalletsEvent::CopyAddress(address) => {
                    return clipboard::write(address);
                }
                WalletsEvent::SortFieldChanged(field) => {
                    if let Ok(storage) = Storage::new() {
                        let _ = storage.save_wallet_sort_field(field);
                    }
                }
                WalletsEvent::ToggleSortDirection => {
                    if let Ok(storage) = Storage::new() {
                        let current = storage.load_wallet_sort_ascending().unwrap_or(false);
                        let _ = storage.save_wallet_sort_ascending(!current);
                    }
                }
                WalletsEvent::TagInputChanged(tag) => {
                    let tag_trimmed = tag.trim().to_string();
                    if !tag_trimmed.is_empty() {
                        // Dùng ví đang mở modal, fallback về ví đang chọn nếu không có modal
                        let target_index = self
                            .wallets_view
                            .tag_modal_index
                            .unwrap_or(self.selected_wallet);
                        if let Some(wallet) = self.wallets.get_mut(target_index) {
                            if !wallet
                                .tags
                                .iter()
                                .any(|t| t.eq_ignore_ascii_case(&tag_trimmed))
                            {
                                wallet.tags.push(tag_trimmed.clone());
                                tracing::info!(
                                    wallet = wallet.name,
                                    tag = tag_trimmed,
                                    "Tag added"
                                );
                                self.save_state(); // Persist immediately
                            }
                        }
                    }
                    self.wallets_view.tag_input.clear();
                }
                WalletsEvent::RemoveTag(tag) => {
                    // Dùng ví đang mở modal, fallback về ví đang chọn nếu không có modal
                    let target_index = self
                        .wallets_view
                        .tag_modal_index
                        .unwrap_or(self.selected_wallet);
                    if let Some(wallet) = self.wallets.get_mut(target_index) {
                        wallet.tags.retain(|t| !t.eq_ignore_ascii_case(&tag));
                        tracing::info!(wallet = wallet.name, tag = tag, "Tag removed");
                        self.save_state(); // Persist immediately
                    }
                }
            }
        }
        Task::none()
    }

    fn insert_wallet_runtime(
        &mut self,
        wallet: Wallet,
        secrets: WalletSecretsRef,
    ) -> Result<usize, String> {
        if wallet_id_exists(&self.wallets, wallet.wallet_id()) {
            return Err(t(
                "Ví này đã tồn tại trong ứng dụng (cùng seed/xpub).",
                "This wallet already exists in the app (same seed/xpub).",
            )
            .to_string());
        }

        let next_index = self.wallets.len();
        self.wallet_vault
            .insert(wallet.wallet_id().to_string(), secrets);
        self.wallets.push(wallet);
        Ok(next_index)
    }

    pub fn handle_create_wallet(
        &mut self,
        name: String,
        network: WalletNetwork,
    ) -> Task<AppMessage> {
        match Wallet::generate(&name, network) {
            Ok(bundle) => match self.insert_wallet_runtime(bundle.wallet, bundle.secrets) {
                Ok(selected_wallet) => {
                    self.selected_wallet = selected_wallet;
                    let _save_succeeded = self.save_state();
                    self.update_dashboard();
                    self.wallets_view = WalletsView::new();
                    self.wallets_view.set_info(t(
                        "Ví mới đã tạo. Hãy backup mnemonic ngay và hoàn thành bài test.",
                        "New wallet created. Please back up the mnemonic now and complete the backup test.",
                    ));
                    self.add_success_toast(format!(
                        "{} '{name}'. {}",
                        t("Đã tạo ví thành công", "Wallet created successfully"),
                        t("Cần backup mnemonic.", "Mnemonic backup is required.")
                    ));
                }
                Err(message) => {
                    self.wallets_view.set_error(message.clone());
                    self.add_error_toast(message);
                }
            },
            Err(err) => {
                let message = format!("{}: {err}", t("Tạo ví thất bại", "Failed to create wallet"));
                self.wallets_view.set_error(message.clone());
                self.add_error_toast(message);
            }
        }
        Task::none()
    }

    pub fn handle_import_wallet_from_mnemonic(
        &mut self,
        name: String,
        network: WalletNetwork,
        mnemonic: String,
    ) -> Task<AppMessage> {
        match Wallet::from_mnemonic(&name, network, &mnemonic) {
            Ok(bundle) => match self.insert_wallet_runtime(bundle.wallet, bundle.secrets) {
                Ok(selected_wallet) => {
                    self.selected_wallet = selected_wallet;
                    let _save_succeeded = self.save_state();
                    self.update_dashboard();
                    self.wallets_view = WalletsView::new();
                    self.wallets_view.set_info(t(
                        "Import mnemonic thành công. Ví này đã được đánh dấu backup.",
                        "Mnemonic import succeeded. This wallet has been marked as backed up.",
                    ));
                    self.add_success_toast(format!(
                        "{} '{name}' {}",
                        t("Đã import ví", "Imported wallet"),
                        t("từ mnemonic", "from mnemonic")
                    ));
                }
                Err(message) => {
                    self.wallets_view.set_error(message.clone());
                    self.add_error_toast(message);
                }
            },
            Err(err) => {
                let message = format!(
                    "{}: {err}",
                    t("Import mnemonic thất bại", "Mnemonic import failed")
                );
                self.wallets_view.set_error(message.clone());
                self.add_error_toast(message);
            }
        }
        Task::none()
    }

    pub fn handle_import_wallet_from_slip39(
        &mut self,
        name: String,
        network: WalletNetwork,
        shares: Vec<String>,
        slip39_passphrase: String,
    ) -> Task<AppMessage> {
        match Wallet::from_slip39_shares(&name, network, &shares, &slip39_passphrase) {
            Ok(bundle) => match self.insert_wallet_runtime(bundle.wallet, bundle.secrets) {
                Ok(selected_wallet) => {
                    self.selected_wallet = selected_wallet;
                    let _save_succeeded = self.save_state();
                    self.update_dashboard();
                    self.wallets_view = WalletsView::new();
                    self.wallets_view.set_info(t(
                        "Import SLIP-0039 thành công. Ví này đã được đánh dấu backup.",
                        "SLIP-0039 import succeeded. This wallet has been marked as backed up.",
                    ));
                    self.add_success_toast(format!(
                        "{} '{name}' {}",
                        t("Đã import ví", "Imported wallet"),
                        t("từ SLIP-0039", "from SLIP-0039")
                    ));
                }
                Err(message) => {
                    self.wallets_view.set_error(message.clone());
                    self.add_error_toast(message);
                }
            },
            Err(err) => {
                let message = format!(
                    "{}: {err}",
                    t("Import SLIP-0039 thất bại", "SLIP-0039 import failed")
                );
                self.wallets_view.set_error(message.clone());
                self.add_error_toast(message);
            }
        }
        Task::none()
    }

    pub fn handle_import_wallet_from_encrypted(
        &mut self,
        path: String,
        mut passphrase: String,
        name_override: Option<String>,
    ) -> Task<AppMessage> {
        let resolved_path = resolve_user_path(&path);
        let import_result = load_encrypted_secret_export(&resolved_path, &passphrase);
        passphrase.zeroize();

        match import_result {
            Ok(mut export) => {
                let imported = match &mut export {
                    DecryptedSecretExport::Mnemonic {
                        wallet_name,
                        network,
                        mnemonic,
                    } => {
                        let import_name =
                            resolve_encrypted_import_name(name_override.as_ref(), wallet_name);
                        match parse_encrypted_import_network(network) {
                            Ok(import_network) => {
                                Wallet::from_mnemonic(&import_name, import_network, mnemonic).map(
                                    |bundle| {
                                        (
                                            bundle,
                                            import_name,
                                            t(
                                                "từ backup mnemonic mã hóa",
                                                "from encrypted mnemonic backup",
                                            ),
                                        )
                                    },
                                )
                            }
                            Err(err) => Err(anyhow!(err)),
                        }
                    }
                    DecryptedSecretExport::Slip39 {
                        wallet_name: _,
                        network: _,
                        threshold: _,
                        share_count: _,
                        slip39_passphrase: _,
                        shares: _,
                    } => {
                        Err(anyhow!(t(
                            "Backup SLIP-0039 mã hóa không còn được hỗ trợ. Hãy dùng thư mục PDF shares hoặc backup ứng dụng tổng.",
                            "Encrypted SLIP-0039 backups are no longer supported. Use the PDF share folder or the full app backup instead.",
                        )))
                    }
                };

                match imported {
                    Ok((bundle, name, source_label)) => {
                        match self.insert_wallet_runtime(bundle.wallet, bundle.secrets) {
                            Ok(selected_wallet) => {
                                self.selected_wallet = selected_wallet;
                                let _save_succeeded = self.save_state();
                                self.update_dashboard();
                                self.wallets_view = WalletsView::new();
                                self.wallets_view.set_info(t(
                                    "Import file .enc thành công. Ví này đã được đánh dấu backup.",
                                    "Encrypted .enc import succeeded. This wallet has been marked as backed up.",
                                ));
                                self.add_success_toast(format!(
                                    "{} '{name}' {}",
                                    t("Đã import ví", "Imported wallet"),
                                    source_label
                                ));
                            }
                            Err(message) => {
                                self.wallets_view.set_error(message.clone());
                                self.add_error_toast(message);
                            }
                        }
                    }
                    Err(err) => {
                        let message = format!(
                            "{}: {err}",
                            t(
                                "Import backup mã hóa thất bại",
                                "Encrypted backup import failed"
                            )
                        );
                        self.wallets_view.set_error(message.clone());
                        self.add_error_toast(message);
                    }
                }
            }
            Err(err) => {
                let message = format!(
                    "{}: {err}",
                    t(
                        "Import backup mã hóa thất bại",
                        "Encrypted backup import failed"
                    )
                );
                self.wallets_view.set_error(message.clone());
                self.add_error_toast(message);
            }
        }

        Task::none()
    }

    pub fn handle_select_wallet(&mut self, index: usize) -> Task<AppMessage> {
        if index < self.wallets.len() {
            self.clear_revealed_mnemonic();
            self.selected_wallet = index;
            self.add_info_toast(format!(
                "{}: {}",
                t("Đã chọn ví", "Selected wallet"),
                self.wallets[index].name
            ));
        }
        Task::none()
    }

    pub fn handle_delete_wallet(&mut self, index: usize) -> Task<AppMessage> {
        if index < self.wallets.len() {
            self.clear_revealed_mnemonic();
            let name = self.wallets[index].name.clone();
            let wallet_id = self.wallets[index].wallet_id().to_string();
            self.wallets.remove(index);
            if should_remove_wallet_secret(&self.wallets, &wallet_id) {
                self.wallet_vault.remove(&wallet_id);
            }

            if self.wallets.is_empty() {
                self.selected_wallet = 0;
            } else if self.selected_wallet >= self.wallets.len() {
                self.selected_wallet = self.wallets.len() - 1;
            }

            let _save_succeeded = self.save_state();
            self.update_dashboard();
            self.add_info_toast(format!("{} '{name}'", t("Đã xóa ví", "Deleted wallet")));
        }
        Task::none()
    }

    pub fn handle_derive_addresses(&mut self, count: u32) -> Task<AppMessage> {
        let Some(secrets) = self.wallet_secret_by_index(self.selected_wallet) else {
            self.add_error_toast(t("Thiếu secret của ví", "Wallet secret is missing").to_string());
            return Task::none();
        };

        if let Some(wallet) = self.wallets.get_mut(self.selected_wallet) {
            match wallet.derive_next_addresses(secrets.as_ref(), count) {
                Ok(addresses) => {
                    let _save_succeeded = self.save_state();
                    self.add_success_toast(format!(
                        "{} {}",
                        t("Đã tạo", "Derived"),
                        address_count_text(addresses.len())
                    ));
                }
                Err(err) => {
                    self.add_error_toast(format!(
                        "{}: {err}",
                        t(
                            "Không thể tạo địa chỉ mới",
                            "Could not derive new addresses"
                        )
                    ));
                }
            }
        } else {
            self.add_error_toast(t("Chưa chọn ví", "No wallet selected").to_string());
        }
        Task::none()
    }

    pub fn handle_reveal_mnemonic(
        &mut self,
        wallet_index: usize,
        passphrase: String,
    ) -> Task<AppMessage> {
        let passphrase = SecretString::from(passphrase);
        if wallet_index >= self.wallets.len() {
            self.wallets_view
                .set_error(t("Ví không tồn tại", "Wallet does not exist"));
            return Task::none();
        }

        if self.current_passphrase().is_none() {
            self.wallets_view.set_error(t(
                "Không có session đăng nhập hợp lệ",
                "No active login session found",
            ));
            return Task::none();
        }

        if !self.passphrase_matches(passphrase.expose_secret()) {
            self.wallets_view.set_error(t(
                "Passphrase không đúng, không thể hiển thị mnemonic",
                "Incorrect passphrase, cannot reveal mnemonic",
            ));
            return Task::none();
        }

        let wallet_name = self.wallets[wallet_index].name.clone();
        let Some(secrets) = self.wallet_secret_by_index(wallet_index) else {
            self.wallets_view
                .set_error(t("Thiếu secret của ví", "Wallet secret is missing"));
            return Task::none();
        };

        if secrets.mnemonic_phrase().is_none() {
            self.wallets_view.set_error(t(
                "Ví này không có mnemonic để hiển thị",
                "This wallet has no mnemonic to reveal",
            ));
            return Task::none();
        }

        self.wallets_view.mark_mnemonic_revealed(wallet_index);
        self.add_info_toast(format!(
            "{} '{wallet_name}'",
            t("Đã mở khóa mnemonic cho ví", "Mnemonic unlocked for wallet")
        ));
        self.schedule_revealed_mnemonic_timeout()
    }

    pub fn handle_verify_mnemonic_backup(
        &mut self,
        wallet_index: usize,
        checks: Vec<(usize, String)>,
    ) -> Task<AppMessage> {
        if wallet_index >= self.wallets.len() {
            self.wallets_view
                .set_error(t("Ví không tồn tại", "Wallet does not exist"));
            return Task::none();
        }

        let verification = {
            let Some(secrets) = self.wallet_secret_by_index(wallet_index) else {
                self.wallets_view
                    .set_error(t("Thiếu secret của ví", "Wallet secret is missing"));
                return Task::none();
            };
            let mnemonic = match secrets.mnemonic_phrase() {
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
                self.add_success_toast(format!(
                    "{} '{wallet_name}'",
                    t(
                        "Ví đã vượt qua bài test backup mnemonic",
                        "Wallet passed mnemonic backup test",
                    )
                ));
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

    pub fn handle_export_mnemonic_pdf(&mut self, wallet_index: usize) -> Task<AppMessage> {
        if wallet_index >= self.wallets.len() {
            self.wallets_view
                .set_error(t("Ví không tồn tại", "Wallet does not exist"));
            return Task::none();
        }

        let wallet = &self.wallets[wallet_index];
        let Some(secrets) = self.wallet_secret(wallet) else {
            self.wallets_view
                .set_error(t("Thiếu secret của ví", "Wallet secret is missing"));
            return Task::none();
        };
        let mnemonic = match secrets.mnemonic_phrase() {
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
                self.add_info_toast(message);
            }
            Err(err) => {
                let message = format!(
                    "{}: {err}",
                    t(
                        "Export mnemonic PDF thất bại",
                        "Failed to export mnemonic PDF"
                    )
                );
                self.wallets_view.set_error(message.clone());
                self.add_error_toast(message);
            }
        }
        Task::none()
    }

    pub fn handle_export_mnemonic_encrypted(&mut self, wallet_index: usize) -> Task<AppMessage> {
        if wallet_index >= self.wallets.len() {
            self.wallets_view
                .set_error(t("Ví không tồn tại", "Wallet does not exist"));
            return Task::none();
        }

        let encryption_passphrase = match self.current_passphrase() {
            Some(value) => value.expose_secret(),
            None => {
                self.wallets_view.set_error(t(
                    "Không có session đăng nhập hợp lệ",
                    "No active login session found",
                ));
                return Task::none();
            }
        };

        let wallet = &self.wallets[wallet_index];
        let Some(secrets) = self.wallet_secret(wallet) else {
            self.wallets_view
                .set_error(t("Thiếu secret của ví", "Wallet secret is missing"));
            return Task::none();
        };
        let mnemonic = match secrets.mnemonic_phrase() {
            Some(value) => value,
            None => {
                self.wallets_view.set_error(t(
                    "Ví này không có mnemonic để export mã hóa",
                    "This wallet has no mnemonic to export as encrypted backup",
                ));
                return Task::none();
            }
        };

        let default_name = default_mnemonic_encrypted_filename(&wallet.name);
        let Some(raw_path) = pick_encrypted_export_path(
            t("Lưu mnemonic mã hóa", "Save encrypted mnemonic backup"),
            &default_name,
        ) else {
            return Task::none();
        };
        let export_path = ensure_enc_extension(raw_path);

        match export_mnemonic_to_encrypted_file(
            &export_path,
            &wallet.name,
            wallet.network.as_str(),
            mnemonic,
            encryption_passphrase,
        ) {
            Ok(_) => {
                let message = format!(
                    "{}: {}",
                    t(
                        "Đã export mnemonic mã hóa bằng passphrase hiện tại",
                        "Exported encrypted mnemonic using the current passphrase",
                    ),
                    export_path.display()
                );
                self.wallets_view.set_info(message.clone());
                self.add_info_toast(message);
            }
            Err(err) => {
                let message = format!(
                    "{}: {err}",
                    t(
                        "Export mnemonic mã hóa thất bại",
                        "Failed to export encrypted mnemonic",
                    )
                );
                self.wallets_view.set_error(message.clone());
                self.add_error_toast(message);
            }
        }

        Task::none()
    }

    pub fn handle_export_wallet_slip39(
        &mut self,
        wallet_index: usize,
        threshold: u8,
        share_count: u8,
        slip39_passphrase: String,
    ) -> Task<AppMessage> {
        if wallet_index >= self.wallets.len() {
            self.wallets_view
                .set_error(t("Ví không tồn tại", "Wallet does not exist"));
            return Task::none();
        }

        let wallet = &self.wallets[wallet_index];
        let Some(secrets) = self.wallet_secret(wallet) else {
            self.wallets_view
                .set_error(t("Thiếu secret của ví", "Wallet secret is missing"));
            return Task::none();
        };
        let mnemonic = match secrets.mnemonic_phrase() {
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

        let default_dir_name = default_slip39_directory_name(&wallet.name, threshold, share_count);
        let Some(base_directory) = pick_slip39_export_directory() else {
            return Task::none();
        };

        let export = Slip39PdfExport {
            wallet_name: &wallet.name,
            network: wallet.network.as_str(),
            threshold,
            share_count,
            has_slip39_passphrase: !slip39_passphrase.trim().is_empty(),
        };

        match export_slip39_shares_to_pdf_directory(
            &base_directory,
            &default_dir_name,
            &export,
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
                self.add_info_toast(message);
            }
            Err(err) => {
                let message = format!(
                    "{}: {err}",
                    t("Export SLIP-0039 thất bại", "Failed to export SLIP-0039")
                );
                self.wallets_view.set_error(message.clone());
                self.add_error_toast(message);
            }
        }
        Task::none()
    }

    pub fn handle_receive_message(&mut self, msg: ReceiveMessage) -> Task<AppMessage> {
        if let Some(event) = self.receive_view.update(msg) {
            match event {
                ReceiveEvent::SelectWallet(index) => return self.handle_select_wallet(index),
                ReceiveEvent::CopyAddress(addr) => {
                    self.track_copy(addr.clone());
                    self.add_success_toast(format!(
                        "{} {}",
                        t("Đã copy địa chỉ", "Copied address"),
                        &addr[..8.min(addr.len())]
                    ));
                    return clipboard::write(addr);
                }
                ReceiveEvent::DeriveAddresses(count) => return self.handle_derive_addresses(count),
            }
        }
        Task::none()
    }
}

fn resolve_encrypted_import_name(name_override: Option<&String>, backup_name: &str) -> String {
    match name_override {
        Some(value) => value.clone(),
        None => {
            let trimmed = backup_name.trim();
            if trimmed.is_empty() {
                t("Ví import", "Imported wallet").to_string()
            } else {
                trimmed.to_string()
            }
        }
    }
}

fn parse_encrypted_import_network(raw: &str) -> Result<WalletNetwork, String> {
    raw.parse().map_err(|_| {
        format!(
            "{}: {raw}",
            t(
                "Network trong backup mã hóa không hợp lệ",
                "Invalid network in encrypted backup",
            )
        )
    })
}

fn wallet_id_exists(wallets: &[Wallet], wallet_id: &str) -> bool {
    wallets.iter().any(|wallet| wallet.wallet_id() == wallet_id)
}

fn should_remove_wallet_secret(wallets: &[Wallet], wallet_id: &str) -> bool {
    !wallet_id_exists(wallets, wallet_id)
}

#[cfg(test)]
mod tests {
    use super::{should_remove_wallet_secret, wallet_id_exists};
    use crate::core::wallet::{Wallet, WalletNetwork};

    #[test]
    fn duplicate_wallet_ids_are_detected() {
        let bundle = Wallet::generate("Primary", WalletNetwork::Testnet)
            .expect("wallet generation should succeed");
        let mut duplicate = bundle.wallet.clone();
        duplicate.name = "Primary Copy".to_string();

        assert!(wallet_id_exists(
            &[bundle.wallet.clone(), duplicate],
            bundle.wallet.wallet_id()
        ));
    }

    #[test]
    fn shared_secret_is_kept_until_last_duplicate_wallet_is_removed() {
        let bundle = Wallet::generate("Primary", WalletNetwork::Testnet)
            .expect("wallet generation should succeed");
        let mut duplicate = bundle.wallet.clone();
        duplicate.name = "Primary Copy".to_string();
        let wallet_id = bundle.wallet.wallet_id().to_string();

        let mut wallets = vec![bundle.wallet.clone(), duplicate];
        wallets.remove(0);
        assert!(!should_remove_wallet_secret(&wallets, &wallet_id));

        wallets.clear();
        assert!(should_remove_wallet_secret(&wallets, &wallet_id));
    }
}
