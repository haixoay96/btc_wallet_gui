use iced::{
    widget::{column, container, row, text, Space},
    Element, Length,
};

use crate::app::structure::*;
use crate::ui::components::{error_card, modal, shortcuts_help_popup};
use crate::ui::i18n::t;
use crate::ui::theme::{get_theme_colors, screen_background_style, text_color, Colors};
use crate::ui::views::sidebar::NavItem;

impl App {
    pub fn view(&self) -> Element<'_, AppMessage> {
        match self.state {
            AppState::Login => container(self.login_view.view().map(AppMessage::LoginMessage))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            AppState::Main => {
                let sidebar = self
                    .sidebar
                    .view(self.wallets.len(), self.compact_mode)
                    .map(AppMessage::SidebarMessage);
                let _selected_wallet = self.wallets.get(self.selected_wallet);

                let main_content = match self.current_page {
                    NavItem::Dashboard => self
                        .dashboard
                        .view(self.is_refreshing, self.show_satoshis, self.compact_mode)
                        .map(AppMessage::DashboardMessage),
                    NavItem::Wallets => self
                        .wallets_view
                        .view(
                            &self.wallets,
                            self.selected_wallet,
                            self.selected_wallet_revealed_mnemonic(),
                            self.compact_mode,
                        )
                        .map(AppMessage::WalletsMessage),
                    NavItem::Send => self
                        .send_view
                        .view(
                            &self.wallets,
                            self.selected_wallet,
                            self.is_estimating_fee,
                            self.is_calculating_max,
                            self.is_sending,
                            &self.address_book,
                            self.compact_mode,
                        )
                        .map(AppMessage::SendMessage),
                    NavItem::Receive => self
                        .receive_view
                        .view(&self.wallets, self.selected_wallet, self.compact_mode)
                        .map(AppMessage::ReceiveMessage),
                    NavItem::History => self
                        .history_view
                        .view(
                            &self.wallets,
                            self.selected_wallet,
                            self.is_refreshing,
                            self.compact_mode,
                        )
                        .map(AppMessage::HistoryMessage),
                    NavItem::Settings => self
                        .settings_view
                        .view(
                            self.theme,
                            self.font_scale,
                            self.high_contrast,
                            self.compact_mode,
                        )
                        .map(AppMessage::SettingsMessage),
                };

                let error_bar = if let Some(app_error) = &self.error {
                    let mut detail = app_error.user_message();
                    if let Some(ctx) = app_error.context() {
                        detail.push_str(&format!("\n📋 {}", ctx));
                    }
                    container(error_card(
                        app_error.title(),
                        format!("{}\n\n💡 {}", detail, app_error.suggestion()),
                        if app_error.is_retryable() {
                            Some(AppMessage::DismissError)
                        } else {
                            None
                        },
                    ))
                    .padding(10)
                } else {
                    container(Space::with_height(0))
                };

                let language_picker = self.language_selector.view(AppMessage::LanguageChanged);

                let header_bar = container(
                    row![
                        text(format!(
                            "{} {}",
                            t("Xin chào,", "Hello,"),
                            self.display_name()
                        ))
                        .size(14)
                        .style(text_color(Colors::TEXT_SECONDARY)),
                        Space::with_width(Length::Fill),
                        text(self.current_page.title())
                            .size(14)
                            .style(text_color(Colors::TEXT_MUTED)),
                        Space::with_width(12),
                        language_picker,
                    ]
                    .align_y(iced::Alignment::Center),
                )
                .padding(12);

                let base_content = container(row![
                    sidebar,
                    column![header_bar, error_bar, main_content,].width(Length::Fill)
                ])
                .width(Length::Fill)
                .height(Length::Fill)
                .style(screen_background_style());

                // Shortcuts help popup overlay - Dùng component Modal mới
                if self.show_shortcuts_help {
                    return modal(
                        base_content.into(),
                        t("Phím tắt", "Keyboard Shortcuts"),
                        shortcuts_help_popup().map(|_| AppMessage::ToggleShortcutsHelp),
                        AppMessage::ToggleShortcutsHelp,
                        self.compact_mode,
                    );
                }

                // Toast notification layer on top - centered horizontally
                if self.toast_manager.has_toasts() {
                    if let Some(toast_view) = self.toast_manager.view() {
                        use iced::widget::stack;

                        // Dùng row với 2 Space ở 2 bên để đẩy toast vào giữa
                        let centered_row = row![
                            Space::with_width(Length::Fill),
                            column![
                                Space::with_height(20),
                                toast_view.map(|_| AppMessage::DismissStatus),
                            ]
                            .spacing(0),
                            Space::with_width(Length::Fill),
                        ]
                        .width(Length::Fill)
                        .spacing(0);

                        let toast_overlay = container(
                            column![centered_row, Space::with_height(Length::Fill),]
                                .spacing(0)
                                .width(Length::Fill),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill);

                        return stack![base_content, toast_overlay]
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into();
                    }
                }

                let final_content: Element<'_, AppMessage> = if self.show_onboarding {
                    // Stack onboarding overlay on top of main content
                    iced::widget::stack![
                        base_content,
                        // Semi-transparent overlay background
                        container(Space::with_width(Length::Fill))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .style(|theme: &iced::Theme| {
                                let colors = get_theme_colors(theme);
                                iced::widget::container::Style {
                                    background: Some(iced::Background::Color(
                                        iced::Color::from_rgba(
                                            colors.bg_primary.r,
                                            colors.bg_primary.g,
                                            colors.bg_primary.b,
                                            0.75,
                                        ),
                                    )),
                                    ..Default::default()
                                }
                            }),
                        // Onboarding card
                        self.onboarding_view
                            .view()
                            .map(AppMessage::OnboardingMessage)
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else {
                    base_content.into()
                };
                final_content
            }
        }
    }
}
