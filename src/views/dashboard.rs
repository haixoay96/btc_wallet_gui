use crate::i18n::t;
use crate::theme::{card_style, primary_button_style, text_color, Colors};
use iced::{
    widget::{button, column, container, row, text, Space},
    Alignment, Element, Length,
};

#[derive(Debug, Clone)]
pub enum DashboardMessage {
    Refresh,
}

pub struct DashboardView {
    total_balance: i64,
    confirmed_balance: i64,
    wallet_count: usize,
}

impl DashboardView {
    pub fn new() -> Self {
        Self {
            total_balance: 0,
            confirmed_balance: 0,
            wallet_count: 0,
        }
    }

    pub fn update_balances(&mut self, total: i64, confirmed: i64, wallets: usize) {
        self.total_balance = total;
        self.confirmed_balance = confirmed;
        self.wallet_count = wallets;
    }

    pub fn view(&self) -> Element<'_, DashboardMessage> {
        let title = text(t("Tổng quan", "Dashboard"))
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let total_btc = self.total_balance as f64 / 100_000_000.0;
        let confirmed_btc = self.confirmed_balance as f64 / 100_000_000.0;

        let balance_card = container(
            column![
                text(t("Tổng số dư", "Total Balance"))
                    .size(14)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(8),
                text(format!("{:.8} BTC", total_btc))
                    .size(36)
                    .style(text_color(Colors::ACCENT_TEAL)),
                Space::with_height(4),
                text(format!("{} sat", self.total_balance))
                    .size(14)
                    .style(text_color(Colors::TEXT_MUTED)),
            ]
            .padding(24),
        )
        .style(card_style())
        .width(Length::Fill);

        let confirmed_card = container(
            column![
                text(t("Số dư đã xác nhận", "Confirmed Balance"))
                    .size(14)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(8),
                text(format!("{:.8} BTC", confirmed_btc))
                    .size(24)
                    .style(text_color(Colors::SUCCESS)),
                Space::with_height(4),
                text(format!("{} sat", self.confirmed_balance))
                    .size(14)
                    .style(text_color(Colors::TEXT_MUTED)),
            ]
            .padding(24),
        )
        .style(card_style())
        .width(Length::Fill);

        let wallets_card = container(
            column![
                text(t("Tổng số ví", "Total Wallets"))
                    .size(14)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(8),
                text(format!("{}", self.wallet_count))
                    .size(36)
                    .style(text_color(Colors::ACCENT_PURPLE)),
            ]
            .padding(24),
        )
        .style(card_style())
        .width(Length::Fill);

        let refresh_button = button(text(format!("🔄 {}", t("Làm mới", "Refresh"))).size(16))
            .on_press(DashboardMessage::Refresh)
            .padding(12)
            .style(primary_button_style());

        let content = column![
            row![title, Space::with_width(Length::Fill), refresh_button].align_y(Alignment::Center),
            Space::with_height(32),
            balance_card,
            Space::with_height(16),
            row![confirmed_card, Space::with_width(16), wallets_card].width(Length::Fill),
        ]
        .padding(32);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
