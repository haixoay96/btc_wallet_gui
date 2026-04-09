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
        vec![Self::Dashboard, Self::Wallets, Self::Send, Self::Receive, Self::History, Self::Settings]
    }

    pub fn icon_char(self) -> String {
        match self {
            Self::Dashboard => '\u{f080}'.to_string(),
            Self::Wallets => '\u{f09a}'.to_string(),
            Self::Send => '\u{f1d9}'.to_string(),
            Self::Receive => '\u{f1da}'.to_string(),
            Self::History => '\u{f1da}'.to_string(),
            Self::Settings => '\u{f013}'.to_string(),
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Dashboard => crate::i18n::t("Bảng điều khiển", "Dashboard"),
            Self::Wallets => crate::i18n::t("Ví", "Wallets"),
            Self::Send => crate::i18n::t("Gửi", "Send"),
            Self::Receive => crate::i18n::t("Nhận", "Receive"),
            Self::History => crate::i18n::t("Lịch sử", "History"),
            Self::Settings => crate::i18n::t("Cài đặt", "Settings"),
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
