use iced::{
    widget::{column, container, row, text, Space},
    Element, Length,
};

use crate::app::structure::*;
use crate::ui::components::{modal, shortcuts_help_popup};
use crate::ui::i18n::t;
use crate::ui::theme::{
    get_theme_colors, screen_background_style, text_muted_color, text_secondary_color,
};
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
                    .view(self.wallets.len())
                    .map(AppMessage::SidebarMessage);
                let _selected_wallet = self.wallets.get(self.selected_wallet);

                let main_content = match self.current_page {
                    NavItem::Dashboard => self
                        .dashboard
                        .view(
                            self.is_refreshing,
                            self.btc_price.clone(),
                            self.is_fetching_price,
                        )
                        .map(AppMessage::DashboardMessage),
                    NavItem::Wallets => self
                        .wallets_view
                        .view(
                            &self.wallets,
                            self.selected_wallet,
                            self.selected_wallet_revealed_mnemonic(),
                            self.btc_price.as_ref().map(|p| p.price_usd),
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
                        )
                        .map(AppMessage::SendMessage),
                    NavItem::Receive => self
                        .receive_view
                        .view(&self.wallets, self.selected_wallet)
                        .map(AppMessage::ReceiveMessage),
                    NavItem::History => self
                        .history_view
                        .view(&self.wallets, self.selected_wallet, self.is_refreshing)
                        .map(AppMessage::HistoryMessage),
                    NavItem::Settings => self
                        .settings_view
                        .view(self.theme, self.font_scale, self.high_contrast)
                        .map(AppMessage::SettingsMessage),
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
                        .style(text_secondary_color()),
                        Space::with_width(Length::Fill),
                        text(self.current_page.title())
                            .size(14)
                            .style(text_muted_color()),
                        Space::with_width(12),
                        language_picker,
                    ]
                    .align_y(iced::Alignment::Center),
                )
                .padding(12);

                let base_content = container(row![
                    sidebar,
                    column![header_bar, main_content,].width(Length::Fill)
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
                        false, // compact_mode removed
                        true,  // close_on_backdrop: true
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
