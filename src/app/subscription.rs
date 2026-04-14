use std::time::Duration;

use iced::{Subscription, keyboard, keyboard::Event as KeyboardEvent, keyboard::Key};

use crate::app::structure::{App, AppMessage};
use crate::ui::components::network_status::DashboardNetworkMessage;
use crate::ui::components::price_widget::structure::PriceWidgetMessage;
use crate::ui::views::dashboard::DashboardMessage;
use crate::ui::views::sidebar::{NavItem, SidebarMessage};

impl App {
    pub fn subscription(&self) -> Subscription<AppMessage> {
        let keyboard_sub = keyboard::listen().map(|event: KeyboardEvent| {
            // Only handle key pressed events
            let (key, modifiers) = match &event {
                KeyboardEvent::KeyPressed { key, modifiers, .. } => (key.clone(), *modifiers),
                _ => return AppMessage::KeyboardNoOp,
            };

            // Priority 1: Help shortcuts (always available)
            if modifiers.control() && key == Key::Character("/".into()) {
                return AppMessage::ToggleShortcutsHelp;
            }
            if key == Key::Named(keyboard::key::Named::F1) {
                return AppMessage::ToggleShortcutsHelp;
            }

            // Priority 2: Esc to close popups (always available)
            if key == Key::Named(keyboard::key::Named::Escape) {
                return AppMessage::GlobalEscKey;
            }

            // Priority 3: Navigation keys (without Ctrl/Cmd)
            if !modifiers.control() && !modifiers.command() {
                match &key {
                    Key::Named(keyboard::key::Named::Tab) => {
                        // Tab navigation - will be handled by widget
                    }
                    Key::Named(keyboard::key::Named::Enter)
                    | Key::Named(keyboard::key::Named::Space) => {
                        // Activate focused element
                        return AppMessage::KeyboardSubmitForm;
                    }
                    // Arrow keys for sidebar navigation
                    Key::Named(keyboard::key::Named::ArrowUp) => {
                        return AppMessage::SidebarMessage(SidebarMessage::NavigatePrevious);
                    }
                    Key::Named(keyboard::key::Named::ArrowDown) => {
                        return AppMessage::SidebarMessage(SidebarMessage::NavigateNext);
                    }
                    _ => {}
                }
            }

            // Priority 4: Form and action shortcuts (with Ctrl/Cmd)
            if modifiers.control() || modifiers.command() {
                match &key {
                    // Navigation shortcuts (Ctrl+1-6)
                    Key::Character(c) if c == "1" => {
                        return AppMessage::SidebarMessage(SidebarMessage::Navigate(
                            NavItem::Dashboard,
                        ));
                    }
                    Key::Character(c) if c == "2" => {
                        return AppMessage::SidebarMessage(SidebarMessage::Navigate(
                            NavItem::Wallets,
                        ));
                    }
                    Key::Character(c) if c == "3" => {
                        return AppMessage::SidebarMessage(SidebarMessage::Navigate(NavItem::Send));
                    }
                    Key::Character(c) if c == "4" => {
                        return AppMessage::SidebarMessage(SidebarMessage::Navigate(
                            NavItem::Receive,
                        ));
                    }
                    Key::Character(c) if c == "5" => {
                        return AppMessage::SidebarMessage(SidebarMessage::Navigate(
                            NavItem::History,
                        ));
                    }
                    Key::Character(c) if c == "6" => {
                        return AppMessage::SidebarMessage(SidebarMessage::Navigate(
                            NavItem::Settings,
                        ));
                    }
                    // Copy shortcut (Ctrl+C)
                    Key::Character(c) if c == "c" || c == "C" => {
                        return AppMessage::KeyboardCopy;
                    }
                    // Paste shortcut (Ctrl+V)
                    Key::Character(c) if c == "v" || c == "V" => {
                        return AppMessage::KeyboardPaste;
                    }
                    // Save state (Ctrl+S)
                    Key::Character(c) if c == "s" || c == "S" => {
                        return AppMessage::KeyboardSaveState;
                    }
                    // Focus search (Ctrl+F)
                    Key::Character(c) if c == "f" || c == "F" => {
                        return AppMessage::KeyboardFocusSearch;
                    }
                    _ => {}
                }
            }

            // No-op for unhandled events
            AppMessage::KeyboardNoOp
        });

        // Auto-refresh confirmations - interval based on setting
        let refresh_interval_secs = if let Ok(storage) = crate::infra::storage::Storage::new() {
            if storage.load_auto_refresh().unwrap_or(false) {
                120 // 2 minutes
            } else {
                0 // Disabled
            }
        } else {
            120
        };

        let refresh_sub = if refresh_interval_secs > 0 {
            iced::time::every(Duration::from_secs(refresh_interval_secs))
                .map(|_| AppMessage::AutoRefreshConfirmations)
        } else {
            Subscription::none()
        };

        // Network status check every 60 seconds
        let network_check_sub = iced::time::every(Duration::from_secs(60)).map(|_| {
            AppMessage::DashboardMessage(DashboardMessage::Network(
                DashboardNetworkMessage::CheckConnection,
            ))
        });

        // BTC Price auto-refresh every 5 minutes
        let price_refresh_sub = iced::time::every(Duration::from_secs(300)).map(|_| {
            AppMessage::DashboardMessage(DashboardMessage::PriceWidget(
                PriceWidgetMessage::RefreshPrice,
            ))
        });

        Subscription::batch([
            keyboard_sub,
            refresh_sub,
            network_check_sub,
            price_refresh_sub,
        ])
    }
}
