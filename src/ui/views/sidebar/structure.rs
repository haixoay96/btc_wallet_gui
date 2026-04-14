// ─── Sidebar ─────────────────────────────────────────────────────────────

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
        use iced_fonts::bootstrap::advanced_text;
        match self {
            Self::Dashboard => advanced_text::speedometer().0,
            Self::Wallets => advanced_text::wallet().0,
            Self::Send => advanced_text::box_arrow_up().0,
            Self::Receive => advanced_text::box_arrow_down().0,
            Self::History => advanced_text::clock().0,
            Self::Settings => advanced_text::gear().0,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Dashboard => crate::ui::i18n::t("Tổng quan", "Dashboard"),
            Self::Wallets => crate::ui::i18n::t("Ví", "Wallets"),
            Self::Send => crate::ui::i18n::t("Gửi", "Send"),
            Self::Receive => crate::ui::i18n::t("Nhận", "Receive"),
            Self::History => crate::ui::i18n::t("Lịch sử", "History"),
            Self::Settings => crate::ui::i18n::t("Cài đặt", "Settings"),
        }
    }
}

// ─── Messages ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    Navigate(NavItem),
    NavigatePrevious,
    NavigateNext,
}

#[derive(Debug, Clone)]
pub enum SidebarEvent {
    Navigate(NavItem),
}

// ─── Sidebar struct ──────────────────────────────────────────────────────

pub struct Sidebar {
    pub active: NavItem,
    pub focused: Option<usize>,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            active: NavItem::Dashboard,
            focused: None,
        }
    }

    pub fn set_active(&mut self, item: NavItem) {
        self.active = item;
    }
}
