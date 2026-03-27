use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Alignment, Element, Length,
};

use crate::theme::{
    card_style, danger_button_style, primary_button_style, secondary_button_style, text_color,
    Colors,
};
use crate::wallet::{WalletEntry, WalletNetwork};

#[derive(Debug, Clone)]
pub enum WalletsMessage {
    ToggleCreateForm,
    CreateWallet,
    NameChanged(String),
    NetworkChanged(WalletNetwork),
    ToggleImportMnemonicForm,
    ImportNameChanged(String),
    ImportNetworkChanged(WalletNetwork),
    ImportMnemonicChanged(String),
    ImportWalletFromMnemonic,
    SelectWallet(usize),
    DeleteWallet(usize),
    ConfirmDelete(usize),
    CancelDelete,
    ShowBackupWarning(usize),
    MnemonicPassphraseChanged(String),
    RevealMnemonic(usize),
    ToggleBackupTest {
        wallet_index: usize,
        word_count: usize,
    },
    ExportMnemonicPdf(usize),
    BackupWordChanged(usize, String),
    SubmitBackupTest(usize),
    DismissWalletNotice,
}

pub struct WalletsView {
    create_name: String,
    create_network: WalletNetwork,
    show_create_form: bool,
    import_name: String,
    import_network: WalletNetwork,
    import_mnemonic: String,
    show_import_mnemonic_form: bool,
    confirm_delete_index: Option<usize>,

    notice_wallet_index: Option<usize>,
    mnemonic_passphrase: String,
    revealed_wallet_index: Option<usize>,
    backup_test_wallet_index: Option<usize>,
    backup_test_positions: Vec<usize>,
    backup_test_answers: Vec<String>,
    info: Option<String>,
    error: Option<String>,
}

impl WalletsView {
    pub fn new() -> Self {
        Self {
            create_name: String::new(),
            create_network: WalletNetwork::Testnet,
            show_create_form: false,
            import_name: String::new(),
            import_network: WalletNetwork::Testnet,
            import_mnemonic: String::new(),
            show_import_mnemonic_form: false,
            confirm_delete_index: None,
            notice_wallet_index: None,
            mnemonic_passphrase: String::new(),
            revealed_wallet_index: None,
            backup_test_wallet_index: None,
            backup_test_positions: Vec::new(),
            backup_test_answers: Vec::new(),
            info: None,
            error: None,
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
            "Mnemonic đã hiển thị. Hãy backup an toàn và hoàn thành bài test xác nhận.".to_string(),
        );
        self.error = None;
    }

    pub fn mark_backup_verified(&mut self, wallet_index: usize) {
        self.notice_wallet_index = None;
        self.backup_test_wallet_index = None;
        self.backup_test_positions.clear();
        self.backup_test_answers.clear();
        self.info = Some("Backup mnemonic đã được xác nhận thành công.".to_string());
        self.error = None;
        self.revealed_wallet_index = Some(wallet_index);
    }

    pub fn update(&mut self, message: WalletsMessage) -> Option<crate::app::AppMessage> {
        match message {
            WalletsMessage::ToggleCreateForm => {
                self.show_create_form = !self.show_create_form;
                if self.show_create_form {
                    self.show_import_mnemonic_form = false;
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
                Some(crate::app::AppMessage::CreateWallet(name, network))
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
                }
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
            WalletsMessage::ImportWalletFromMnemonic => {
                if self.import_name.trim().is_empty() {
                    self.error = Some("Vui lòng nhập tên ví import".to_string());
                    return None;
                }
                if self.import_mnemonic.trim().is_empty() {
                    self.error = Some("Vui lòng nhập mnemonic để import".to_string());
                    return None;
                }

                let name = self.import_name.trim().to_string();
                let network = self.import_network;
                let mnemonic = self.import_mnemonic.trim().to_string();

                self.import_name.clear();
                self.import_mnemonic.clear();
                self.show_import_mnemonic_form = false;
                self.error = None;

                Some(crate::app::AppMessage::ImportWalletFromMnemonic {
                    name,
                    network,
                    mnemonic,
                })
            }
            WalletsMessage::SelectWallet(index) => {
                self.revealed_wallet_index = None;
                self.mnemonic_passphrase.clear();
                self.backup_test_wallet_index = None;
                self.backup_test_positions.clear();
                self.backup_test_answers.clear();
                self.show_create_form = false;
                self.show_import_mnemonic_form = false;
                self.error = None;
                Some(crate::app::AppMessage::SelectWallet(index))
            }
            WalletsMessage::DeleteWallet(index) => {
                self.confirm_delete_index = Some(index);
                None
            }
            WalletsMessage::ConfirmDelete(index) => {
                self.confirm_delete_index = None;
                Some(crate::app::AppMessage::DeleteWallet(index))
            }
            WalletsMessage::CancelDelete => {
                self.confirm_delete_index = None;
                None
            }
            WalletsMessage::ShowBackupWarning(index) => {
                self.notice_wallet_index = Some(index);
                self.info = Some(
                    "Ví này chưa backup mnemonic. Hãy mở mnemonic và hoàn thành bài test."
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
                    self.error = Some("Vui lòng nhập passphrase để hiện mnemonic".to_string());
                    return None;
                }

                self.error = None;
                Some(crate::app::AppMessage::RevealMnemonic {
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
                    self.error = Some("Hãy mở mnemonic trước khi export PDF".to_string());
                    return None;
                }
                if self.backup_test_wallet_index == Some(wallet_index) {
                    self.error =
                        Some("Không thể export PDF khi đang làm bài test backup".to_string());
                    return None;
                }
                self.error = None;
                Some(crate::app::AppMessage::ExportMnemonicPdf(wallet_index))
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
                    self.error = Some("Bạn chưa bắt đầu bài test backup cho ví này".to_string());
                    return None;
                }

                if self
                    .backup_test_answers
                    .iter()
                    .any(|word| word.trim().is_empty())
                {
                    self.error = Some("Vui lòng điền đầy đủ các từ trong bài test".to_string());
                    return None;
                }

                let checks = self
                    .backup_test_positions
                    .iter()
                    .copied()
                    .zip(self.backup_test_answers.iter().cloned())
                    .collect::<Vec<_>>();

                Some(crate::app::AppMessage::VerifyMnemonicBackup {
                    wallet_index,
                    checks,
                })
            }
            WalletsMessage::DismissWalletNotice => {
                self.notice_wallet_index = None;
                self.info = None;
                self.error = None;
                None
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        wallets: &'a [WalletEntry],
        selected: usize,
    ) -> Element<'a, WalletsMessage> {
        let title = text("Wallets")
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let create_toggle_btn = button(
            text(if self.show_create_form {
                "Cancel Create"
            } else {
                "+ Create Wallet"
            })
            .size(14),
        )
        .on_press(WalletsMessage::ToggleCreateForm)
        .padding(10)
        .style(if self.show_create_form {
            secondary_button_style()
        } else {
            primary_button_style()
        });

        let import_toggle_btn = button(
            text(if self.show_import_mnemonic_form {
                "Cancel Import"
            } else {
                "+ Import Mnemonic"
            })
            .size(14),
        )
        .on_press(WalletsMessage::ToggleImportMnemonicForm)
        .padding(10)
        .style(if self.show_import_mnemonic_form {
            secondary_button_style()
        } else {
            primary_button_style()
        });

        let mut content = column![
            title,
            Space::with_height(16),
            row![create_toggle_btn, Space::with_width(10), import_toggle_btn]
                .align_y(Alignment::Center)
        ]
        .spacing(16)
        .padding(32);

        if let Some(info) = &self.info {
            content = content.push(
                container(
                    row![
                        text(info.as_str())
                            .size(13)
                            .style(text_color(Colors::WARNING)),
                        Space::with_width(Length::Fill),
                        button(text("x").size(12))
                            .on_press(WalletsMessage::DismissWalletNotice)
                            .padding(4)
                            .style(secondary_button_style()),
                    ]
                    .align_y(Alignment::Center),
                )
                .style(card_style())
                .padding(10)
                .width(Length::Fill),
            );
        }

        if let Some(error) = &self.error {
            content = content.push(
                container(
                    text(error.as_str())
                        .size(13)
                        .style(text_color(Colors::ERROR)),
                )
                .style(card_style())
                .padding(10)
                .width(Length::Fill),
            );
        }

        if self.show_create_form {
            let name_input = text_input("Wallet name...", &self.create_name)
                .on_input(WalletsMessage::NameChanged)
                .padding(12)
                .size(16);

            let network_testnet = button(text("Testnet").size(14))
                .on_press(WalletsMessage::NetworkChanged(WalletNetwork::Testnet))
                .padding(8)
                .style(if self.create_network == WalletNetwork::Testnet {
                    primary_button_style()
                } else {
                    secondary_button_style()
                });

            let network_mainnet = button(text("Mainnet").size(14))
                .on_press(WalletsMessage::NetworkChanged(WalletNetwork::Mainnet))
                .padding(8)
                .style(if self.create_network == WalletNetwork::Mainnet {
                    primary_button_style()
                } else {
                    secondary_button_style()
                });

            let create_btn = button(text("Create").size(14))
                .on_press(WalletsMessage::CreateWallet)
                .padding(10)
                .style(primary_button_style());

            let form = container(
                column![
                    text("Create New Wallet")
                        .size(18)
                        .style(text_color(Colors::TEXT_PRIMARY)),
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

            content = content.push(form);
        }

        if self.show_import_mnemonic_form {
            let import_name_input = text_input("Wallet name...", &self.import_name)
                .on_input(WalletsMessage::ImportNameChanged)
                .padding(12)
                .size(16);

            let import_mnemonic_input = text_input(
                "Mnemonic (12 words, cách nhau bởi dấu cách)...",
                &self.import_mnemonic,
            )
            .on_input(WalletsMessage::ImportMnemonicChanged)
            .padding(12)
            .size(14);

            let network_testnet = button(text("Testnet").size(14))
                .on_press(WalletsMessage::ImportNetworkChanged(WalletNetwork::Testnet))
                .padding(8)
                .style(if self.import_network == WalletNetwork::Testnet {
                    primary_button_style()
                } else {
                    secondary_button_style()
                });

            let network_mainnet = button(text("Mainnet").size(14))
                .on_press(WalletsMessage::ImportNetworkChanged(WalletNetwork::Mainnet))
                .padding(8)
                .style(if self.import_network == WalletNetwork::Mainnet {
                    primary_button_style()
                } else {
                    secondary_button_style()
                });

            let import_btn = button(text("Import from Mnemonic").size(14))
                .on_press(WalletsMessage::ImportWalletFromMnemonic)
                .padding(10)
                .style(primary_button_style());

            let form = container(
                column![
                    text("Import Wallet from Mnemonic")
                        .size(18)
                        .style(text_color(Colors::TEXT_PRIMARY)),
                    Space::with_height(8),
                    text("Ví import mnemonic sẽ được đánh dấu đã backup.")
                        .size(12)
                        .style(text_color(Colors::TEXT_SECONDARY)),
                    Space::with_height(12),
                    import_name_input,
                    Space::with_height(8),
                    row![network_testnet, network_mainnet].spacing(8),
                    Space::with_height(8),
                    import_mnemonic_input,
                    Space::with_height(12),
                    import_btn,
                ]
                .spacing(8),
            )
            .style(card_style())
            .padding(20)
            .width(Length::Fill);

            content = content.push(form);
        }

        if !wallets.is_empty() {
            let mut wallet_list = column![];

            for (index, wallet) in wallets.iter().enumerate() {
                let is_selected = index == selected;
                let needs_backup = wallet.mnemonic.is_some() && !wallet.mnemonic_backed_up;
                let balance: i64 = wallet.history.iter().map(|tx| tx.amount_sat).sum();
                let balance_btc = balance as f64 / 100_000_000.0;

                let select_btn = button(
                    row![
                        column![
                            text(wallet.name.as_str())
                                .size(16)
                                .style(text_color(Colors::TEXT_PRIMARY)),
                            text(format!(
                                "{} | {:.8} BTC",
                                wallet.network.as_str(),
                                balance_btc
                            ))
                            .size(12)
                            .style(text_color(Colors::TEXT_SECONDARY)),
                        ]
                        .spacing(4),
                        Space::with_width(Length::Fill),
                        if is_selected {
                            text("✓").size(20).style(text_color(Colors::SUCCESS))
                        } else {
                            text("")
                        },
                    ]
                    .align_y(Alignment::Center),
                )
                .on_press(WalletsMessage::SelectWallet(index))
                .padding(12)
                .width(Length::Fill)
                .style(if is_selected {
                    primary_button_style()
                } else {
                    secondary_button_style()
                });

                let warning_button: Element<'_, WalletsMessage> = if needs_backup {
                    button(text("!").size(16).style(text_color(Colors::WARNING)))
                        .on_press(WalletsMessage::ShowBackupWarning(index))
                        .padding(8)
                        .style(secondary_button_style())
                        .into()
                } else {
                    Space::with_width(0).into()
                };

                let delete_btn = button(text("🗑").size(16))
                    .on_press(WalletsMessage::DeleteWallet(index))
                    .padding(8)
                    .style(danger_button_style());

                wallet_list = wallet_list.push(
                    container(
                        row![
                            select_btn,
                            Space::with_width(8),
                            warning_button,
                            Space::with_width(8),
                            delete_btn
                        ]
                        .align_y(Alignment::Center),
                    )
                    .style(card_style())
                    .padding(8),
                );
                wallet_list = wallet_list.push(Space::with_height(8));
            }

            content = content.push(column![
                text("Your Wallets")
                    .size(18)
                    .style(text_color(Colors::TEXT_PRIMARY)),
                Space::with_height(12),
                wallet_list,
            ]);

            if let Some(selected_wallet) = wallets.get(selected) {
                content = content.push(Space::with_height(12));
                content = content.push(self.backup_panel(selected, selected_wallet));
            }
        } else if !self.show_create_form && !self.show_import_mnemonic_form {
            content = content.push(
                container(
                    text("No wallets yet. Create your first wallet!")
                        .size(16)
                        .style(text_color(Colors::TEXT_SECONDARY)),
                )
                .padding(40)
                .center_x(Length::Fill),
            );
        }

        if let Some(index) = self.confirm_delete_index {
            let wallet_name = wallets
                .get(index)
                .map(|wallet| wallet.name.clone())
                .unwrap_or_default();

            let dialog = container(
                column![
                    text("Confirm Delete")
                        .size(20)
                        .style(text_color(Colors::ERROR)),
                    Space::with_height(12),
                    text(format!("Delete wallet '{wallet_name}'?"))
                        .size(16)
                        .style(text_color(Colors::TEXT_PRIMARY)),
                    Space::with_height(16),
                    row![
                        button(text("Cancel").size(14))
                            .on_press(WalletsMessage::CancelDelete)
                            .padding(10)
                            .style(secondary_button_style()),
                        Space::with_width(12),
                        button(text("Delete").size(14))
                            .on_press(WalletsMessage::ConfirmDelete(index))
                            .padding(10)
                            .style(danger_button_style()),
                    ],
                ]
                .spacing(8)
                .padding(24),
            )
            .style(card_style())
            .width(Length::Fixed(420.0));

            content = content.push(container(dialog).center_x(Length::Fill).padding(20));
        }

        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn backup_panel<'a>(
        &'a self,
        selected_index: usize,
        wallet: &'a WalletEntry,
    ) -> Element<'a, WalletsMessage> {
        let needs_backup = wallet.mnemonic.is_some() && !wallet.mnemonic_backed_up;

        let mut panel = column![text("Mnemonic Backup")
            .size(18)
            .style(text_color(Colors::TEXT_PRIMARY)),]
        .spacing(8);

        if self.notice_wallet_index == Some(selected_index) && needs_backup {
            panel = panel.push(
                text("! Wallet này chưa backup mnemonic. Vui lòng backup ngay.")
                    .size(13)
                    .style(text_color(Colors::WARNING)),
            );
        }

        match &wallet.mnemonic {
            None => {
                panel = panel.push(
                    text("Wallet này không có mnemonic (ví import từ xprv).")
                        .size(13)
                        .style(text_color(Colors::TEXT_SECONDARY)),
                );
            }
            Some(mnemonic) => {
                if self.revealed_wallet_index != Some(selected_index) {
                    let reveal_button_label = if wallet.mnemonic_backed_up {
                        "Hiển thị mnemonic"
                    } else {
                        "Hiện mnemonic và tiếp tục backup"
                    };

                    panel = panel
                        .push(
                            text("Nhập passphrase hiện tại để xem mnemonic")
                                .size(13)
                                .style(text_color(Colors::TEXT_SECONDARY)),
                        )
                        .push(
                            text_input("Passphrase...", &self.mnemonic_passphrase)
                                .on_input(WalletsMessage::MnemonicPassphraseChanged)
                                .secure(true)
                                .padding(10)
                                .size(13),
                        )
                        .push(
                            button(text(reveal_button_label).size(13))
                                .on_press(WalletsMessage::RevealMnemonic(selected_index))
                                .padding(10)
                                .style(primary_button_style()),
                        );
                } else {
                    let words: Vec<&str> = mnemonic.split_whitespace().collect();
                    let mnemonic_line = words.join(" ");
                    let test_active = self.backup_test_wallet_index == Some(selected_index);

                    if test_active {
                        panel = panel.push(
                            text("Mnemonic đang được ẩn khi làm bài test backup.")
                                .size(12)
                                .style(text_color(Colors::WARNING)),
                        );
                    } else {
                        panel = panel.push(
                            container(
                                column![
                                    text("Mnemonic (12 words):")
                                        .size(12)
                                        .style(text_color(Colors::TEXT_SECONDARY)),
                                    Space::with_height(6),
                                    text(mnemonic_line)
                                        .size(14)
                                        .style(text_color(Colors::ACCENT_TEAL)),
                                ]
                                .spacing(2),
                            )
                            .style(card_style())
                            .padding(12)
                            .width(Length::Fill),
                        );

                        panel = panel.push(
                            button(text("Export mnemonic to PDF").size(13))
                                .on_press(WalletsMessage::ExportMnemonicPdf(selected_index))
                                .padding(10)
                                .style(secondary_button_style()),
                        );
                    }

                    if wallet.mnemonic_backed_up {
                        panel = panel.push(
                            text("Mnemonic backup: Verified")
                                .size(13)
                                .style(text_color(Colors::SUCCESS)),
                        );
                    } else {
                        panel = panel.push(
                            button(
                                text(if test_active {
                                    "Hủy bài test backup"
                                } else {
                                    "Bắt đầu bài test backup"
                                })
                                .size(13),
                            )
                            .on_press(WalletsMessage::ToggleBackupTest {
                                wallet_index: selected_index,
                                word_count: words.len(),
                            })
                            .padding(10)
                            .style(secondary_button_style()),
                        );

                        if test_active {
                            let mut test_form =
                                column![text("Nhập đúng các từ theo vị trí để xác nhận backup")
                                    .size(12)
                                    .style(text_color(Colors::TEXT_SECONDARY)),]
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
                                        text(format!("Word #{}", position))
                                            .size(12)
                                            .style(text_color(Colors::TEXT_PRIMARY)),
                                        text_input("Nhập từ mnemonic...", value)
                                            .on_input(move |input| {
                                                WalletsMessage::BackupWordChanged(
                                                    field_index,
                                                    input,
                                                )
                                            })
                                            .padding(10)
                                            .size(13),
                                    ]
                                    .spacing(4),
                                );
                            }

                            test_form = test_form.push(
                                button(text("Xác nhận đã backup").size(13))
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
                }
            }
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

    let mut positions = vec![1, ((word_count + 1) / 2).max(1), word_count];
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
