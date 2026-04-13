use iced::{
    widget::{button, column, container, row, Space},
    Alignment, Element, Length,
};

use crate::ui::i18n::t;
use crate::ui::theme::{
    card_style, get_theme_colors, primary_button_style, secondary_button_style,
    text_accent_purple_color, text_accent_teal_color, text_color, text_muted_color,
    text_primary_color, text_scaled, text_secondary_color, text_warning_color,
};

use super::structure::*;

impl OnboardingView {
    pub fn new() -> Self {
        Self {
            current_step: 0,
            total_steps: 5,
        }
    }

    pub fn update(&mut self, message: OnboardingMessage) -> Option<OnboardingEvent> {
        match message {
            OnboardingMessage::Next => {
                if self.current_step < self.total_steps - 1 {
                    self.current_step += 1;
                } else {
                    return Some(OnboardingEvent::Finished);
                }
                None
            }
            OnboardingMessage::Previous => {
                if self.current_step > 0 {
                    self.current_step -= 1;
                }
                None
            }
            OnboardingMessage::Skip => Some(OnboardingEvent::Skipped),
            OnboardingMessage::Complete => Some(OnboardingEvent::Finished),
        }
    }

    fn step_title(step: u8) -> &'static str {
        match step {
            0 => t("Chào mừng!", "Welcome!"),
            1 => t("Dashboard - Tổng quan", "Dashboard - Overview"),
            2 => t("Quản lý ví", "Wallet Management"),
            3 => t("Gửi & Nhận BTC", "Send & Receive BTC"),
            4 => t("Bảo mật & Backup", "Security & Backup"),
            _ => "",
        }
    }

    fn step_description(step: u8) -> &'static str {
        match step {
            0 => t(
                "Bitcoin Wallet giúp bạn quản lý Bitcoin an toàn, phi tập trung.\nKhông tài khoản, không trung gian - chỉ có bạn và blockchain.",
                "Bitcoin Wallet helps you manage Bitcoin securely & decentral.\nNo accounts, no intermediaries - just you and the blockchain.",
            ),
            1 => t(
                "Theo dõi toàn bộ tài sản, số dư đã xác nhận & đang chờ.\nXem nhanh số ví, giao dịch gần nhất và trạng thái backup.",
                "Track all assets, confirmed & pending balances.\nQuick view of wallets, recent transactions & backup status.",
            ),
            2 => t(
                "Tạo ví mới, nhập ví có sẵn hoặc khôi phục từ mnemonic seed.\nHỗ trợ đa ví, chuyển đổi linh hoạt giữa Mainnet & Testnet.",
                "Create new wallets, import existing ones or restore from mnemonic seed.\nMulti-wallet support, seamless Mainnet & Testnet switching.",
            ),
            3 => t(
                "Gửi Bitcoin đến bất kỳ địa chỉ nào với ước tính phí thông minh.\nNhận thanh toán dễ dàng qua QR Code hoặc sao chép địa chỉ.",
                "Send Bitcoin to any address with smart fee estimation.\nReceive payments easily via QR Code or copy-paste address.",
            ),
            4 => t(
                "Mnemonic seed là CHÌA KHÓA DUY NHẤT khôi phục ví. Mất seed = Mất BTC!\nHãy backup ngay và lưu trữ offline an toàn tuyệt đối.",
                "Mnemonic seed is the ONLY KEY to recover your wallet. Lost seed = Lost BTC!\nBackup now and store offline securely.",
            ),
            _ => "",
        }
    }

    fn step_mockup(step: u8) -> Element<'static, OnboardingMessage> {
        match step {
            0 => container(text_scaled('₿', 64).style(text_accent_teal_color()))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into(),
            1 => {
                let total_card = container(
                    column![
                        text_scaled(t("Tổng số dư", "Total Balance"), 10).style(text_muted_color()),
                        Space::with_height(4),
                        text_scaled("0.12345678 BTC", 18).style(text_accent_teal_color()),
                    ]
                    .padding(12),
                )
                .style(|theme: &iced::Theme| {
                    let colors = get_theme_colors(theme);
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(colors.bg_input)),
                        border: iced::border::rounded(8),
                        ..Default::default()
                    }
                })
                .width(Length::Fill);

                let stats_row = row![
                    container(
                        column![
                            text_scaled(t("Ví", "Wallets"), 9).style(text_muted_color()),
                            Space::with_height(2),
                            text_scaled("3", 16).style(text_primary_color()),
                        ]
                        .padding(8)
                    )
                    .style(|theme: &iced::Theme| {
                        let colors = get_theme_colors(theme);
                        iced::widget::container::Style {
                            background: Some(iced::Background::Color(colors.bg_input)),
                            border: iced::border::rounded(8),
                            ..Default::default()
                        }
                    })
                    .width(Length::Fill),
                    Space::with_width(8),
                    container(
                        column![
                            text_scaled(t("Giao dịch", "Txns"), 9).style(text_muted_color()),
                            Space::with_height(2),
                            text_scaled("42", 16).style(text_primary_color()),
                        ]
                        .padding(8)
                    )
                    .style(|theme: &iced::Theme| {
                        let colors = get_theme_colors(theme);
                        iced::widget::container::Style {
                            background: Some(iced::Background::Color(colors.bg_input)),
                            border: iced::border::rounded(8),
                            ..Default::default()
                        }
                    })
                    .width(Length::Fill),
                ];

                column![total_card, Space::with_height(8), stats_row]
                    .spacing(0)
                    .into()
            }
            2 => {
                let wallet_cards = column![
                    container(
                        row![
                            text_scaled("Main", 11).style(text_primary_color()),
                            Space::with_width(Length::Fill),
                            text_scaled("0.08 BTC", 11).style(text_accent_teal_color()),
                        ]
                        .padding(10)
                    )
                    .style(|theme: &iced::Theme| {
                        let colors = get_theme_colors(theme);
                        iced::widget::container::Style {
                            background: Some(iced::Background::Color(colors.bg_input)),
                            border: iced::border::rounded(8),
                            ..Default::default()
                        }
                    })
                    .width(Length::Fill),
                    Space::with_height(6),
                    container(
                        row![
                            text_scaled("Testnet", 11).style(text_primary_color()),
                            Space::with_width(Length::Fill),
                            text_scaled("0.04 BTC", 11).style(text_accent_teal_color()),
                        ]
                        .padding(10)
                    )
                    .style(|theme: &iced::Theme| {
                        let colors = get_theme_colors(theme);
                        iced::widget::container::Style {
                            background: Some(iced::Background::Color(colors.bg_input)),
                            border: iced::border::rounded(8),
                            ..Default::default()
                        }
                    })
                    .width(Length::Fill),
                    Space::with_height(6),
                    container(
                        row![
                            text_scaled("+", 12).style(text_muted_color()),
                            Space::with_width(6),
                            text_scaled(t("Thêm ví mới", "Add new wallet"), 11)
                                .style(text_muted_color()),
                        ]
                        .padding(10)
                        .align_y(Alignment::Center)
                    )
                    .style(|theme: &iced::Theme| {
                        let colors = get_theme_colors(theme);
                        iced::widget::container::Style {
                            background: Some(iced::Background::Color(colors.bg_secondary)),
                            border: iced::border::rounded(8),
                            ..Default::default()
                        }
                    })
                    .width(Length::Fill),
                ];
                wallet_cards.into()
            }
            3 => {
                let qr_placeholder = container(
                    column![
                        text_scaled("QR", 24).style(text_primary_color()),
                        Space::with_height(4),
                        text_scaled("Code", 9).style(text_muted_color()),
                    ]
                    .align_x(Alignment::Center),
                )
                .style(|theme: &iced::Theme| {
                    let colors = get_theme_colors(theme);
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(colors.bg_input)),
                        border: iced::border::rounded(8),
                        ..Default::default()
                    }
                })
                .width(Length::Fixed(80.0))
                .height(Length::Fixed(80.0))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center);

                let address_preview =
                    container(text_scaled("bc1q...2dreul", 10).style(text_accent_purple_color()))
                        .style(|theme: &iced::Theme| {
                            let colors = get_theme_colors(theme);
                            iced::widget::container::Style {
                                background: Some(iced::Background::Color(colors.bg_input)),
                                border: iced::border::rounded(8),
                                ..Default::default()
                            }
                        })
                        .padding(8)
                        .width(Length::Fill);

                row![
                    qr_placeholder,
                    Space::with_width(12),
                    column![
                        address_preview,
                        Space::with_height(8),
                        container(
                            text_scaled(t("Sao chép", "Copy"), 10)
                                .style(text_color(iced::Color::from_rgb(1.0, 1.0, 1.0))),
                        )
                        .style(|theme: &iced::Theme| {
                            let colors = get_theme_colors(theme);
                            iced::widget::container::Style {
                                background: Some(iced::Background::Color(colors.accent_teal)),
                                border: iced::border::rounded(6),
                                ..Default::default()
                            }
                        })
                        .padding(6),
                    ]
                    .width(Length::Fill),
                ]
                .into()
            }
            4 => {
                let warning_banner = container(
                    row![
                        text_scaled("!", 16).style(text_warning_color()),
                        Space::with_width(8),
                        text_scaled(t("Backup ngay!", "Backup now!"), 12)
                            .style(text_warning_color()),
                    ]
                    .align_y(Alignment::Center),
                )
                .style(|theme: &iced::Theme| {
                    let colors = get_theme_colors(theme);
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            colors.warning.r,
                            colors.warning.g,
                            colors.warning.b,
                            0.15,
                        ))),
                        border: iced::border::rounded(8),
                        ..Default::default()
                    }
                })
                .padding(10)
                .width(Length::Fill);

                let seed_preview = container(
                    column![
                        row![
                            text_scaled("1.", 10).style(text_muted_color()),
                            Space::with_width(4),
                            text_scaled("abandon", 10).style(text_primary_color()),
                        ],
                        row![
                            text_scaled("2.", 10).style(text_muted_color()),
                            Space::with_width(4),
                            text_scaled("ability", 10).style(text_primary_color()),
                        ],
                        row![
                            text_scaled("3.", 10).style(text_muted_color()),
                            Space::with_width(4),
                            text_scaled("able", 10).style(text_primary_color()),
                        ],
                    ]
                    .padding(8),
                )
                .style(|theme: &iced::Theme| {
                    let colors = get_theme_colors(theme);
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(colors.bg_input)),
                        border: iced::border::rounded(8),
                        ..Default::default()
                    }
                })
                .width(Length::Fill);

                column![
                    warning_banner,
                    Space::with_height(8),
                    text_scaled(t("12 từ mnemonic", "12-word mnemonic"), 10)
                        .style(text_muted_color()),
                    Space::with_height(4),
                    seed_preview,
                ]
                .into()
            }
            _ => Space::with_height(80).into(),
        }
    }

    pub fn view(&self) -> Element<'_, OnboardingMessage> {
        let title =
            text_scaled(Self::step_title(self.current_step), 24).style(text_primary_color());

        let description = text_scaled(Self::step_description(self.current_step), 14)
            .style(text_secondary_color())
            .width(Length::Fill);

        let mockup =
            container(Self::step_mockup(self.current_step).map(|_| OnboardingMessage::Next))
                .style(|theme: &iced::Theme| {
                    let colors = get_theme_colors(theme);
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            colors.bg_card.r,
                            colors.bg_card.g,
                            colors.bg_card.b,
                            0.95,
                        ))),
                        border: iced::border::rounded(12),
                        ..Default::default()
                    }
                })
                .padding(20)
                .height(Length::Fixed(160.0))
                .width(Length::Fill);

        let mut dots = row![];
        for i in 0..self.total_steps {
            let is_current = i == self.current_step;
            let is_completed = i < self.current_step;
            let dot =
                container(Space::with_width(10).height(10)).style(move |theme: &iced::Theme| {
                    let colors = get_theme_colors(theme);
                    let color = if is_current {
                        colors.accent_purple
                    } else if is_completed {
                        colors.accent_teal
                    } else {
                        colors.border
                    };
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(color)),
                        border: iced::border::rounded(5),
                        ..Default::default()
                    }
                });
            dots = dots.push(dot);
            if i < self.total_steps - 1 {
                dots = dots.push(Space::with_width(6));
            }
        }

        let progress_indicator = container(dots).padding(10);

        let mut nav_buttons = row![];
        if self.current_step > 0 {
            nav_buttons = nav_buttons.push(
                button(text_scaled(t("Quay lại", "Back"), 13))
                    .on_press(OnboardingMessage::Previous)
                    .padding([10, 16])
                    .style(secondary_button_style()),
            );
            nav_buttons = nav_buttons.push(Space::with_width(12));
        } else {
            nav_buttons = nav_buttons.push(
                button(text_scaled(t("Bỏ qua", "Skip"), 13))
                    .on_press(OnboardingMessage::Skip)
                    .padding([10, 16])
                    .style(secondary_button_style()),
            );
            nav_buttons = nav_buttons.push(Space::with_width(12));
        }

        if self.current_step < self.total_steps - 1 {
            nav_buttons = nav_buttons.push(
                button(text_scaled(t("Tiếp theo", "Next"), 13))
                    .on_press(OnboardingMessage::Next)
                    .padding([10, 16])
                    .style(primary_button_style()),
            );
        } else {
            nav_buttons = nav_buttons.push(
                button(text_scaled(t("Bắt đầu", "Get Started"), 13))
                    .on_press(OnboardingMessage::Complete)
                    .padding([10, 16])
                    .style(primary_button_style()),
            );
        }

        let content: Element<'_, OnboardingMessage> = column![
            progress_indicator,
            Space::with_height(16),
            mockup,
            Space::with_height(20),
            title,
            Space::with_height(10),
            description,
            Space::with_height(28),
            nav_buttons.align_y(Alignment::Center),
        ]
        .align_x(Alignment::Center)
        .padding(36)
        .width(Length::Fixed(440.0))
        .into();

        container(content)
            .style(card_style())
            .width(Length::Shrink)
            .height(Length::Shrink)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
