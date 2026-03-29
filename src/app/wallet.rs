use iced::{clipboard, Task};

use crate::i18n::t;
use crate::views::receive::{ReceiveEvent, ReceiveMessage};
use crate::views::wallets::{WalletsEvent, WalletsMessage, WalletsView};
use crate::wallet::{Wallet, WalletNetwork};

use super::{
    address_count_text, default_mnemonic_pdf_filename, default_slip39_directory_name,
    ensure_pdf_extension, export_mnemonic_to_pdf, export_slip39_shares_to_pdf_directory,
    pick_mnemonic_pdf_path, pick_slip39_export_directory, App, AppMessage,
};

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
                    return self.handle_import_wallet_from_mnemonic(name, network, mnemonic);
                }
                WalletsEvent::ImportWalletFromSlip39 {
                    name,
                    network,
                    shares,
                    slip39_passphrase,
                } => {
                    return self.handle_import_wallet_from_slip39(name, network, shares, slip39_passphrase);
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
                    return self.handle_reveal_mnemonic(wallet_index, passphrase);
                }
                WalletsEvent::VerifyMnemonicBackup { wallet_index, checks } => {
                    return self.handle_verify_mnemonic_backup(wallet_index, checks);
                }
                WalletsEvent::ExportMnemonicPdf(index) => {
                    return self.handle_export_mnemonic_pdf(index);
                }
                WalletsEvent::ExportWalletSlip39 {
                    wallet_index,
                    threshold,
                    share_count,
                    slip39_passphrase,
                } => {
                    return self.handle_export_wallet_slip39(wallet_index, threshold, share_count, slip39_passphrase);
                }
            }
        }
        Task::none()
    }

    pub fn handle_create_wallet(&mut self, name: String, network: WalletNetwork) -> Task<AppMessage> {
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

    pub fn handle_import_wallet_from_mnemonic(
        &mut self,
        name: String,
        network: WalletNetwork,
        mnemonic: String,
    ) -> Task<AppMessage> {
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

    pub fn handle_import_wallet_from_slip39(
        &mut self,
        name: String,
        network: WalletNetwork,
        shares: Vec<String>,
        slip39_passphrase: String,
    ) -> Task<AppMessage> {
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

    pub fn handle_select_wallet(&mut self, index: usize) -> Task<AppMessage> {
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

    pub fn handle_delete_wallet(&mut self, index: usize) -> Task<AppMessage> {
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

    pub fn handle_derive_addresses(&mut self, count: u32) -> Task<AppMessage> {
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

    pub fn handle_reveal_mnemonic(
        &mut self,
        wallet_index: usize,
        passphrase: String,
    ) -> Task<AppMessage> {
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

    pub fn handle_export_mnemonic_pdf(&mut self, wallet_index: usize) -> Task<AppMessage> {
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

    pub fn handle_receive_message(&mut self, msg: ReceiveMessage) -> Task<AppMessage> {
        if let Some(event) = self.receive_view.update(msg) {
            match event {
                ReceiveEvent::SelectWallet(index) => return self.handle_select_wallet(index),
                ReceiveEvent::CopyAddress(addr) => {
                    self.status = Some(
                        t(
                            "Đã copy địa chỉ vào clipboard",
                            "Copied address to clipboard",
                        )
                        .to_string(),
                    );
                    self.error = None;
                    return clipboard::write(addr);
                }
                ReceiveEvent::DeriveAddresses(count) => return self.handle_derive_addresses(count),
            }
        }
        Task::none()
    }
}
