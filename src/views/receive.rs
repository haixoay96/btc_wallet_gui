use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Alignment, Element, Length,
};

use crate::theme::{card_style, primary_button_style, secondary_button_style, text_color, Colors};
use crate::wallet::WalletEntry;

#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    CopyAddress(String),
    DeriveNewAddress,
    SelectAddress(usize),
}

pub struct ReceiveView {
    selected_index: usize,
    copied: bool,
}

impl ReceiveView {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            copied: false,
        }
    }

    pub fn update(&mut self, message: ReceiveMessage) -> Option<crate::app::AppMessage> {
        match message {
            ReceiveMessage::CopyAddress(addr) => {
                self.copied = true;
                Some(crate::app::AppMessage::CopyAddress(addr))
            }
            ReceiveMessage::DeriveNewAddress => {
                self.copied = false;
                Some(crate::app::AppMessage::DeriveAddresses(1))
            }
            ReceiveMessage::SelectAddress(index) => {
                self.selected_index = index;
                self.copied = false;
                None
            }
        }
    }

    pub fn view(&self, wallet: Option<&WalletEntry>) -> Element<'_, ReceiveMessage> {
        let title = text("Receive BTC")
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let mut content = column![title].spacing(16).padding(32);

        if let Some(wallet) = wallet {
            let balance: i64 = wallet.history.iter().map(|tx| tx.amount_sat).sum();
            let balance_btc = balance as f64 / 100_000_000.0;

            content = content.push(
                text(format!(
                    "Balance: {:.8} BTC | Network: {}",
                    balance_btc,
                    wallet.network.as_str()
                ))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            );

            let derive_button = button(text("+ Derive New Address").size(14))
                .on_press(ReceiveMessage::DeriveNewAddress)
                .padding(10)
                .style(primary_button_style());

            content = content.push(derive_button);

            if wallet.addresses.is_empty() {
                content = content.push(
                    text("Ví chưa có địa chỉ, hãy bấm 'Derive New Address'.")
                        .size(14)
                        .style(text_color(Colors::TEXT_MUTED)),
                );
            } else {
                let selected_index = self.selected_index.min(wallet.addresses.len() - 1);
                if let Some(addr) = wallet.addresses.get(selected_index) {
                    content = content.push(Space::with_height(12));
                    content = content.push(
                        text("Selected Address:")
                            .size(16)
                            .style(text_color(Colors::TEXT_PRIMARY)),
                    );
                    content = content.push(
                        container(
                            text(addr.address.clone())
                                .size(14)
                                .style(text_color(Colors::ACCENT_TEAL)),
                        )
                        .style(card_style())
                        .padding(16)
                        .width(Length::Fill),
                    );
                    content = content.push(
                        button(text(if self.copied { "Copied!" } else { "Copy Address" }).size(14))
                            .on_press(ReceiveMessage::CopyAddress(addr.address.clone()))
                            .padding(10)
                            .style(if self.copied {
                                secondary_button_style()
                            } else {
                                primary_button_style()
                            }),
                    );
                }

                content = content.push(Space::with_height(16));
                content = content.push(
                    text("All Addresses")
                        .size(18)
                        .style(text_color(Colors::TEXT_PRIMARY)),
                );

                let mut list = column![];
                for (i, addr) in wallet.addresses.iter().enumerate() {
                    let is_selected = i == selected_index;
                    let row_content = row![
                        text(format!("#{}", addr.index))
                            .size(12)
                            .style(text_color(Colors::TEXT_MUTED)),
                        Space::with_width(8),
                        text(addr.address.clone())
                            .size(11)
                            .style(text_color(Colors::TEXT_PRIMARY)),
                        Space::with_width(Length::Fill),
                        if is_selected {
                            text("Selected")
                                .size(11)
                                .style(text_color(Colors::SUCCESS))
                        } else {
                            text("")
                        },
                    ]
                    .align_y(Alignment::Center);

                    list = list.push(
                        button(container(row_content).width(Length::Fill))
                            .on_press(ReceiveMessage::SelectAddress(i))
                            .padding(8)
                            .style(if is_selected {
                                primary_button_style()
                            } else {
                                secondary_button_style()
                            })
                            .width(Length::Fill),
                    );
                    list = list.push(Space::with_height(6));
                }

                content = content.push(scrollable(list).height(Length::Fill));
            }
        } else {
            content = content.push(text("No wallet selected").size(16).style(text_color(Colors::ERROR)));
        }

        container(content).width(Length::Fill).height(Length::Fill).into()
    }
}
