use crate::i18n::t;
use crate::theme::{
    card_style, notice_style, primary_button_style, secondary_button_style, text_color, Colors,
    NoticeTone,
};
use crate::views::sidebar::NavItem;
use iced::{
    widget::{button, column, container, row, text, Space},
    Alignment, Element, Length,
};

#[derive(Debug, Clone)]
pub enum DashboardMessage {
    Refresh,
    Navigate(NavItem),
}

pub struct DashboardView {
    total_balance: i64,
    confirmed_balance: i64,
    pending_balance: i64,
    wallet_count: usize,
    backup_needed_wallets: usize,
    last_synced_label: Option<String>,
}

impl DashboardView {
    pub fn new() -> Self {
        Self {
            total_balance: 0,
            confirmed_balance: 0,
            pending_balance: 0,
            wallet_count: 0,
            backup_needed_wallets: 0,
            last_synced_label: None,
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

    pub fn view(&self, is_refreshing: bool) -> Element<'_, DashboardMessage> {
        let title = text(t("Tổng quan", "Dashboard"))
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let total_btc = self.total_balance as f64 / 100_000_000.0;
        let confirmed_btc = self.confirmed_balance as f64 / 100_000_000.0;
        let pending_btc = self.pending_balance as f64 / 100_000_000.0;

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

        let pending_card = container(
            column![
                text(t("Số dư chờ xác nhận", "Pending Balance"))
                    .size(14)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(8),
                text(format!("{:.8} BTC", pending_btc))
                    .size(24)
                    .style(text_color(Colors::WARNING)),
                Space::with_height(4),
                text(format!("{} sat", self.pending_balance))
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

        let backup_card = container(
            column![
                text(t("Ví cần backup", "Wallets Needing Backup"))
                    .size(14)
                    .style(text_color(Colors::TEXT_SECONDARY)),
                Space::with_height(8),
                text(format!("{}", self.backup_needed_wallets))
                    .size(28)
                    .style(if self.backup_needed_wallets == 0 {
                        text_color(Colors::SUCCESS)
                    } else {
                        text_color(Colors::WARNING)
                    }),
                Space::with_height(4),
                text(if self.backup_needed_wallets == 0 {
                    t(
                        "Tất cả ví đã xác minh backup",
                        "All wallets have verified backups",
                    )
                } else {
                    t(
                        "Nên xử lý sớm để tránh mất seed",
                        "Should be handled soon to avoid seed loss",
                    )
                })
                .size(12)
                .style(text_color(Colors::TEXT_MUTED)),
            ]
            .padding(24),
        )
        .style(card_style())
        .width(Length::Fill);

        let refresh_label = if is_refreshing {
            t("Đang làm mới...", "Refreshing...")
        } else {
            t("Làm mới", "Refresh")
        };

        let mut refresh_button = button(text(refresh_label).size(16))
            .padding(12)
            .style(primary_button_style());
        if !is_refreshing {
            refresh_button = refresh_button.on_press(DashboardMessage::Refresh);
        }

        let quick_actions = row![
            button(text(t("Quản lý ví", "Manage Wallets")).size(14))
                .on_press(DashboardMessage::Navigate(NavItem::Wallets))
                .padding(12)
                .style(secondary_button_style()),
            button(text(t("Gửi BTC", "Send BTC")).size(14))
                .on_press(DashboardMessage::Navigate(NavItem::Send))
                .padding(12)
                .style(secondary_button_style()),
            button(text(t("Nhận BTC", "Receive BTC")).size(14))
                .on_press(DashboardMessage::Navigate(NavItem::Receive))
                .padding(12)
                .style(secondary_button_style()),
        ]
        .spacing(10);

        let mut content = column![
            row![title, Space::with_width(Length::Fill), refresh_button].align_y(Alignment::Center),
            Space::with_height(12),
            quick_actions,
            Space::with_height(24),
        ]
        .padding(32)
        .spacing(0);

        if let Some(last_synced) = &self.last_synced_label {
            content = content.push(
                container(
                    text(format!(
                        "{}: {}",
                        t("Đồng bộ gần nhất", "Last synced"),
                        last_synced
                    ))
                    .size(12)
                    .style(text_color(Colors::TEXT_PRIMARY)),
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
                        text(t("Chưa có ví nào", "No wallets yet"))
                            .size(22)
                            .style(text_color(Colors::TEXT_PRIMARY)),
                        text(t(
                            "Bắt đầu bằng cách tạo ví mới hoặc import một ví hiện có.",
                            "Start by creating a new wallet or importing an existing one.",
                        ))
                        .size(14)
                        .style(text_color(Colors::TEXT_SECONDARY)),
                        Space::with_height(12),
                        button(text(t("Đi tới Wallets", "Go to Wallets")).size(14))
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
        } else {
            content = content
                .push(balance_card)
                .push(Space::with_height(16))
                .push(row![confirmed_card, Space::with_width(16), pending_card].width(Length::Fill))
                .push(Space::with_height(16))
                .push(row![wallets_card, Space::with_width(16), backup_card].width(Length::Fill));
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
