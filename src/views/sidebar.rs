use crate::i18n::t;
use crate::theme::{secondary_button_style, sidebar_style, text_color, Colors};
use iced::{
    widget::{button, column, container, row, text, Space},
    Alignment, Element, Length, Padding,
};

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

    pub fn icon(self) -> &'static str {
        match self {
            Self::Dashboard => "📊",
            Self::Wallets => "👛",
            Self::Send => "📤",
            Self::Receive => "📥",
            Self::History => "📜",
            Self::Settings => "⚙️",
        }
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

    pub fn view(&self) -> Element<'_, SidebarMessage> {
        let logo = text("₿").size(48).style(text_color(Colors::ACCENT_PURPLE));

        let logo_container = container(logo)
            .padding(Padding::from([20, 30]))
            .center_x(Length::Fill);

        let nav_items: Element<_> = column(
            NavItem::all()
                .into_iter()
                .map(|item| {
                    let is_active = self.active == item;
                    let icon = text(item.icon()).size(24);
                    let title = text(item.title()).size(14);

                    let style = if is_active {
                        crate::theme::primary_button_style()
                    } else {
                        secondary_button_style()
                    };

                    button(row![icon, Space::with_width(12), title].align_y(Alignment::Center))
                        .on_press(SidebarMessage::Navigate(item))
                        .padding(12)
                        .width(Length::Fill)
                        .style(style)
                        .into()
                })
                .collect::<Vec<_>>(),
        )
        .spacing(8)
        .padding(Padding::from(16))
        .into();

        let content = column![logo_container, Space::with_height(20), nav_items,];

        container(content)
            .width(220)
            .height(Length::Fill)
            .style(sidebar_style())
            .into()
    }
}
