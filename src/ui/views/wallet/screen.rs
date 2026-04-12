use iced::{
    widget::{
        button, column, container, mouse_area, pick_list, row, scrollable, text, text_input,
        tooltip, Space,
    },
    Alignment, Background, Color, Element, Length,
};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

use crate::core::wallet::{AddressChain, Wallet, WalletNetwork};
use crate::ui::components::info_box;
use crate::ui::components::modal;
use crate::ui::i18n::t;
use crate::ui::theme::{
    card_style, danger_button_style, get_theme_colors, input_style, notice_style,
    pick_list_menu_style, pick_list_style, primary_button_style, secondary_button_style,
    selected_button_style, text_color, text_muted_color, text_primary_color, text_scaled,
    text_secondary_color, Colors, NoticeTone,
};

use super::structure::*;
use crate::infra::storage::WalletSortField;
use crate::ui::components::tag_picker::tag_picker;

impl WalletsView {
    pub fn new() -> Self {
        Self {
            create_name: String::new(),
            create_network: WalletNetwork::Testnet,
            show_create_form: false,
            import_mode: ImportMode::Bip39,
            import_name: String::new(),
            import_network: WalletNetwork::Testnet,
            import_mnemonic: String::new(),
            import_encrypted_path: String::new(),
            import_encrypted_passphrase: String::new(),
            import_slip39_passphrase: String::new(),
            import_slip39_shares: vec![String::new(), String::new()],
            show_import_mnemonic_form: false,
            confirm_delete_index: None,
            delete_passphrase: String::new(),
            notice_wallet_index: None,
            mnemonic_passphrase: String::new(),
            revealed_wallet_index: None,
            backup_test_wallet_index: None,
            backup_test_positions: Vec::new(),
            backup_test_answers: Vec::new(),
            slip39_export_threshold: "2".to_string(),
            slip39_export_share_count: "3".to_string(),
            slip39_export_passphrase: String::new(),
            show_external_addresses: false,
            show_internal_addresses: false,
            copied_address: None,
            info: None,
            error: None,
            compact_mode: false,
            sort_field: WalletSortField::Balance,
            sort_ascending: false,
            tag_input: String::new(),
            tag_modal_index: None,
        }
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error = Some(message.into());
    }

    pub fn set_info(&mut self, message: impl Into<String>) {
        self.info = Some(message.into());
        self.error = None;
    }

    pub fn mark_mnemonic_revealed(&mut self, wallet_index: usize) {
        self.revealed_wallet_index = Some(wallet_index);
        self.mnemonic_passphrase.clear();
        self.notice_wallet_index = Some(wallet_index);
        self.info = Some(
            t(
                "Mnemonic đã hiển thị. Hãy backup an toàn và hoàn thành bài test xác nhận.",
                "Mnemonic revealed. Please back it up safely and complete the verification test.",
            )
            .to_string(),
        );
        self.error = None;
    }

    pub fn mark_backup_verified(&mut self, wallet_index: usize) {
        self.notice_wallet_index = None;
        self.backup_test_wallet_index = None;
        self.backup_test_positions.clear();
        self.backup_test_answers.clear();
        self.info = Some(
            t(
                "Backup mnemonic đã được xác nhận thành công.",
                "Mnemonic backup has been verified successfully.",
            )
            .to_string(),
        );
        self.error = None;
        self.revealed_wallet_index = Some(wallet_index);
    }

    pub fn clear_import_sensitive_inputs(&mut self) {
        self.import_mnemonic.clear();
        self.import_encrypted_passphrase.clear();
        self.import_slip39_passphrase.clear();
        self.import_slip39_shares.iter_mut().for_each(String::clear);
    }

    pub fn set_import_encrypted_path(&mut self, path: impl Into<String>) {
        self.import_encrypted_path = path.into();
        self.error = None;
    }

    pub fn clear_reveal_sensitive_inputs(&mut self) {
        self.mnemonic_passphrase.clear();
    }

    pub fn clear_backup_test_inputs(&mut self) {
        self.backup_test_answers.iter_mut().for_each(String::clear);
    }

    pub fn clear_export_sensitive_inputs(&mut self) {
        self.slip39_export_passphrase.clear();
    }

    pub fn clear_revealed_mnemonic(&mut self) {
        self.revealed_wallet_index = None;
        self.mnemonic_passphrase.clear();
        self.backup_test_wallet_index = None;
        self.backup_test_positions.clear();
        self.backup_test_answers.clear();
        self.slip39_export_passphrase.clear();
    }

    pub fn revealed_wallet_index(&self) -> Option<usize> {
        self.revealed_wallet_index
    }

    pub fn update(&mut self, message: WalletsMessage) -> Option<WalletsEvent> {
        match message {
            WalletsMessage::ToggleCreateForm => {
                self.show_create_form = !self.show_create_form;
                if self.show_create_form {
                    self.show_import_mnemonic_form = false;
                    self.clear_import_sensitive_inputs();
                }
                self.error = None;
                None
            }
            WalletsMessage::CreateWallet => {
                if self.create_name.trim().is_empty() {
                    return None;
                }
                let name = self.create_name.clone();
                let network = self.create_network;
                self.create_name.clear();
                self.show_create_form = false;
                self.error = None;
                Some(WalletsEvent::CreateWallet(name, network))
            }
            WalletsMessage::NameChanged(name) => {
                self.create_name = name;
                None
            }
            WalletsMessage::NetworkChanged(network) => {
                self.create_network = network;
                None
            }
            WalletsMessage::ToggleImportMnemonicForm => {
                self.show_import_mnemonic_form = !self.show_import_mnemonic_form;
                if self.show_import_mnemonic_form {
                    self.show_create_form = false;
                } else {
                    self.clear_import_sensitive_inputs();
                }
                self.error = None;
                None
            }
            WalletsMessage::ImportModeChanged(mode) => {
                self.import_mode = mode;
                self.clear_import_sensitive_inputs();
                self.error = None;
                None
            }
            WalletsMessage::ImportNameChanged(name) => {
                self.import_name = name;
                self.error = None;
                None
            }
            WalletsMessage::ImportNetworkChanged(network) => {
                self.import_network = network;
                self.error = None;
                None
            }
            WalletsMessage::ImportMnemonicChanged(value) => {
                self.import_mnemonic = value;
                self.error = None;
                None
            }
            WalletsMessage::ImportEncryptedPathChanged(value) => {
                self.import_encrypted_path = value;
                self.error = None;
                None
            }
            WalletsMessage::ImportEncryptedPassphraseChanged(value) => {
                self.import_encrypted_passphrase = value;
                self.error = None;
                None
            }
            WalletsMessage::ImportSlip39PassphraseChanged(value) => {
                self.import_slip39_passphrase = value;
                self.error = None;
                None
            }
            WalletsMessage::ImportSlip39ShareChanged(index, value) => {
                if let Some(slot) = self.import_slip39_shares.get_mut(index) {
                    *slot = value;
                }
                self.error = None;
                None
            }
            WalletsMessage::AddImportSlip39Share => {
                if self.import_slip39_shares.len() >= 16 {
                    self.error = Some(
                        t(
                            "Tối đa 16 SLIP-0039 share",
                            "Maximum 16 SLIP-0039 shares are supported",
                        )
                        .to_string(),
                    );
                    return None;
                }
                self.import_slip39_shares.push(String::new());
                self.error = None;
                None
            }
            WalletsMessage::RemoveImportSlip39Share => {
                if self.import_slip39_shares.len() > 2 {
                    self.import_slip39_shares.pop();
                }
                self.error = None;
                None
            }
            WalletsMessage::BrowseImportEncryptedPath => {
                self.error = None;
                Some(WalletsEvent::BrowseImportEncryptedPath)
            }
            WalletsMessage::ImportWalletFromMnemonic => {
                if self.import_name.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập tên ví import",
                            "Please enter a wallet name for import",
                        )
                        .to_string(),
                    );
                    return None;
                }
                if self.import_mnemonic.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập mnemonic để import",
                            "Please enter a mnemonic to import",
                        )
                        .to_string(),
                    );
                    return None;
                }

                let name = self.import_name.trim().to_string();
                let network = self.import_network;
                let mnemonic = self.import_mnemonic.trim().to_string();

                self.import_name.clear();
                self.import_mnemonic.clear();
                self.show_import_mnemonic_form = false;
                self.error = None;

                Some(WalletsEvent::ImportWalletFromMnemonic {
                    name,
                    network,
                    mnemonic,
                })
            }
            WalletsMessage::ImportWalletFromSlip39 => {
                if self.import_name.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập tên ví import",
                            "Please enter a wallet name for import",
                        )
                        .to_string(),
                    );
                    return None;
                }

                let shares = self
                    .import_slip39_shares
                    .iter()
                    .map(|share| share.trim())
                    .filter(|share| !share.is_empty())
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();

                if shares.len() < 2 {
                    self.error = Some(
                        t(
                            "Vui lòng nhập ít nhất 2 SLIP-0039 share",
                            "Please enter at least 2 SLIP-0039 shares",
                        )
                        .to_string(),
                    );
                    return None;
                }

                let name = self.import_name.trim().to_string();
                let network = self.import_network;
                let slip39_passphrase = self.import_slip39_passphrase.clone();

                self.import_name.clear();
                self.import_mnemonic.clear();
                self.import_slip39_passphrase.clear();
                self.import_slip39_shares = vec![String::new(), String::new()];
                self.show_import_mnemonic_form = false;
                self.error = None;

                Some(WalletsEvent::ImportWalletFromSlip39 {
                    name,
                    network,
                    shares,
                    slip39_passphrase,
                })
            }
            WalletsMessage::ImportWalletFromEncrypted => {
                if self.import_encrypted_path.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng chọn file backup mnemonic mã hóa (.enc)",
                            "Please choose an encrypted mnemonic backup file (.enc)",
                        )
                        .to_string(),
                    );
                    return None;
                }

                if self.import_encrypted_passphrase.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập passphrase để giải mã mnemonic backup .enc",
                            "Please enter the passphrase to decrypt the encrypted mnemonic backup",
                        )
                        .to_string(),
                    );
                    return None;
                }

                self.error = None;
                Some(WalletsEvent::ImportWalletFromEncrypted {
                    path: self.import_encrypted_path.trim().to_string(),
                    passphrase: self.import_encrypted_passphrase.clone(),
                    name_override: optional_trimmed_value(&self.import_name),
                })
            }
            WalletsMessage::SelectWallet(index) => {
                self.revealed_wallet_index = None;
                self.clear_import_sensitive_inputs();
                self.mnemonic_passphrase.clear();
                self.backup_test_wallet_index = None;
                self.backup_test_positions.clear();
                self.backup_test_answers.clear();
                self.show_create_form = false;
                self.show_import_mnemonic_form = false;
                self.error = None;
                Some(WalletsEvent::SelectWallet(index))
            }
            WalletsMessage::DeleteWallet(index) => {
                self.confirm_delete_index = Some(index);
                None
            }
            WalletsMessage::ConfirmDelete(index) => {
                self.confirm_delete_index = None;
                Some(WalletsEvent::DeleteWallet(index))
            }
            WalletsMessage::CancelDelete => {
                self.confirm_delete_index = None;
                self.delete_passphrase.clear();
                None
            }
            WalletsMessage::DeletePassphraseChanged(passphrase) => {
                self.delete_passphrase = passphrase;
                None
            }
            WalletsMessage::ShowBackupWarning(index) => {
                self.notice_wallet_index = Some(index);
                self.info = Some(
                    t(
                        "Ví này chưa backup mnemonic. Hãy mở mnemonic và hoàn thành bài test.",
                        "This wallet has not backed up its mnemonic yet. Reveal it and complete the backup test.",
                    )
                    .to_string(),
                );
                None
            }
            WalletsMessage::MnemonicPassphraseChanged(value) => {
                self.mnemonic_passphrase = value;
                self.error = None;
                None
            }
            WalletsMessage::RevealMnemonic(wallet_index) => {
                if self.mnemonic_passphrase.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập passphrase để hiện mnemonic",
                            "Please enter passphrase to reveal mnemonic",
                        )
                        .to_string(),
                    );
                    return None;
                }

                self.error = None;
                Some(WalletsEvent::RevealMnemonic {
                    wallet_index,
                    passphrase: self.mnemonic_passphrase.clone(),
                })
            }
            WalletsMessage::ToggleBackupTest {
                wallet_index,
                word_count,
            } => {
                if self.backup_test_wallet_index == Some(wallet_index) {
                    self.backup_test_wallet_index = None;
                    self.backup_test_positions.clear();
                    self.backup_test_answers.clear();
                    return None;
                }

                let positions = test_positions(word_count);
                self.backup_test_answers = vec![String::new(); positions.len()];
                self.backup_test_positions = positions;
                self.backup_test_wallet_index = Some(wallet_index);
                self.error = None;
                None
            }
            WalletsMessage::ExportMnemonicPdf(wallet_index) => {
                if self.revealed_wallet_index != Some(wallet_index) {
                    self.error = Some(
                        t(
                            "Hãy mở mnemonic trước khi export PDF",
                            "Please reveal mnemonic before exporting PDF",
                        )
                        .to_string(),
                    );
                    return None;
                }
                if self.backup_test_wallet_index == Some(wallet_index) {
                    self.error = Some(
                        t(
                            "Không thể export PDF khi đang làm bài test backup",
                            "Cannot export PDF while backup test is in progress",
                        )
                        .to_string(),
                    );
                    return None;
                }
                self.error = None;
                Some(WalletsEvent::ExportMnemonicPdf(wallet_index))
            }
            WalletsMessage::ExportMnemonicEncrypted(wallet_index) => {
                if self.revealed_wallet_index != Some(wallet_index) {
                    self.error = Some(
                        t(
                            "Hãy mở mnemonic trước khi export bản mã hóa",
                            "Please reveal mnemonic before exporting encrypted backup",
                        )
                        .to_string(),
                    );
                    return None;
                }
                if self.backup_test_wallet_index == Some(wallet_index) {
                    self.error = Some(
                        t(
                            "Không thể export bản mã hóa khi đang làm bài test backup",
                            "Cannot export encrypted backup while backup test is in progress",
                        )
                        .to_string(),
                    );
                    return None;
                }
                self.error = None;
                Some(WalletsEvent::ExportMnemonicEncrypted(wallet_index))
            }
            WalletsMessage::Slip39ExportThresholdChanged(value) => {
                self.slip39_export_threshold = value;
                self.error = None;
                None
            }
            WalletsMessage::Slip39ExportShareCountChanged(value) => {
                self.slip39_export_share_count = value;
                self.error = None;
                None
            }
            WalletsMessage::Slip39ExportPassphraseChanged(value) => {
                self.slip39_export_passphrase = value;
                self.error = None;
                None
            }
            WalletsMessage::ExportSlip39Shares(wallet_index) => {
                if self.revealed_wallet_index != Some(wallet_index) {
                    self.error = Some(
                        t(
                            "Hãy mở mnemonic trước khi export SLIP-0039",
                            "Please reveal mnemonic before exporting SLIP-0039",
                        )
                        .to_string(),
                    );
                    return None;
                }
                if self.backup_test_wallet_index == Some(wallet_index) {
                    self.error = Some(
                        t(
                            "Không thể export SLIP-0039 khi đang làm bài test backup",
                            "Cannot export SLIP-0039 while backup test is in progress",
                        )
                        .to_string(),
                    );
                    return None;
                }

                let threshold =
                    match parse_u8_field(&self.slip39_export_threshold, "Ngưỡng K", "Threshold K")
                    {
                        Ok(value) => value,
                        Err(message) => {
                            self.error = Some(message);
                            return None;
                        }
                    };
                let share_count = match parse_u8_field(
                    &self.slip39_export_share_count,
                    "Số lượng share N",
                    "Total share count N",
                ) {
                    Ok(value) => value,
                    Err(message) => {
                        self.error = Some(message);
                        return None;
                    }
                };

                if threshold < 2 {
                    self.error = Some(
                        t(
                            "Ngưỡng K nên từ 2 trở lên",
                            "Threshold K must be at least 2",
                        )
                        .to_string(),
                    );
                    return None;
                }
                if share_count < threshold {
                    self.error = Some(
                        t(
                            "Số lượng share N phải >= ngưỡng K",
                            "Total share count N must be >= threshold K",
                        )
                        .to_string(),
                    );
                    return None;
                }
                if share_count > 16 {
                    self.error = Some(
                        t(
                            "SLIP-0039 hiện hỗ trợ tối đa 16 share",
                            "SLIP-0039 currently supports at most 16 shares",
                        )
                        .to_string(),
                    );
                    return None;
                }

                self.error = None;
                Some(WalletsEvent::ExportWalletSlip39 {
                    wallet_index,
                    threshold,
                    share_count,
                    slip39_passphrase: self.slip39_export_passphrase.clone(),
                })
            }
            WalletsMessage::BackupWordChanged(field_index, value) => {
                if let Some(slot) = self.backup_test_answers.get_mut(field_index) {
                    *slot = value;
                }
                self.error = None;
                None
            }
            WalletsMessage::SubmitBackupTest(wallet_index) => {
                if self.backup_test_wallet_index != Some(wallet_index) {
                    self.error = Some(
                        t(
                            "Bạn chưa bắt đầu bài test backup cho ví này",
                            "You have not started the backup test for this wallet",
                        )
                        .to_string(),
                    );
                    return None;
                }

                if self
                    .backup_test_answers
                    .iter()
                    .any(|word| word.trim().is_empty())
                {
                    self.error = Some(
                        t(
                            "Vui lòng điền đầy đủ các từ trong bài test",
                            "Please fill in all words in the backup test",
                        )
                        .to_string(),
                    );
                    return None;
                }

                let checks = self
                    .backup_test_positions
                    .iter()
                    .copied()
                    .zip(self.backup_test_answers.iter().cloned())
                    .collect::<Vec<_>>();

                Some(WalletsEvent::VerifyMnemonicBackup {
                    wallet_index,
                    checks,
                })
            }
            WalletsMessage::ToggleExternalAddresses => {
                self.show_external_addresses = !self.show_external_addresses;
                None
            }
            WalletsMessage::ToggleInternalAddresses => {
                self.show_internal_addresses = !self.show_internal_addresses;
                None
            }
            WalletsMessage::CopyAddress(address) => {
                self.copied_address = Some(address.clone());
                Some(WalletsEvent::CopyAddress(address))
            }
            WalletsMessage::DismissWalletNotice => {
                self.notice_wallet_index = None;
                self.info = None;
                self.error = None;
                None
            }
            WalletsMessage::SortFieldChanged(field) => {
                self.sort_field = field;
                Some(WalletsEvent::SortFieldChanged(field))
            }
            WalletsMessage::ToggleSortDirection => {
                self.sort_ascending = !self.sort_ascending;
                Some(WalletsEvent::ToggleSortDirection)
            }
            WalletsMessage::TagMessage(tag_msg) => match tag_msg {
                crate::ui::components::tag_picker::TagMessage::InputChanged(text) => {
                    self.tag_input = text;
                    None
                }
                crate::ui::components::tag_picker::TagMessage::SelectTag(tag) => {
                    self.tag_input = tag.clone();
                    Some(WalletsEvent::TagInputChanged(tag))
                }
                crate::ui::components::tag_picker::TagMessage::CreateTag(tag) => {
                    if !tag.is_empty() {
                        self.tag_input = tag.clone();
                        Some(WalletsEvent::TagInputChanged(tag))
                    } else {
                        None
                    }
                }
                crate::ui::components::tag_picker::TagMessage::RemoveTag(tag) => {
                    self.tag_input.clear();
                    Some(WalletsEvent::RemoveTag(tag))
                }
            },
            WalletsMessage::ToggleTagModal(index) => {
                if self.tag_modal_index == Some(index) {
                    self.tag_modal_index = None;
                    self.tag_input.clear();
                } else {
                    self.tag_modal_index = Some(index);
                    self.tag_input.clear();
                }
                None
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        wallets: &'a [Wallet],
        selected: usize,
        revealed_mnemonic: Option<&'a str>,
        compact_mode: bool,
        btc_price_usd: Option<f64>,
    ) -> Element<'a, WalletsMessage> {
        let title = text_scaled(t("Ví", "Wallets"), 32).style(text_primary_color());

        // Apply compact mode
        let spacing = if compact_mode { 8 } else { 16 };
        let btn_spacing = if compact_mode { 6 } else { 10 };
        let btn_padding = if compact_mode { 8 } else { 10 };
        let _card_padding = if compact_mode { 12 } else { 16 };
        let main_padding = if compact_mode { 16 } else { 32 };

        let create_toggle_btn = button(text_scaled(
            if self.show_create_form {
                t("Hủy tạo", "Cancel Create")
            } else {
                t("+ Tạo ví", "+ Create Wallet")
            },
            14,
        ))
        .on_press(WalletsMessage::ToggleCreateForm)
        .padding(btn_padding)
        .style(if self.show_create_form {
            secondary_button_style()
        } else {
            primary_button_style()
        });

        let import_toggle_btn = button(text_scaled(
            if self.show_import_mnemonic_form {
                t("Hủy import", "Cancel Import")
            } else {
                t("+ Import ví", "+ Import Wallet")
            },
            14,
        ))
        .on_press(WalletsMessage::ToggleImportMnemonicForm)
        .padding(btn_padding)
        .style(if self.show_import_mnemonic_form {
            secondary_button_style()
        } else {
            primary_button_style()
        });

        // Header (Pinned at top)
        let header = column![
            title,
            Space::with_height(spacing),
            row![
                create_toggle_btn,
                Space::with_width(btn_spacing),
                import_toggle_btn
            ]
            .align_y(Alignment::Center)
        ]
        .spacing(spacing)
        .padding(main_padding);

        // Scrollable Content
        let mut scroll_content = column![].spacing(spacing).padding(main_padding);

        if let Some(info) = &self.info {
            scroll_content = scroll_content.push(
                container(
                    row![
                        text_scaled(info.as_str(), 13).style(text_primary_color()),
                        Space::with_width(Length::Fill),
                        button(text_scaled(t("Đóng", "Close"), 12))
                            .on_press(WalletsMessage::DismissWalletNotice)
                            .padding(4)
                            .style(secondary_button_style()),
                    ]
                    .align_y(Alignment::Center),
                )
                .style(notice_style(NoticeTone::Info))
                .padding(10)
                .width(Length::Fill),
            );
        }

        if let Some(error) = &self.error {
            scroll_content = scroll_content.push(
                container(text(error.as_str()).size(13).style(text_primary_color()))
                    .style(notice_style(NoticeTone::Error))
                    .padding(10)
                    .width(Length::Fill),
            );
        }

        if self.show_create_form {
            let name_input = text_input(t("Tên ví...", "Wallet name..."), &self.create_name)
                .on_input(WalletsMessage::NameChanged)
                .padding(12)
                .size(16);

            let network_testnet = button(text_scaled(t("Testnet", "Testnet"), 14))
                .on_press(WalletsMessage::NetworkChanged(WalletNetwork::Testnet))
                .padding(8)
                .style(if self.create_network == WalletNetwork::Testnet {
                    selected_button_style()
                } else {
                    secondary_button_style()
                });

            let network_mainnet = button(text_scaled(t("Mainnet", "Mainnet"), 14))
                .on_press(WalletsMessage::NetworkChanged(WalletNetwork::Mainnet))
                .padding(8)
                .style(if self.create_network == WalletNetwork::Mainnet {
                    selected_button_style()
                } else {
                    secondary_button_style()
                });

            let create_btn = button(text_scaled(t("Tạo ví", "Create"), 14))
                .on_press(WalletsMessage::CreateWallet)
                .padding(10)
                .style(primary_button_style());

            let form = container(
                column![
                    text_scaled(t("Tạo ví mới", "Create New Wallet"), 18)
                        .style(text_primary_color()),
                    text_scaled(t(
                        "Ví mới sẽ tạo mnemonic 12 từ và yêu cầu backup ngay sau khi tạo.",
                        "A new wallet will generate a 12-word mnemonic and prompt immediate backup.",
                    ), 12)
                    .style(text_secondary_color()),
                    Space::with_height(12),
                    name_input,
                    Space::with_height(8),
                    row![network_testnet, network_mainnet].spacing(8),
                    Space::with_height(12),
                    create_btn,
                ]
                .spacing(8),
            )
            .style(card_style())
            .padding(20)
            .width(Length::Fill);

            scroll_content = scroll_content.push(form);
        }

        if self.show_import_mnemonic_form {
            let import_name_placeholder = if self.import_mode == ImportMode::Encrypted {
                t(
                    "Tên ví override (không bắt buộc)...",
                    "Optional wallet name override...",
                )
            } else {
                t("Tên ví...", "Wallet name...")
            };
            let import_name_input = text_input(import_name_placeholder, &self.import_name)
                .on_input(WalletsMessage::ImportNameChanged)
                .padding(12)
                .size(16)
                .style(input_style());

            let network_testnet = button(text_scaled(t("Testnet", "Testnet"), 14))
                .on_press(WalletsMessage::ImportNetworkChanged(WalletNetwork::Testnet))
                .padding(8)
                .style(if self.import_network == WalletNetwork::Testnet {
                    selected_button_style()
                } else {
                    secondary_button_style()
                });

            let network_mainnet = button(text_scaled(t("Mainnet", "Mainnet"), 14))
                .on_press(WalletsMessage::ImportNetworkChanged(WalletNetwork::Mainnet))
                .padding(8)
                .style(if self.import_network == WalletNetwork::Mainnet {
                    selected_button_style()
                } else {
                    secondary_button_style()
                });

            let mode_bip39 = button(text_scaled("BIP39", 13))
                .on_press(WalletsMessage::ImportModeChanged(ImportMode::Bip39))
                .padding(8)
                .style(if self.import_mode == ImportMode::Bip39 {
                    selected_button_style()
                } else {
                    secondary_button_style()
                });

            let mode_slip39 = button(text_scaled("SLIP-0039", 13))
                .on_press(WalletsMessage::ImportModeChanged(ImportMode::Slip39))
                .padding(8)
                .style(if self.import_mode == ImportMode::Slip39 {
                    selected_button_style()
                } else {
                    secondary_button_style()
                });

            let mode_encrypted = button(text_scaled("Mnemonic .enc", 13))
                .on_press(WalletsMessage::ImportModeChanged(ImportMode::Encrypted))
                .padding(8)
                .style(if self.import_mode == ImportMode::Encrypted {
                    selected_button_style()
                } else {
                    secondary_button_style()
                });

            let mut form_content = column![
                text_scaled(t("Import ví", "Import Wallet"), 18).style(text_primary_color()),
                Space::with_height(8),
                text_scaled(
                    t(
                        "Khôi phục từng ví riêng lẻ vào app hiện tại.",
                        "Restore individual wallets into the current app."
                    ),
                    12
                )
                .style(text_secondary_color()),
                Space::with_height(12),
                row![mode_bip39, mode_slip39, mode_encrypted].spacing(8),
            ]
            .spacing(8);

            if self.import_mode == ImportMode::Encrypted {
                form_content = form_content
                    .push(
                        text_scaled(t(
                            "File .enc này dùng cho mnemonic backup mã hóa, đã chứa tên ví và network. Bạn có thể override tên ví nếu muốn.",
                            "This .enc file is for encrypted mnemonic backups and already contains the wallet name and network. You can optionally override the wallet name.",
                        ), 12)
                        .style(text_secondary_color()),
                    )
                    .push(import_name_input);
            } else {
                form_content = form_content
                    .push(import_name_input)
                    .push(Space::with_height(8))
                    .push(row![network_testnet, network_mainnet].spacing(8));
            }

            match self.import_mode {
                ImportMode::Bip39 => {
                    let import_mnemonic_input = text_input(
                        t(
                            "Mnemonic (12 từ, cách nhau bởi dấu cách)...",
                            "Mnemonic (12 words, separated by spaces)...",
                        ),
                        &self.import_mnemonic,
                    )
                    .on_input(WalletsMessage::ImportMnemonicChanged)
                    .padding(12)
                    .size(14);

                    let import_btn = button(text_scaled(
                        t("Import từ mnemonic", "Import from Mnemonic"),
                        14,
                    ))
                    .on_press(WalletsMessage::ImportWalletFromMnemonic)
                    .padding(10)
                    .style(primary_button_style());

                    form_content = form_content
                        .push(Space::with_height(8))
                        .push(import_mnemonic_input)
                        .push(Space::with_height(12))
                        .push(import_btn);
                }
                ImportMode::Slip39 => {
                    let passphrase_input = text_input(
                        t(
                            "SLIP-0039 passphrase (không bắt buộc)...",
                            "SLIP-0039 passphrase (optional)...",
                        ),
                        &self.import_slip39_passphrase,
                    )
                    .on_input(WalletsMessage::ImportSlip39PassphraseChanged)
                    .secure(true)
                    .padding(12)
                    .size(14);

                    let mut shares_form = column![
                        info_box(
                            t("SLIP-0039 là gì?", "What is SLIP-0039?"),
                            t("SLIP-0039 (Shamir Secret Sharing) chia mnemonic thành nhiều mảnh. Cần tối thiểu K mảnh để khôi phục ví.",
                              "SLIP-0039 (Shamir Secret Sharing) splits mnemonic into multiple shares. At least K shares are needed to restore the wallet.")
                        ).map(|_| WalletsMessage::ImportModeChanged(ImportMode::Slip39)),
                        text_scaled(t(
                            "Nhập tối thiểu K share, mỗi ô là 1 cụm từ SLIP-0039",
                            "Enter at least K shares, each field is one SLIP-0039 phrase",
                        ), 12)
                        .style(text_secondary_color())
                    ]
                    .spacing(8);

                    for (index, share_value) in self.import_slip39_shares.iter().enumerate() {
                        shares_form = shares_form.push(
                            column![
                                text(format!("{} #{}", t("Mảnh", "Share"), index + 1))
                                    .size(12)
                                    .style(text_primary_color()),
                                text_input(
                                    t("Từ của SLIP-0039 share...", "SLIP-0039 share words..."),
                                    share_value
                                )
                                .on_input(move |input| {
                                    WalletsMessage::ImportSlip39ShareChanged(index, input)
                                })
                                .padding(10)
                                .size(13),
                            ]
                            .spacing(4),
                        );
                    }

                    let add_share_btn = button(text_scaled(t("+ Thêm share", "+ Add share"), 13))
                        .on_press(WalletsMessage::AddImportSlip39Share)
                        .padding(8)
                        .style(secondary_button_style());

                    let remove_share_btn =
                        button(text_scaled(t("- Bớt share", "- Remove share"), 13))
                            .on_press(WalletsMessage::RemoveImportSlip39Share)
                            .padding(8)
                            .style(secondary_button_style());

                    let import_btn = button(text_scaled(
                        t("Import từ SLIP-0039", "Import from SLIP-0039"),
                        14,
                    ))
                    .on_press(WalletsMessage::ImportWalletFromSlip39)
                    .padding(10)
                    .style(primary_button_style());

                    form_content = form_content
                        .push(Space::with_height(8))
                        .push(passphrase_input)
                        .push(
                            container(shares_form)
                                .style(card_style())
                                .padding(12)
                                .width(Length::Fill),
                        )
                        .push(row![add_share_btn, remove_share_btn].spacing(8))
                        .push(Space::with_height(6))
                        .push(import_btn);
                }
                ImportMode::Encrypted => {
                    let browse_button = button(
                        text(t(
                            "Chọn backup mnemonic mã hóa (.enc)",
                            "Choose encrypted mnemonic backup (.enc)",
                        ))
                        .size(13),
                    )
                    .on_press(WalletsMessage::BrowseImportEncryptedPath)
                    .padding(10)
                    .style(secondary_button_style());

                    let encrypted_path_input = text_input(
                        t(
                            "Đường dẫn file mnemonic .enc...",
                            "Path to encrypted mnemonic .enc file...",
                        ),
                        &self.import_encrypted_path,
                    )
                    .on_input(WalletsMessage::ImportEncryptedPathChanged)
                    .padding(12)
                    .size(14);

                    let passphrase_input = text_input(
                        t(
                            "Passphrase đã dùng khi export mnemonic .enc...",
                            "Passphrase used when exporting the mnemonic .enc file...",
                        ),
                        &self.import_encrypted_passphrase,
                    )
                    .on_input(WalletsMessage::ImportEncryptedPassphraseChanged)
                    .secure(true)
                    .padding(12)
                    .size(14);

                    let import_btn = button(text_scaled(
                        t("Import từ mnemonic .enc", "Import from mnemonic .enc"),
                        14,
                    ))
                    .on_press(WalletsMessage::ImportWalletFromEncrypted)
                    .padding(10)
                    .style(primary_button_style());

                    form_content = form_content
                        .push(Space::with_height(8))
                        .push(browse_button)
                        .push(encrypted_path_input)
                        .push(passphrase_input)
                        .push(
                            text_scaled(t(
                                "Nếu để trống tên ví override, app sẽ dùng tên ví lưu trong mnemonic backup mã hóa.",
                                "If the wallet name override is empty, the app will use the wallet name stored in the encrypted mnemonic backup.",
                            ), 12)
                            .style(text_secondary_color()),
                        )
                        .push(Space::with_height(6))
                        .push(import_btn);
                }
            }

            let form = container(form_content)
                .style(card_style())
                .padding(20)
                .width(Length::Fill);

            scroll_content = scroll_content.push(form);
        }

        if !wallets.is_empty() {
            // Sorting controls (inside a card to match wallet list)
            let sort_controls = container(
                row![
                    text(t("Sắp xếp theo:", "Sort by:"))
                        .size(14)
                        .style(text_secondary_color()),
                    Space::with_width(12),
                    pick_list(
                        vec![
                            WalletSortField::Balance,
                            WalletSortField::Name,
                            WalletSortField::Created,
                            WalletSortField::Network,
                        ],
                        Some(self.sort_field),
                        WalletsMessage::SortFieldChanged
                    )
                    .padding(8)
                    .style(pick_list_style())
                    .menu_style(pick_list_menu_style()),
                    Space::with_width(Length::Fill),
                    button(
                        text(if self.sort_ascending {
                            Bootstrap::ArrowUp.to_string()
                        } else {
                            Bootstrap::ArrowDown.to_string()
                        })
                        .size(12)
                        .font(BOOTSTRAP_FONT),
                    )
                    .on_press(WalletsMessage::ToggleSortDirection)
                    .padding([8, 12])
                    .style(secondary_button_style()),
                ]
                .align_y(Alignment::Center),
            )
            .style(card_style())
            .padding(16)
            .width(Length::Fill);

            scroll_content = scroll_content.push(sort_controls);
            scroll_content = scroll_content.push(Space::with_height(12));

            // Prepare sorted indices
            let mut sorted_indices: Vec<usize> = (0..wallets.len()).collect();
            sorted_indices.sort_by(|&a, &b| {
                let wa = &wallets[a];
                let wb = &wallets[b];
                let ord = match self.sort_field {
                    WalletSortField::Balance => wa.balance().cmp(&wb.balance()),
                    WalletSortField::Name => wa.name.to_lowercase().cmp(&wb.name.to_lowercase()),
                    WalletSortField::Created => a.cmp(&b), // Use index as proxy for created time
                    WalletSortField::Network => wa.network.as_str().cmp(wb.network.as_str()),
                };
                if self.sort_ascending {
                    ord
                } else {
                    ord.reverse()
                }
            });

            let mut wallet_list = column![];

            for sorted_index in sorted_indices {
                let wallet = &wallets[sorted_index];
                let is_selected = sorted_index == selected;
                let needs_backup = wallet.has_mnemonic && !wallet.mnemonic_backed_up;
                let balance_btc = wallet.balance() as f64 / 100_000_000.0;
                let balance_usd = btc_price_usd.map(|p| balance_btc * p);

                // Tag button to open tag modal
                let tag_btn = button(
                    text(Bootstrap::Tags.to_string())
                        .size(14)
                        .font(BOOTSTRAP_FONT)
                        .style(text_color(Colors::ACCENT_PURPLE)),
                )
                .on_press(WalletsMessage::ToggleTagModal(sorted_index))
                .padding([4, 6])
                .style(secondary_button_style());

                // Render existing tags as small text badges
                let mut tags_display = row![].spacing(4).align_y(Alignment::Center);

                for (i, tag) in wallet.tags.iter().enumerate().take(3) {
                    let color = crate::ui::components::tag_picker::TAG_COLORS
                        [i % crate::ui::components::tag_picker::TAG_COLORS.len()];
                    let tag_text = if tag.len() > 8 {
                        format!("{}...", &tag[..8])
                    } else {
                        tag.clone()
                    };

                    tags_display = tags_display.push(
                        container(text(tag_text).size(9).style(text_primary_color()))
                            .padding([2, 6])
                            .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                                background: Some(iced::Background::Color(Color {
                                    r: color.r,
                                    g: color.g,
                                    b: color.b,
                                    a: 0.85,
                                })),
                                border: iced::Border {
                                    radius: 4.0.into(),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                },
                                ..Default::default()
                            }),
                    );
                }
                if wallet.tags.len() > 3 {
                    let remaining_count = wallet.tags.len() - 3;
                    let remaining_tags: Vec<_> = wallet.tags.iter().skip(3).cloned().collect();
                    let tooltip_content = remaining_tags.join(", ");

                    let plus_indicator = text(format!("+{}", remaining_count))
                        .size(10)
                        .style(text_muted_color());

                    tags_display = tags_display.push(tooltip(
                        plus_indicator,
                        text(tooltip_content).size(12).style(text_secondary_color()),
                        tooltip::Position::Top,
                    ));
                }

                let select_btn = button(
                    row![
                        column![
                            row![
                                text_scaled(wallet.name.as_str(), 16).style(text_primary_color()),
                                Space::with_width(8),
                                tags_display,
                            ]
                            .align_y(Alignment::Center),
                            text(format!(
                                "{} | {:.8} BTC{} | {}",
                                wallet.network.as_str(),
                                balance_btc,
                                balance_usd
                                    .map(|u| format!(" | ${:.2} USD", u))
                                    .unwrap_or_default(),
                                backup_status_text(wallet.has_mnemonic, wallet.mnemonic_backed_up)
                            ))
                            .size(12)
                            .style(text_secondary_color()),
                        ]
                        .spacing(4),
                        Space::with_width(Length::Fill),
                        // Selection checkmark
                        if is_selected {
                            text_scaled(iced_fonts::Bootstrap::Check.to_string(), 16)
                                .font(iced_fonts::BOOTSTRAP_FONT)
                                .style(text_color(Colors::SUCCESS))
                        } else {
                            text("").size(16).font(iced_fonts::BOOTSTRAP_FONT)
                        },
                        Space::with_width(4),
                        // Backup status icon
                        if needs_backup {
                            button(
                                text_scaled(iced_fonts::Bootstrap::Exclamation.to_string(), 16)
                                    .font(iced_fonts::BOOTSTRAP_FONT)
                                    .style(text_color(Colors::WARNING)),
                            )
                            .on_press(WalletsMessage::ShowBackupWarning(sorted_index))
                            .padding(0)
                            .style(crate::ui::theme::flat_icon_button_style())
                        } else {
                            button(
                                text_scaled(iced_fonts::Bootstrap::ShieldCheck.to_string(), 16)
                                    .font(iced_fonts::BOOTSTRAP_FONT)
                                    .style(text_color(Colors::SUCCESS)),
                            )
                            .padding(0)
                            .style(crate::ui::theme::flat_icon_button_style())
                        },
                    ]
                    .align_y(Alignment::Center),
                )
                .on_press(WalletsMessage::SelectWallet(sorted_index))
                .padding(12)
                .style(if is_selected {
                    selected_button_style()
                } else {
                    secondary_button_style()
                });

                let delete_btn = button(text_scaled(t("Xóa", "Delete"), 12))
                    .on_press(WalletsMessage::DeleteWallet(sorted_index))
                    .padding([6, 10])
                    .style(danger_button_style());

                wallet_list = wallet_list.push(
                    container(
                        row![
                            select_btn,
                            Space::with_width(8),
                            tag_btn,
                            Space::with_width(4),
                            delete_btn
                        ]
                        .align_y(Alignment::Center),
                    )
                    .style(card_style())
                    .padding(12),
                );
                wallet_list = wallet_list.push(Space::with_height(8));
            }

            scroll_content = scroll_content.push(
                container(
                    column![
                        text_scaled(t("Danh sách ví", "Your Wallets"), 18)
                            .style(text_primary_color()),
                        Space::with_height(12),
                        wallet_list,
                    ]
                    .spacing(0),
                )
                .style(card_style())
                .padding(16)
                .width(Length::Fill),
            );

            if let Some(selected_wallet) = wallets.get(selected) {
                scroll_content = scroll_content.push(Space::with_height(12));
                scroll_content = scroll_content.push(wallet_summary_card(selected_wallet));
                scroll_content = scroll_content.push(Space::with_height(12));
                scroll_content = scroll_content.push(
                    container(
                        column![
                            text_scaled(t("Danh sách địa chỉ", "Addresses"), 18)
                                .style(text_primary_color()),
                            Space::with_height(12),
                            wallet_addresses_detail(
                                selected_wallet,
                                self.copied_address.as_deref(),
                                self.show_external_addresses,
                                self.show_internal_addresses,
                            ),
                        ]
                        .spacing(0),
                    )
                    .style(card_style())
                    .padding(16)
                    .width(Length::Fill),
                );
                scroll_content = scroll_content.push(Space::with_height(12));
                scroll_content = scroll_content.push(self.backup_panel(
                    selected,
                    selected_wallet,
                    revealed_mnemonic,
                ));
            }
        } else if !self.show_create_form && !self.show_import_mnemonic_form {
            scroll_content = scroll_content.push(
                container(
                    text(t(
                        "Chưa có ví nào. Hãy tạo ví đầu tiên!",
                        "No wallets yet. Create your first wallet!",
                    ))
                    .size(16)
                    .style(text_secondary_color()),
                )
                .padding(40)
                .center_x(Length::Fill),
            );
        }

        let base_content: Element<'_, WalletsMessage> =
            column![header, scrollable(scroll_content).width(Length::Fill),]
                .spacing(0)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();

        if let Some(index) = self.confirm_delete_index {
            let wallet_name = wallets
                .get(index)
                .map(|wallet| wallet.name.clone())
                .unwrap_or_default();

            let delete_content = column![
                text(format!("{} '{wallet_name}'?", t("Xóa ví", "Delete wallet")))
                    .size(16)
                    .style(text_primary_color()),
                Space::with_height(12),
                text_input(
                    t(
                        "Nhập passphrase để xác nhận...",
                        "Enter passphrase to confirm..."
                    ),
                    &self.delete_passphrase,
                )
                .on_input(WalletsMessage::DeletePassphraseChanged)
                .on_submit(WalletsMessage::ConfirmDelete(index))
                .secure(true)
                .padding(12)
                .size(14),
                Space::with_height(16),
                container(
                    row![
                        button(text_scaled(t("Hủy", "Cancel"), 14))
                            .on_press(WalletsMessage::CancelDelete)
                            .padding(10)
                            .style(secondary_button_style()),
                        Space::with_width(12),
                        button(text_scaled(t("Xóa", "Delete"), 14))
                            .on_press(WalletsMessage::ConfirmDelete(index))
                            .padding(10)
                            .style(danger_button_style()),
                    ]
                    .spacing(8),
                )
                .width(Length::Fill)
                .align_x(Alignment::Center),
            ]
            .spacing(0);

            return modal(
                base_content,
                t("Xác nhận xóa", "Confirm Delete"),
                delete_content.into(),
                WalletsMessage::CancelDelete,
                compact_mode,
                true, // close_on_backdrop: true
            );
        }

        // Tag Management Modal
        if let Some(tag_index) = self.tag_modal_index {
            if let Some(wallet) = wallets.get(tag_index) {
                let tag_modal_content =
                    tag_picker(&wallet.tags, &self.tag_input).map(WalletsMessage::TagMessage);

                return modal(
                    base_content,
                    t("Quản lý Tag", "Manage Tags"),
                    tag_modal_content,
                    WalletsMessage::ToggleTagModal(tag_index),
                    compact_mode,
                    false, // close_on_backdrop: false (chỉ đóng khi bấm X)
                );
            }
        }

        base_content
    }

    fn backup_panel<'a>(
        &'a self,
        selected_index: usize,
        wallet: &'a Wallet,
        revealed_mnemonic: Option<&'a str>,
    ) -> Element<'a, WalletsMessage> {
        let needs_backup = wallet.has_mnemonic && !wallet.mnemonic_backed_up;

        let mut panel = column![
            text_scaled(t("Backup Center", "Backup Center"), 18).style(text_primary_color()),
        ]
        .spacing(8);

        if self.notice_wallet_index == Some(selected_index) && needs_backup {
            panel = panel.push(
                container(
                    text_scaled(t(
                        "Ví này chưa backup mnemonic. Hãy hoàn thành 3 bước bên dưới càng sớm càng tốt.",
                        "This wallet has not backed up its mnemonic. Please complete the 3 steps below as soon as possible.",
                    ), 13)
                    .style(text_primary_color()),
                )
                .style(notice_style(NoticeTone::Warning))
                .padding(12)
                .width(Length::Fill),
            );
        }

        if !wallet.has_mnemonic {
            panel = panel.push(
                text(t(
                    "Wallet này không có mnemonic (ví import từ xprv).",
                    "This wallet has no mnemonic (imported from xprv).",
                ))
                .size(13)
                .style(text_secondary_color()),
            );
        } else if self.revealed_wallet_index != Some(selected_index) {
            let reveal_button_label = if wallet.mnemonic_backed_up {
                t("Hiển thị mnemonic", "Show mnemonic")
            } else {
                t(
                    "Hiện mnemonic và tiếp tục backup",
                    "Show mnemonic and continue backup",
                )
            };

            panel = panel
                .push(
                    text_scaled(t("Bước 1. Mở mnemonic", "Step 1. Reveal the mnemonic"), 14)
                        .style(text_primary_color()),
                )
                .push(
                    text_scaled(
                        t(
                            "Nhập passphrase hiện tại để xem mnemonic",
                            "Enter your current passphrase to view mnemonic",
                        ),
                        13,
                    )
                    .style(text_secondary_color()),
                )
                .push(
                    text_input(
                        t("Passphrase...", "Passphrase..."),
                        &self.mnemonic_passphrase,
                    )
                    .on_input(WalletsMessage::MnemonicPassphraseChanged)
                    .secure(true)
                    .padding(10)
                    .size(13),
                )
                .push(
                    button(text_scaled(reveal_button_label, 13))
                        .on_press(WalletsMessage::RevealMnemonic(selected_index))
                        .padding(10)
                        .style(primary_button_style()),
                );
        } else if let Some(mnemonic) = revealed_mnemonic {
            let words: Vec<&str> = mnemonic.split_whitespace().collect();
            let mnemonic_line = words.join(" ");
            let word_count = wallet.mnemonic_word_count.unwrap_or(words.len());
            let test_active = self.backup_test_wallet_index == Some(selected_index);

            if test_active {
                panel = panel.push(
                    container(
                        text_scaled(
                            t(
                                "Mnemonic đang được ẩn khi làm bài test backup.",
                                "Mnemonic is hidden while backup test is active.",
                            ),
                            12,
                        )
                        .style(text_primary_color()),
                    )
                    .style(notice_style(NoticeTone::Warning))
                    .padding(10)
                    .width(Length::Fill),
                );
            } else {
                panel = panel.push(
                    container(
                        column![
                            text(format!(
                                "{} ({word_count} {})",
                                t("Mnemonic", "Mnemonic"),
                                t("từ", "words")
                            ))
                            .size(12)
                            .style(text_secondary_color()),
                            Space::with_height(6),
                            text_scaled(mnemonic_line, 14).style(text_color(Colors::ACCENT_TEAL)),
                        ]
                        .spacing(2),
                    )
                    .style(card_style())
                    .padding(12)
                    .width(Length::Fill),
                );

                panel = panel.push(
                    text_scaled(
                        t("Bước 2. Lưu backup an toàn", "Step 2. Save a safe backup"),
                        14,
                    )
                    .style(text_primary_color()),
                );

                panel = panel.push(
                    button(text_scaled(
                        t("Export mnemonic ra PDF", "Export mnemonic to PDF"),
                        13,
                    ))
                    .on_press(WalletsMessage::ExportMnemonicPdf(selected_index))
                    .padding(10)
                    .style(secondary_button_style()),
                );

                panel = panel.push(
                    container(
                        column![
                            text_scaled(
                                t(
                                    "Export mã hóa dùng passphrase ứng dụng hiện tại.",
                                    "Encrypted export uses the current app passphrase.",
                                ),
                                12
                            )
                            .style(text_secondary_color()),
                            button(
                                text(t(
                                    "Export mnemonic mã hóa (.enc)",
                                    "Export encrypted mnemonic (.enc)",
                                ))
                                .size(13),
                            )
                            .on_press(WalletsMessage::ExportMnemonicEncrypted(selected_index))
                            .padding(10)
                            .style(primary_button_style()),
                        ]
                        .spacing(8),
                    )
                    .style(card_style())
                    .padding(12)
                    .width(Length::Fill),
                );

                let slip39_threshold_input = text_input("K", &self.slip39_export_threshold)
                    .on_input(WalletsMessage::Slip39ExportThresholdChanged)
                    .padding(8)
                    .size(13)
                    .width(Length::Fixed(100.0))
                    .style(input_style());

                let slip39_share_count_input = text_input("N", &self.slip39_export_share_count)
                    .on_input(WalletsMessage::Slip39ExportShareCountChanged)
                    .padding(8)
                    .size(13)
                    .width(Length::Fixed(100.0))
                    .style(input_style());

                let slip39_passphrase_input = text_input(
                    t(
                        "SLIP-0039 passphrase (không bắt buộc)...",
                        "SLIP-0039 passphrase (optional)...",
                    ),
                    &self.slip39_export_passphrase,
                )
                .on_input(WalletsMessage::Slip39ExportPassphraseChanged)
                .secure(true)
                .padding(10)
                .size(13)
                .style(input_style());

                panel =
                    panel.push(
                        container(
                            column![
                            text_scaled(t("Backup tách mảnh SLIP-0039", "SLIP-0039 split backup"), 13)
                                .style(text_primary_color()),
                            text(t(
                                "Cấu hình K/N (ví dụ 2/3) để tách mnemonic thành nhiều share.",
                                "Configure K/N (e.g. 2/3) to split mnemonic into multiple shares.",
                            ))
                            .size(12)
                            .style(text_secondary_color()),
                            row![
                                column![
                                    text_scaled(t("Ngưỡng K", "Threshold K"), 12)
                                        .style(text_secondary_color()),
                                    slip39_threshold_input,
                                ]
                                .spacing(4),
                                column![
                                    text_scaled(t("Tổng share N", "Total share N"), 12)
                                        .style(text_secondary_color()),
                                    slip39_share_count_input,
                                ]
                                .spacing(4),
                            ]
                            .spacing(10),
                            slip39_passphrase_input,
                            button(text(t(
                                "Export SLIP-0039 shares (thư mục PDF)",
                                "Export SLIP-0039 shares (PDF folder)",
                            ))
                            .size(13))
                            .on_press(WalletsMessage::ExportSlip39Shares(selected_index))
                            .padding(10)
                            .style(secondary_button_style()),
                            text_scaled(t(
                                "Nếu cần backup tổng toàn ứng dụng, hãy dùng Encrypted App Backup trong Settings thay vì gói shares vào .enc.",
                                "If you need a full app backup, use Encrypted App Backup in Settings instead of packing the shares into .enc.",
                            ), 12)
                            .style(text_secondary_color()),
                        ]
                            .spacing(8),
                        )
                        .style(card_style())
                        .padding(12)
                        .width(Length::Fill),
                    );
            }

            if wallet.mnemonic_backed_up {
                panel = panel.push(
                    text_scaled(
                        t("Backup mnemonic: Đã xác minh", "Mnemonic backup: Verified"),
                        13,
                    )
                    .style(text_color(Colors::SUCCESS)),
                );
            } else {
                panel = panel.push(
                    text_scaled(
                        t("Bước 3. Xác minh backup", "Step 3. Verify the backup"),
                        14,
                    )
                    .style(text_primary_color()),
                );
                panel = panel.push(
                    button(text_scaled(
                        if test_active {
                            t("Hủy bài test backup", "Cancel backup test")
                        } else {
                            t("Bắt đầu bài test backup", "Start backup test")
                        },
                        13,
                    ))
                    .on_press(WalletsMessage::ToggleBackupTest {
                        wallet_index: selected_index,
                        word_count: wallet.mnemonic_word_count.unwrap_or(12),
                    })
                    .padding(10)
                    .style(secondary_button_style()),
                );

                if test_active {
                    let mut test_form = column![text_scaled(
                        t(
                            "Nhập đúng các từ theo vị trí để xác nhận backup",
                            "Enter the correct words at positions to verify backup",
                        ),
                        12
                    )
                    .style(text_secondary_color()),]
                    .spacing(8);

                    for (field_index, position) in
                        self.backup_test_positions.iter().copied().enumerate()
                    {
                        let value = self
                            .backup_test_answers
                            .get(field_index)
                            .map(String::as_str)
                            .unwrap_or("");

                        test_form = test_form.push(
                            column![
                                text(format!("{} #{}", t("Từ", "Word"), position))
                                    .size(12)
                                    .style(text_primary_color()),
                                text_input(
                                    t("Nhập từ mnemonic...", "Enter mnemonic word..."),
                                    value
                                )
                                .on_input(move |input| {
                                    WalletsMessage::BackupWordChanged(field_index, input)
                                })
                                .padding(10)
                                .size(13),
                            ]
                            .spacing(4),
                        );
                    }

                    test_form = test_form.push(
                        button(text_scaled(t("Xác nhận đã backup", "Confirm backup"), 13))
                            .on_press(WalletsMessage::SubmitBackupTest(selected_index))
                            .padding(10)
                            .style(primary_button_style()),
                    );

                    panel = panel.push(
                        container(test_form)
                            .style(card_style())
                            .padding(12)
                            .width(Length::Fill),
                    );
                }
            }
        } else {
            panel = panel.push(
                text_scaled(
                    t(
                        "Mnemonic đã được mở nhưng không thể nạp từ vault hiện tại.",
                        "Mnemonic was unlocked but could not be loaded from the current vault.",
                    ),
                    13,
                )
                .style(text_color(Colors::ERROR)),
            );
        }

        container(panel)
            .style(card_style())
            .padding(16)
            .width(Length::Fill)
            .into()
    }
}

fn test_positions(word_count: usize) -> Vec<usize> {
    if word_count == 0 {
        return Vec::new();
    }

    let mut positions = vec![1, word_count.div_ceil(2).max(1), word_count];
    positions.retain(|position| *position <= word_count && *position > 0);
    positions.sort_unstable();
    positions.dedup();

    let target = word_count.min(3);
    if positions.len() < target {
        for position in 1..=word_count {
            if !positions.contains(&position) {
                positions.push(position);
                if positions.len() == target {
                    break;
                }
            }
        }
    }

    positions.sort_unstable();
    positions
}

fn wallet_summary_card<'a>(wallet: &'a Wallet) -> Element<'a, WalletsMessage> {
    let balance_btc = wallet.balance() as f64 / 100_000_000.0;
    let external_count = wallet
        .addresses
        .iter()
        .filter(|a| a.chain == AddressChain::External)
        .count();
    let internal_count = wallet
        .addresses
        .iter()
        .filter(|a| a.chain == AddressChain::Internal)
        .count();
    let history_count = wallet.history.len();

    container(
        column![
            text_scaled(t("Wallet Summary", "Wallet Summary"), 18).style(text_primary_color()),
            Space::with_height(10),
            row![
                summary_metric(t("Tên ví", "Wallet"), wallet.name.clone()),
                summary_metric(
                    t("Trạng thái backup", "Backup status"),
                    backup_status_text(wallet.has_mnemonic, wallet.mnemonic_backed_up).to_string(),
                ),
            ]
            .spacing(12),
            Space::with_height(10),
            row![
                summary_metric(t("Mạng", "Network"), wallet.network.as_str().to_string()),
                summary_metric(t("Số dư", "Balance"), format!("{balance_btc:.8} BTC")),
            ]
            .spacing(12),
            Space::with_height(10),
            row![
                summary_metric(
                    t("Tổng địa chỉ", "Total Addresses"),
                    format!(
                        "{} ({} {}, {} {})",
                        external_count + internal_count,
                        external_count,
                        t("nhận", "receive"),
                        internal_count,
                        t("thay đổi", "change")
                    )
                ),
                summary_metric(t("Giao dịch", "Transactions"), history_count.to_string()),
            ]
            .spacing(12),
        ]
        .spacing(0),
    )
    .style(card_style())
    .padding(16)
    .width(Length::Fill)
    .into()
}

fn wallet_addresses_detail<'a>(
    wallet: &'a Wallet,
    copied_address: Option<&'a str>,
    show_external: bool,
    show_internal: bool,
) -> Element<'a, WalletsMessage> {
    let external_addresses = wallet
        .addresses
        .iter()
        .filter(|a| a.chain == AddressChain::External)
        .collect::<Vec<_>>();

    let internal_addresses = wallet
        .addresses
        .iter()
        .filter(|a| a.chain == AddressChain::Internal)
        .collect::<Vec<_>>();

    let mut col = column![];

    // External addresses section container
    let mut external_section = column![];

    // External addresses toggle button
    external_section = external_section.push(
        mouse_area(
            container(
                row![
                    text(format!(
                        "{}{}({})",
                        if show_external {
                            t("Ẩn", "Hide")
                        } else {
                            t("Hiện", "Show")
                        },
                        t(" địa chỉ nhận tiền ", " Receiving Addresses "),
                        external_addresses.len()
                    ))
                    .size(12)
                    .style(text_secondary_color()),
                    Space::with_width(Length::Fill)
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .center_x(Length::Fill),
        )
        .on_press(WalletsMessage::ToggleExternalAddresses),
    );

    // External addresses list
    if show_external {
        external_section = external_section.push(Space::with_height(8));
        if external_addresses.is_empty() {
            external_section = external_section.push(
                text_scaled(t("Chưa có địa chỉ nhận", "No receiving addresses"), 12)
                    .style(text_muted_color()),
            );
        } else {
            let mut external_list = column![];
            for (i, addr) in external_addresses.iter().enumerate() {
                let is_copied = copied_address == Some(addr.address.as_str());

                let row_content = row![
                    text_scaled(format!("#{}", addr.index), 12).style(text_muted_color()),
                    Space::with_width(8),
                    text_scaled(addr.address.clone(), 12).style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    if is_copied {
                        text_scaled(t("Đã sao chép", "Copied"), 11)
                            .style(text_color(Colors::SUCCESS))
                    } else {
                        text("")
                    },
                ]
                .align_y(Alignment::Center);

                external_list = external_list.push(
                    button(container(row_content).width(Length::Fill))
                        .on_press(WalletsMessage::CopyAddress(addr.address.clone()))
                        .padding(8)
                        .style(secondary_button_style())
                        .width(Length::Fill),
                );

                if i < external_addresses.len() - 1 {
                    external_list = external_list.push(Space::with_height(4));
                }
            }
            external_section =
                external_section.push(scrollable(external_list).height(Length::Shrink));
        }
    }

    let external_container_height = if show_external {
        Length::Shrink
    } else {
        Length::Fixed(40.0)
    };

    let external_column = if show_external {
        column![external_section.spacing(0)].width(Length::Fill)
    } else {
        column![
            Space::with_height(Length::Fill),
            external_section.spacing(0),
            Space::with_height(Length::Fill)
        ]
        .width(Length::Fill)
        .height(external_container_height)
    };

    col = col.push(
        container(external_column)
            .width(Length::Fill)
            .height(external_container_height)
            .center_x(Length::Fill)
            .style(|theme: &iced::Theme| {
                let colors = get_theme_colors(theme);
                container::Style {
                    background: Some(Background::Color(colors.bg_card)),
                    border: iced::Border {
                        color: colors.border_subtle,
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    text_color: None,
                    shadow: iced::Shadow::default(),
                }
            })
            .padding(12),
    );
    col = col.push(Space::with_height(12));

    // Internal addresses section container
    let mut internal_section = column![];

    // Internal addresses toggle button
    internal_section = internal_section.push(
        mouse_area(
            container(
                row![
                    text(format!(
                        "{}{}({})",
                        if show_internal {
                            t("Ẩn", "Hide")
                        } else {
                            t("Hiện", "Show")
                        },
                        t(" địa chỉ thay đổi ", " Change Addresses "),
                        internal_addresses.len()
                    ))
                    .size(12)
                    .style(text_secondary_color()),
                    Space::with_width(Length::Fill)
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .center_x(Length::Fill),
        )
        .on_press(WalletsMessage::ToggleInternalAddresses),
    );

    // Internal addresses list
    if show_internal {
        internal_section = internal_section.push(Space::with_height(8));
        if internal_addresses.is_empty() {
            internal_section = internal_section.push(
                text_scaled(t("Chưa có địa chỉ đổi", "No change addresses"), 12)
                    .style(text_muted_color()),
            );
        } else {
            let mut internal_list = column![];
            for (i, addr) in internal_addresses.iter().enumerate() {
                let is_copied = copied_address == Some(addr.address.as_str());

                let row_content = row![
                    text_scaled(format!("#{}", addr.index), 12).style(text_muted_color()),
                    Space::with_width(8),
                    text_scaled(addr.address.clone(), 12).style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    if is_copied {
                        text_scaled(t("Đã sao chép", "Copied"), 11)
                            .style(text_color(Colors::SUCCESS))
                    } else {
                        text("")
                    },
                ]
                .align_y(Alignment::Center);

                internal_list = internal_list.push(
                    button(container(row_content).width(Length::Fill))
                        .on_press(WalletsMessage::CopyAddress(addr.address.clone()))
                        .padding(8)
                        .style(secondary_button_style())
                        .width(Length::Fill),
                );

                if i < internal_addresses.len() - 1 {
                    internal_list = internal_list.push(Space::with_height(4));
                }
            }
            internal_section =
                internal_section.push(scrollable(internal_list).height(Length::Shrink));
        }
    }

    let internal_container_height = if show_internal {
        Length::Shrink
    } else {
        Length::Fixed(40.0)
    };

    let internal_column = if show_internal {
        column![internal_section.spacing(0)].width(Length::Fill)
    } else {
        column![
            Space::with_height(Length::Fill),
            internal_section.spacing(0),
            Space::with_height(Length::Fill)
        ]
        .width(Length::Fill)
        .height(internal_container_height)
    };

    col = col.push(
        container(internal_column)
            .width(Length::Fill)
            .height(internal_container_height)
            .center_x(Length::Fill)
            .style(|theme: &iced::Theme| {
                let colors = get_theme_colors(theme);
                container::Style {
                    background: Some(Background::Color(colors.bg_card)),
                    border: iced::Border {
                        color: colors.border_subtle,
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    text_color: None,
                    shadow: iced::Shadow::default(),
                }
            })
            .padding(12),
    );

    col.into()
}

fn summary_metric<'a>(label: &'a str, value: String) -> Element<'a, WalletsMessage> {
    container(
        column![
            text_scaled(label, 12).style(text_secondary_color()),
            Space::with_height(4),
            text_scaled(value, 14).style(text_primary_color()),
        ]
        .spacing(0),
    )
    .style(card_style())
    .padding(12)
    .width(Length::Fill)
    .into()
}

fn backup_status_text(has_mnemonic: bool, mnemonic_backed_up: bool) -> &'static str {
    if !has_mnemonic {
        t("Không có seed", "No seed")
    } else if mnemonic_backed_up {
        t("Đã xác minh", "Verified")
    } else {
        t("Chưa backup", "Not backed up")
    }
}

fn parse_u8_field(raw: &str, field_name_vi: &str, field_name_en: &str) -> Result<u8, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "{} {}",
            t("Vui lòng nhập", "Please enter"),
            t(field_name_vi, field_name_en)
        ));
    }

    trimmed.parse::<u8>().map_err(|_| {
        format!(
            "{} {}",
            t(field_name_vi, field_name_en),
            t(
                "phải là số nguyên từ 0 đến 255",
                "must be an integer from 0 to 255"
            )
        )
    })
}

fn optional_trimmed_value(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        optional_trimmed_value, parse_u8_field, test_positions, ImportMode, WalletsEvent,
        WalletsMessage, WalletsView,
    };

    #[test]
    fn backup_test_positions_cover_first_middle_last() {
        assert_eq!(test_positions(12), vec![1, 6, 12]);
        assert_eq!(test_positions(2), vec![1, 2]);
    }

    #[test]
    fn parse_u8_field_rejects_blank_input() {
        assert!(parse_u8_field("", "Ngưỡng", "Threshold").is_err());
        assert!(matches!(parse_u8_field("7", "Ngưỡng", "Threshold"), Ok(7)));
    }

    #[test]
    fn optional_trimmed_value_drops_blank_text() {
        assert_eq!(optional_trimmed_value("  "), None);
        assert_eq!(
            optional_trimmed_value("  Savings  "),
            Some("Savings".to_string())
        );
    }

    #[test]
    fn encrypted_import_requires_path_and_passphrase() {
        let mut view = WalletsView::new();
        view.import_mode = ImportMode::Encrypted;

        assert!(view
            .update(WalletsMessage::ImportWalletFromEncrypted)
            .is_none());

        view.import_encrypted_path = "backup.enc".to_string();
        assert!(view
            .update(WalletsMessage::ImportWalletFromEncrypted)
            .is_none());

        view.import_encrypted_passphrase = "secret".to_string();
        view.import_name = "  Cold Wallet  ".to_string();

        let event = view.update(WalletsMessage::ImportWalletFromEncrypted);
        assert!(matches!(
            event,
            Some(WalletsEvent::ImportWalletFromEncrypted {
                path,
                passphrase,
                name_override
            }) if path == "backup.enc"
                && passphrase == "secret"
                && name_override == Some("Cold Wallet".to_string())
        ));
    }
}
