use iced::{
    widget::{button, column, container, row, text, Space, scrollable},
    Alignment, Element, Length,
};
use crate::theme::{Colors, card_style, primary_button_style, secondary_button_style, text_color};
use crate::wallet::WalletEntry;

#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    CopyAddress(String),
    DeriveNewAddress,
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
            ReceiveMessage::CopyAddress(_addr) => {
                self.copied = true;
                None
            }
            ReceiveMessage::DeriveNewAddress => {
                None
            }
        }
    }

    pub fn view(&self, wallet: Option<&WalletEntry>) -> Element<ReceiveMessage> {
        let title = text("Receive BTC")
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let mut content = column![title].spacing(16).padding(32);

        if let Some(wallet) = wallet {
            let balance: i64 = wallet.history.iter().map(|tx| tx.amount_sat).sum();
            let balance_btc = balance as f64 / 100_000_000.0;

            content = content.push(
                text(format!("Balance: {:.8} BTC | Network: {}", balance_btc, wallet.network.as_str()))
                    .size(14)
                    .style(text_color(Colors::TEXT_SECONDARY))
            );

            let selected_addr = wallet.addresses.get(self.selected_index).cloned();
            if let Some(addr) = selected_addr {
                content = content.push(Space::with_height(16));
                content = content.push(
                    text("Selected Address:").size(16).style(text_color(Colors::TEXT_PRIMARY))
                );
                content = content.push(Space::with_height(8));
                content = content.push(
                    container(
                        text(addr.address.clone())
                            .size(14)
                            .style(text_color(Colors::ACCENT_TEAL))
                    )
                    .style(card_style())
                    .padding(16)
                    .width(Length::Fill)
                );
                content = content.push(Space::with_height(12));
                content = content.push(
                    button(text(if self.copied { "✓ Copied!" } else { "📋 Copy Address" }).size(14))
                        .on_press(ReceiveMessage::CopyAddress(addr.address))
                        .padding(12)
                        .style(primary_button_style())
                );
            }

            content = content.push(Space::with_height(24));
            content = content.push(
                text("All Addresses").size(18).style(text_color(Colors::TEXT_PRIMARY))
            );
            content = content.push(Space::with_height(8));

            for (i, addr) in wallet.addresses.iter().enumerate() {
                let is_selected = i == self.selected_index;
                content = content.push(
                    container(
                        row![
                            text(format!("#{}", addr.index)).size(12).style(text_color(Colors::TEXT_MUTED)),
                            Space::with_width(8),
                            text(addr.address.clone()).size(11).style(text_color(Colors::TEXT_PRIMARY)),
                        ]
                        .align_y(Alignment::Center)
                    )
                    .style(card_style())
                    .padding(8)
                    .width(Length::Fill)
                );
                content = content.push(Space::with_height(4));
            }
        } else {
            content = content.push(
                text("No wallet selected").size(16).style(text_color(Colors::ERROR))
            );
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}