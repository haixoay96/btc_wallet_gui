use crate::ui::components::network_status::{network_status_indicator, NetworkStatus};
use crate::ui::components::skeleton_wallet_cards;
use crate::ui::i18n::t;
use crate::ui::theme::{
    card_style, notice_style, primary_button_style, secondary_button_style, text_color,
    text_muted_color, text_primary_color, text_scaled, text_secondary_color, Colors, NoticeTone,
};
use crate::ui::views::sidebar::NavItem;
use iced::{
    widget::{button, column, container, row, Space},
    Alignment, Element, Length,
};

use super::structure::*;

impl DashboardView {
    pub fn new() -> Self {
        Self {
            total_balance: 0,
            confirmed_balance: 0,
            pending_balance: 0,
            wallet_count: 0,
            backup_needed_wallets: 0,
            last_synced_label: None,
            network_status: NetworkStatus::Checking,
        }
    }

    pub fn update_balances(
        &mut self,
        total: i64,
        confirmed: i64,
        pending: i64,
        wallets: usize,
        backup_needed_wallets: usize,
    ) {
        self.total_balance = total;
        self.confirmed_balance = confirmed;
        self.pending_balance = pending;
        self.wallet_count = wallets;
        self.backup_needed_wallets = backup_needed_wallets;
    }

    pub fn set_last_synced_label(&mut self, label: Option<String>) {
        self.last_synced_label = label;
    }

    pub fn view(
        &self,
        is_refreshing: bool,
        show_satoshis: bool,
        compact: bool,
    ) -> Element<'_, DashboardMessage> {
        let title = text_scaled(t("Tổng quan", "Dashboard"), 32).style(text_primary_color());

        // Network status indicator
        let network_indicator = network_status_indicator(self.network_status, !is_refreshing)
            .map(DashboardMessage::Network);

        let total_btc = self.total_balance as f64 / 100_000_000.0;
        let confirmed_btc = self.confirmed_balance as f64 / 100_000_000.0;
        let pending_btc = self.pending_balance as f64 / 100_000_000.0;

        // Compact mode adjustments
        let card_padding = if compact { 12 } else { 24 };
        let content_padding = if compact { 16 } else { 32 };
        let spacing = if compact { 8 } else { 16 };

        let balance_card = container(
            column![
                text_scaled(t("Tổng số dư", "Total Balance"), 14).style(text_secondary_color()),
                Space::with_height(4),
                text_scaled(format!("{:.8} BTC", total_btc), 32).style(text_primary_color()),
                Space::with_height(2),
                if show_satoshis {
                    text_scaled(format!("{} sat", self.total_balance), 14).style(text_muted_color())
                } else {
                    text_scaled("", 10).height(0)
                }
            ]
            .padding(card_padding),
        )
        .style(card_style())
        .width(Length::Fill);

        let confirmed_card = container(
            column![
                text_scaled(t("Số dư đã xác nhận", "Confirmed Balance"), 14)
                    .style(text_secondary_color()),
                Space::with_height(4),
                text_scaled(format!("{:.8} BTC", confirmed_btc), 24)
                    .style(text_color(Colors::SUCCESS)),
                Space::with_height(2),
                if show_satoshis {
                    text_scaled(format!("{} sat", self.confirmed_balance), 14)
                        .style(text_muted_color())
                } else {
                    text_scaled("", 10).height(0)
                }
            ]
            .padding(card_padding),
        )
        .style(card_style())
        .width(Length::Fill);

        let pending_card = container(
            column![
                text_scaled(t("Số dư chờ xác nhận", "Pending Balance"), 14)
                    .style(text_secondary_color()),
                Space::with_height(4),
                text_scaled(format!("{:.8} BTC", pending_btc), 24)
                    .style(text_color(Colors::WARNING)),
                Space::with_height(2),
                if show_satoshis {
                    text_scaled(format!("{} sat", self.pending_balance), 14)
                        .style(text_muted_color())
                } else {
                    text_scaled("", 10).height(0)
                }
            ]
            .padding(card_padding),
        )
        .style(card_style())
        .width(Length::Fill);

        let wallets_card = container(
            column![
                text_scaled(t("Tổng số ví", "Total Wallets"), 14).style(text_secondary_color()),
                Space::with_height(4),
                text_scaled(format!("{}", self.wallet_count), 36)
                    .style(text_color(Colors::ACCENT_PURPLE)),
            ]
            .padding(card_padding),
        )
        .style(card_style())
        .width(Length::Fill);

        let backup_card = container(
            column![
                text_scaled(t("Ví cần backup", "Wallets Needing Backup"), 14)
                    .style(text_secondary_color()),
                Space::with_height(4),
                text_scaled(format!("{}", self.backup_needed_wallets), 28).style(
                    if self.backup_needed_wallets == 0 {
                        text_primary_color()
                    } else {
                        text_color(Colors::WARNING)
                    }
                ),
                Space::with_height(2),
                text_scaled(
                    if self.backup_needed_wallets == 0 {
                        t(
                            "Tất cả ví đã xác minh backup",
                            "All wallets have verified backups",
                        )
                    } else {
                        t(
                            "Nên xử lý sớm để tránh mất seed",
                            "Should be handled soon to avoid seed loss",
                        )
                    },
                    12
                )
                .style(text_muted_color()),
            ]
            .padding(card_padding),
        )
        .style(card_style())
        .width(Length::Fill);

        let refresh_label = if is_refreshing {
            t("Đang làm mới...", "Refreshing...")
        } else {
            t("Làm mới", "Refresh")
        };

        let mut refresh_button = button(text_scaled(refresh_label, 16))
            .padding(12)
            .style(primary_button_style());
        if !is_refreshing {
            refresh_button = refresh_button.on_press(DashboardMessage::Refresh);
        }

        let quick_actions = row![
            button(text_scaled(t("Quản lý ví", "Manage Wallets"), 14))
                .on_press(DashboardMessage::Navigate(NavItem::Wallets))
                .padding(12)
                .style(secondary_button_style()),
            button(text_scaled(t("Gửi BTC", "Send BTC"), 14))
                .on_press(DashboardMessage::Navigate(NavItem::Send))
                .padding(12)
                .style(secondary_button_style()),
            button(text_scaled(t("Nhận BTC", "Receive BTC"), 14))
                .on_press(DashboardMessage::Navigate(NavItem::Receive))
                .padding(12)
                .style(secondary_button_style()),
        ]
        .spacing(10);

        let mut content = column![
            row![
                title,
                Space::with_width(Length::Fill),
                network_indicator,
                Space::with_width(8),
                refresh_button
            ]
            .align_y(Alignment::Center),
            Space::with_height(12),
            quick_actions,
            Space::with_height(spacing),
        ]
        .padding(content_padding)
        .spacing(0);

        if let Some(last_synced) = &self.last_synced_label {
            content = content.push(
                container(
                    text_scaled(
                        format!("{}: {}", t("Đồng bộ gần nhất", "Last synced"), last_synced),
                        12,
                    )
                    .style(text_primary_color()),
                )
                .style(notice_style(NoticeTone::Info))
                .padding(12)
                .width(Length::Fill),
            );
            content = content.push(Space::with_height(16));
        }

        if self.wallet_count == 0 {
            content = content.push(
                container(
                    column![
                        text_scaled(t("Chưa có ví nào", "No wallets yet"), 22)
                            .style(text_primary_color()),
                        text_scaled(
                            t(
                                "Bắt đầu bằng cách tạo ví mới hoặc import một ví hiện có.",
                                "Start by creating a new wallet or importing an existing one.",
                            ),
                            14
                        )
                        .style(text_secondary_color()),
                        Space::with_height(12),
                        button(text_scaled(t("Đi tới Wallets", "Go to Wallets"), 14))
                            .on_press(DashboardMessage::Navigate(NavItem::Wallets))
                            .padding(12)
                            .style(primary_button_style()),
                    ]
                    .spacing(10)
                    .align_x(Alignment::Center),
                )
                .style(card_style())
                .padding(28)
                .width(Length::Fill),
            );
        } else if is_refreshing {
            // Show skeleton loading state
            content = content.push(skeleton_wallet_cards(3).map(|_| DashboardMessage::Refresh));
        } else {
            content = content
                .push(balance_card)
                .push(Space::with_height(spacing))
                .push(
                    row![confirmed_card, Space::with_width(spacing), pending_card]
                        .width(Length::Fill),
                )
                .push(Space::with_height(spacing))
                .push(
                    row![wallets_card, Space::with_width(spacing), backup_card].width(Length::Fill),
                );
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
