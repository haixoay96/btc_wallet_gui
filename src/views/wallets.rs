use iced::{
    widget::{button, column, container, row, text, text_input, Space, scrollable},
    Alignment, Element, Length,
};
use crate::theme::{Colors, card_style, primary_button_style, secondary_button_style, text_color, danger_button_style};
use crate::wallet::WalletNetwork;

#[derive(Debug, Clone)]
pub enum WalletsMessage {
    ToggleCreateForm,
    CreateWallet,
    NameChanged(String),
    NetworkChanged(WalletNetwork),
    SelectWallet(usize),
    DeleteWallet(usize),
    ConfirmDelete(usize),
    CancelDelete,
}

pub struct WalletsView {
    create_name: String,
    create_network: WalletNetwork,
    show_create_form: bool,
    confirm_delete_index: Option<usize>,
}

impl WalletsView {
    pub fn new() -> Self {
        Self {
            create_name: String::new(),
            create_network: WalletNetwork::Testnet,
            show_create_form: false,
            confirm_delete_index: None,
        }
    }

    pub fn update(&mut self, message: WalletsMessage) -> Option<crate::app::AppMessage> {
        match message {
            WalletsMessage::ToggleCreateForm => {
                self.show_create_form = !self.show_create_form;
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
            WalletsMessage::SelectWallet(index) => {
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
        }
    }

    pub fn view(&self, wallets: &[crate::wallet::WalletEntry], selected: usize) -> Element<'_, WalletsMessage> {
        let title = text("Wallets")
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let toggle_btn = button(
            text(if self.show_create_form { "✕ Cancel" } else { "+ Create Wallet" }).size(14)
        )
        .on_press(WalletsMessage::ToggleCreateForm)
        .padding(10)
        .style(if self.show_create_form { secondary_button_style() } else { primary_button_style() });

        let mut content = column![title, Space::with_height(16), toggle_btn]
            .spacing(16)
            .padding(32);

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

            let create_btn = button(text("✓ Create").size(14))
                .on_press(WalletsMessage::CreateWallet)
                .padding(10)
                .style(primary_button_style());

            let form = container(
                column![
                    text("Create New Wallet").size(18).style(text_color(Colors::TEXT_PRIMARY)),
                    Space::with_height(12),
                    name_input,
                    Space::with_height(8),
                    row![network_testnet, network_mainnet].spacing(8),
                    Space::with_height(12),
                    create_btn,
                ]
                .spacing(8)
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
                let balance: i64 = wallet.history.iter().map(|tx| tx.amount_sat).sum();
                let balance_btc = balance as f64 / 100_000_000.0;
                let wallet_name = wallet.name.clone();
                let wallet_network = wallet.network.as_str().to_string();

                let select_btn = button(
                    row![
                        column![
                            text(wallet_name).size(16).style(text_color(Colors::TEXT_PRIMARY)),
                            text(format!("{} | {:.8} BTC", wallet_network, balance_btc))
                                .size(12)
                                .style(text_color(Colors::TEXT_SECONDARY)),
                        ].spacing(4),
                        Space::with_width(Length::Fill),
                        if is_selected {
                            text("✓").size(20).style(text_color(Colors::SUCCESS))
                        } else {
                            text("").size(20)
                        },
                    ]
                    .align_y(Alignment::Center)
                )
                .on_press(WalletsMessage::SelectWallet(index))
                .padding(12)
                .width(Length::Fill)
                .style(if is_selected { primary_button_style() } else { secondary_button_style() });

                let delete_btn = button(text("🗑").size(16))
                    .on_press(WalletsMessage::DeleteWallet(index))
                    .padding(8)
                    .style(danger_button_style());

                wallet_list = wallet_list.push(
                    container(
                        row![select_btn, Space::with_width(8), delete_btn]
                            .align_y(Alignment::Center)
                    )
                    .style(card_style())
                    .padding(8)
                );
                wallet_list = wallet_list.push(Space::with_height(8));
            }

            content = content.push(
                column![
                    text("Your Wallets").size(18).style(text_color(Colors::TEXT_PRIMARY)),
                    Space::with_height(12),
                    scrollable(wallet_list).height(Length::Shrink),
                ]
            );
        } else if !self.show_create_form {
            content = content.push(
                container(
                    text("No wallets yet. Create your first wallet!")
                        .size(16)
                        .style(text_color(Colors::TEXT_SECONDARY))
                )
                .padding(40)
                .center_x(Length::Fill)
            );
        }

        if let Some(index) = self.confirm_delete_index {
            let wallet_name = wallets.get(index)
                .map(|w| w.name.clone())
                .unwrap_or_default();
                
            let dialog = container(
                column![
                    text("Confirm Delete").size(20).style(text_color(Colors::ERROR)),
                    Space::with_height(12),
                    text(format!("Delete wallet '{}'?", wallet_name))
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
                    ]
                ]
                .spacing(8)
                .padding(24)
            )
            .style(card_style())
            .width(Length::Fixed(400.0));

            content = content.push(
                container(dialog)
                    .center_x(Length::Fill)
                    .padding(20)
            );
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }
}
