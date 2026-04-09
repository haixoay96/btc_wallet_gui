use std::time::Duration;

use iced::{keyboard, Subscription};

use crate::app::structure::{App, AppMessage};
use crate::infra::storage::Storage;
use crate::ui::views::sidebar::{NavItem, SidebarMessage};

impl App {
    pub fn subscription(&self) -> Subscription<AppMessage> {
        let keyboard_sub = keyboard::on_key_press(|key_code, modifiers| {
            // Priority 1: Help shortcuts (always available)
            if modifiers.control() && key_code == keyboard::Key::Character("/".into()) {
                return Some(AppMessage::ToggleShortcutsHelp);
            }
            if key_code == keyboard::Key::Named(keyboard::key::Named::F1) {
                return Some(AppMessage::ToggleShortcutsHelp);
            }

            // Priority 2: Esc to close popups (always available)
            if key_code == keyboard::Key::Named(keyboard::key::Named::Escape) {
                return Some(AppMessage::GlobalEscKey);
            }

            // Priority 3: Navigation keys (without Ctrl/Cmd)
            if !modifiers.control() && !modifiers.command() {
                match key_code {
                    keyboard::Key::Named(keyboard::key::Named::Tab) => {
                        // Tab navigation
                        return None;
                    }
                    keyboard::Key::Named(keyboard::key::Named::Enter)
                    | keyboard::Key::Named(keyboard::key::Named::Space) => {
                        // Activate focused element
                        return Some(AppMessage::KeyboardSubmitForm);
                    }
                    // Arrow keys for sidebar navigation
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::NavigatePrevious));
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::NavigateNext));
                    }
                    _ => {}
                }
            }

            // Priority 4: Form and action shortcuts (with Ctrl/Cmd)
            if modifiers.control() || modifiers.command() {
                match key_code {
                    // Navigation shortcuts (Ctrl+1-6)
                    keyboard::Key::Character(c) if c == "1" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(
                            NavItem::Dashboard,
                        )));
                    }
                    keyboard::Key::Character(c) if c == "2" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(
                            NavItem::Wallets,
                        )));
                    }
                    keyboard::Key::Character(c) if c == "3" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(
                            NavItem::Send,
                        )));
                    }
                    keyboard::Key::Character(c) if c == "4" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(
                            NavItem::Receive,
                        )));
                    }
                    keyboard::Key::Character(c) if c == "5" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(
                            NavItem::History,
                        )));
                    }
                    keyboard::Key::Character(c) if c == "6" => {
                        return Some(AppMessage::SidebarMessage(SidebarMessage::Navigate(
                            NavItem::Settings,
                        )));
                    }
                    // Copy shortcut (Ctrl+C)
                    keyboard::Key::Character(c) if c == "c" || c == "C" => {
                        return Some(AppMessage::KeyboardCopy);
                    }
                    // Paste shortcut (Ctrl+V)
                    keyboard::Key::Character(c) if c == "v" || c == "V" => {
                        return Some(AppMessage::KeyboardPaste);
                    }
                    // Save state (Ctrl+S)
                    keyboard::Key::Character(c) if c == "s" || c == "S" => {
                        return Some(AppMessage::KeyboardSaveState);
                    }
                    // Focus search (Ctrl+F)
                    keyboard::Key::Character(c) if c == "f" || c == "F" => {
                        return Some(AppMessage::KeyboardFocusSearch);
                    }
                    _ => {}
                }
            }

            None
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

        Subscription::batch([keyboard_sub, refresh_sub])
    }
}
