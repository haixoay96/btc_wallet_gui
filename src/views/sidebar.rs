use crate::i18n::t;
use crate::theme::{text_scaled,
    secondary_button_style, selected_button_style, sidebar_style, text_color,
    text_primary_color, text_secondary_color,
    Colors,
};
use iced::{
    widget::{button, column, container, row, text, Space},
    Alignment, Element, Length, Padding,
};
use iced_fonts::BOOTSTRAP_FONT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavItem {
    Dashboard,
    Wallets,
    Send,
    Receive,
    History,
    Settings,
}

impl NavItem {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Dashboard,
            Self::Wallets,
            Self::Send,
            Self::Receive,
            Self::History,
            Self::Settings,
        ]
    }

    pub fn icon_char(self) -> String {
        match self {
            Self::Dashboard => iced_fonts::Bootstrap::Speedometer,
            Self::Wallets => iced_fonts::Bootstrap::Wallet,
            Self::Send => iced_fonts::Bootstrap::BoxArrowUp,
            Self::Receive => iced_fonts::Bootstrap::BoxArrowDown,
            Self::History => iced_fonts::Bootstrap::ClockHistory,
            Self::Settings => iced_fonts::Bootstrap::Gear,
        }
        .to_string()
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Dashboard => t("Tổng quan", "Dashboard"),
            Self::Wallets => t("Ví", "Wallets"),
            Self::Send => t("Gửi", "Send"),
            Self::Receive => t("Nhận", "Receive"),
            Self::History => t("Lịch sử", "History"),
            Self::Settings => t("Cài đặt", "Settings"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    Navigate(NavItem),
}

#[derive(Debug, Clone)]
pub enum SidebarEvent {
    Navigate(NavItem),
}

pub struct Sidebar {
    active: NavItem,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            active: NavItem::Dashboard,
        }
    }

    pub fn set_active(&mut self, item: NavItem) {
        self.active = item;
    }

    pub fn update(&mut self, message: SidebarMessage) -> SidebarEvent {
        match message {
            SidebarMessage::Navigate(item) => {
                self.active = item;
                SidebarEvent::Navigate(item)
            }
        }
    }

    pub fn view(&self, wallet_count: usize, compact: bool) -> Element<'_, SidebarMessage> {
        let logo = text_scaled("₿", 48).style(text_color(Colors::ACCENT_PURPLE));
        let icon_size = if compact { 16 } else { 20 };
        let title_size = if compact { 12 } else { 14 };
        let item_padding = if compact { 8 } else { 12 };
        let item_spacing = if compact { 6 } else { 8 };
        let logo_padding = if compact {
            iced::Padding::from([12, 20])
        } else {
            iced::Padding::from([20, 30])
        };

        let logo_container = container(logo)
            .padding(logo_padding)
            .center_x(Length::Fill);

        let nav_items: Element<_> = column(
            NavItem::all()
                .into_iter()
                .map(|item| {
                    let is_active = self.active == item;
                    let icon = text_scaled(item.icon_char(), icon_size as u16)
                        .font(BOOTSTRAP_FONT)
                        .style(text_color(if is_active {
                            Colors::ACCENT_TEAL
                        } else {
                            Colors::TEXT_SECONDARY
                        }));
                    let title = text_scaled(item.title(), title_size as u16).style(text_secondary_color());

                    let item_row = if item == NavItem::Wallets && wallet_count > 0 {
                        let badge = container(
                            text_scaled(format!("{}", wallet_count), 11)
                                .style(text_primary_color()),
                        )
                        .padding(Padding::from([2, 6]));

                        row![
                            icon, 
                            Space::with_width(8), 
                            title,
                            Space::with_width(8), 
                            badge
                        ]
                        .align_y(Alignment::Center)
                    } else {
                        row![icon, Space::with_width(12), title].align_y(Alignment::Center)
                    };

                    let style = if is_active {
                        selected_button_style()
                    } else {
                        secondary_button_style()
                    };

                    button(item_row)
                        .on_press(SidebarMessage::Navigate(item))
                        .padding(item_padding)
                        .width(Length::Fill)
                        .style(style)
                        .into()
                })
                .collect::<Vec<_>>(),
        )
        .spacing(item_spacing)
        .padding(Padding::from(if compact { 12 } else { 16 }))
        .into();

        let content = column![logo_container, Space::with_height(if compact { 12 } else { 20 }), nav_items,];

        container(content)
            .width(220)
            .height(Length::Fill)
            .style(sidebar_style())
            .into()
    }
}
