use iced::{
    widget::{
        button, column, container, pick_list, row, scrollable, slider, text, text_input, Space,
    },
    Alignment, Element, Length,
};

use crate::infra::storage::AppTheme;
use crate::ui::components::modal;
use crate::ui::i18n::t;
use crate::ui::theme::{
    card_style, danger_button_style, info_style, input_style, notice_style, pick_list_menu_style,
    pick_list_style, primary_button_style, secondary_button_style, text_color, text_muted_color,
    text_primary_color, text_secondary_color, warning_style, Colors, NoticeTone,
};

use super::structure::*;

impl SettingsView {
    pub fn new() -> Self {
        let debug_logging = if let Ok(storage) = crate::infra::storage::Storage::new() {
            let value = storage.load_enable_debug().unwrap_or(false);
            tracing::info!(
                settings_view_init = true,
                debug_logging_loaded = value,
                "SettingsView initialized - debug_logging from storage"
            );
            value
        } else {
            tracing::warn!("Failed to create Storage instance for loading debug_logging");
            false
        };
        let auto_refresh = if let Ok(storage) = crate::infra::storage::Storage::new() {
            let value = storage.load_auto_refresh().unwrap_or(false);
            tracing::info!(
                settings_view_init = true,
                auto_refresh_loaded = value,
                "SettingsView initialized - auto_refresh from storage"
            );
            value
        } else {
            tracing::warn!("Failed to create Storage instance for loading auto_refresh");
            false
        };
        let show_satoshis = if let Ok(storage) = crate::infra::storage::Storage::new() {
            let value = storage.load_show_satoshis().unwrap_or(false);
            tracing::info!(
                settings_view_init = true,
                show_satoshis_loaded = value,
                "SettingsView initialized - show_satoshis from storage"
            );
            value
        } else {
            tracing::warn!("Failed to create Storage instance for loading show_satoshis");
            false
        };
        let compact_mode = if let Ok(storage) = crate::infra::storage::Storage::new() {
            let value = storage.load_compact_mode().unwrap_or(false);
            tracing::info!(
                settings_view_init = true,
                compact_mode_loaded = value,
                "SettingsView initialized - compact_mode from storage"
            );
            value
        } else {
            tracing::warn!("Failed to create Storage instance for loading compact_mode");
            false
        };
        let esplora_endpoint = if let Ok(storage) = crate::infra::storage::Storage::new() {
            let value = storage
                .load_esplora_endpoint()
                .unwrap_or_else(|_| "https://blockstream.info/api".to_string());
            tracing::info!(
                settings_view_init = true,
                endpoint = %value,
                "SettingsView initialized - esplora_endpoint from storage"
            );
            value
        } else {
            tracing::warn!("Failed to create Storage instance for loading esplora_endpoint");
            "https://blockstream.info/api".to_string()
        };
        let timeout_secs = if let Ok(storage) = crate::infra::storage::Storage::new() {
            let value = storage.load_timeout_secs().unwrap_or(15);
            tracing::info!(
                settings_view_init = true,
                timeout_secs_loaded = value,
                "SettingsView initialized - timeout_secs from storage"
            );
            value
        } else {
            tracing::warn!("Failed to create Storage instance for loading timeout_secs");
            15
        };
        let font_scale = if let Ok(storage) = crate::infra::storage::Storage::new() {
            let value = storage.load_font_scale().unwrap_or(1.0);
            tracing::info!(
                settings_view_init = true,
                font_scale_loaded = value,
                "SettingsView initialized - font_scale from storage"
            );
            value
        } else {
            tracing::warn!("Failed to create Storage instance for loading font_scale");
            1.0
        };
        let high_contrast = if let Ok(storage) = crate::infra::storage::Storage::new() {
            let value = storage.load_high_contrast().unwrap_or(false);
            tracing::info!(
                settings_view_init = true,
                high_contrast_loaded = value,
                "SettingsView initialized - high_contrast from storage"
            );
            value
        } else {
            tracing::warn!("Failed to create Storage instance for loading high_contrast");
            false
        };

        Self {
            show_change_passphrase: false,
            current_passphrase: String::new(),
            new_passphrase: String::new(),
            confirm_passphrase: String::new(),
            show_about: false,
            show_clear_data_confirm: false,
            clear_data_passphrase: String::new(),
            error: None,
            success: None,
            font_scale,
            high_contrast,
            esplora_endpoint,
            timeout_secs,
            testing_connection: false,
            connection_test_result: None,
            debug_logging,
            auto_refresh,
            show_satoshis,
            compact_mode,
            data_folder_path: String::new(),
            data_folder_size: String::new(),
        }
    }

    pub fn load_data_folder_info(&mut self) {
        if let Ok(storage) = crate::infra::storage::Storage::new() {
            let path = storage.file_path();
            self.data_folder_path = path
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            let mut total_size: u64 = 0;
            if let Ok(entries) =
                std::fs::read_dir(path.parent().unwrap_or_else(|| std::path::Path::new(".")))
            {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                }
            }
            if total_size < 1024 {
                self.data_folder_size = format!("{} B", total_size);
            } else if total_size < 1024 * 1024 {
                self.data_folder_size = format!("{:.1} KB", total_size as f64 / 1024.0);
            } else {
                self.data_folder_size = format!("{:.1} MB", total_size as f64 / (1024.0 * 1024.0));
            }
        }
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error = Some(message.into());
        self.success = None;
    }

    pub fn set_success(&mut self, message: impl Into<String>) {
        self.success = Some(message.into());
        self.error = None;
    }

    pub fn clear_sensitive_inputs(&mut self) {
        self.current_passphrase.clear();
        self.new_passphrase.clear();
        self.confirm_passphrase.clear();
        self.clear_data_passphrase.clear();
    }

    pub fn update(&mut self, message: SettingsMessage) -> Option<SettingsEvent> {
        match message {
            SettingsMessage::ToggleChangePassphrase => {
                self.show_change_passphrase = !self.show_change_passphrase;
                self.error = None;
                self.success = None;
                None
            }
            SettingsMessage::CurrentPassphraseChanged(p) => {
                self.current_passphrase = p;
                self.error = None;
                None
            }
            SettingsMessage::NewPassphraseChanged(p) => {
                self.new_passphrase = p;
                self.error = None;
                None
            }
            SettingsMessage::ConfirmPassphraseChanged(p) => {
                self.confirm_passphrase = p;
                self.error = None;
                None
            }
            SettingsMessage::SubmitPassphraseChange => {
                if self.current_passphrase.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập passphrase hiện tại",
                            "Please enter your current passphrase",
                        )
                        .to_string(),
                    );
                    return None;
                }
                if self.new_passphrase.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập passphrase mới",
                            "Please enter a new passphrase",
                        )
                        .to_string(),
                    );
                    return None;
                }
                if self.new_passphrase != self.confirm_passphrase {
                    self.error = Some(
                        t(
                            "Passphrase mới và xác nhận không khớp",
                            "New passphrase and confirmation do not match",
                        )
                        .to_string(),
                    );
                    return None;
                }
                self.error = None;
                self.success = None;
                Some(SettingsEvent::ChangePassphrase {
                    current: self.current_passphrase.clone(),
                    new_passphrase: self.new_passphrase.clone(),
                })
            }
            SettingsMessage::ExportWallet => {
                self.error = None;
                self.success = None;
                Some(SettingsEvent::ExportWallet)
            }
            SettingsMessage::ToggleAbout => {
                self.show_about = !self.show_about;
                None
            }
            SettingsMessage::ToggleClearDataConfirm => {
                self.show_clear_data_confirm = !self.show_clear_data_confirm;
                if !self.show_clear_data_confirm {
                    self.clear_data_passphrase.clear();
                }
                self.error = None;
                self.success = None;
                None
            }
            SettingsMessage::ClearDataPassphraseChanged(value) => {
                self.clear_data_passphrase = value;
                self.error = None;
                None
            }
            SettingsMessage::ConfirmClearData => {
                if self.clear_data_passphrase.trim().is_empty() {
                    self.error = Some(
                        t(
                            "Vui lòng nhập passphrase hiện tại để xác nhận",
                            "Please enter your current passphrase to confirm",
                        )
                        .to_string(),
                    );
                    return None;
                }
                self.show_clear_data_confirm = false;
                self.error = None;
                self.success = None;
                Some(SettingsEvent::ClearAllData(
                    self.clear_data_passphrase.clone(),
                ))
            }
            SettingsMessage::CancelClearData => {
                self.show_clear_data_confirm = false;
                self.clear_data_passphrase.clear();
                None
            }
            SettingsMessage::ThemeSelected(theme) => Some(SettingsEvent::ThemeChanged(theme)),
            SettingsMessage::ShowOnboardingTour => Some(SettingsEvent::ShowOnboardingTour),
            SettingsMessage::FontScaleChanged(scale) => {
                self.font_scale = scale.clamp(0.8, 1.5); // Update immediately for real-time feedback
                Some(SettingsEvent::FontScaleChanged(scale))
            }
            SettingsMessage::HighContrastToggled(enabled) => {
                self.high_contrast = enabled; // Update immediately for real-time feedback
                Some(SettingsEvent::HighContrastToggled(enabled))
            }
            SettingsMessage::EsploraEndpointChanged(endpoint) => {
                self.esplora_endpoint = endpoint;
                None
            }
            SettingsMessage::TimeoutSecsChanged(secs) => {
                self.timeout_secs = secs;
                Some(SettingsEvent::TimeoutSecsChanged(secs))
            }
            SettingsMessage::TestConnection => {
                self.testing_connection = true;
                Some(SettingsEvent::TestConnection)
            }
            SettingsMessage::DebugLoggingToggled(enabled) => {
                self.debug_logging = enabled;
                Some(SettingsEvent::DebugLoggingToggled(enabled))
            }
            SettingsMessage::AutoRefreshToggled(enabled) => {
                self.auto_refresh = enabled;
                Some(SettingsEvent::AutoRefreshToggled(enabled))
            }
            SettingsMessage::ShowSatoshisToggled(enabled) => {
                self.show_satoshis = enabled;
                Some(SettingsEvent::ShowSatoshisToggled(enabled))
            }
            SettingsMessage::CompactModeToggled(enabled) => {
                self.compact_mode = enabled;
                Some(SettingsEvent::CompactModeToggled(enabled))
            }
            SettingsMessage::TestConnectionSuccess(result) => {
                self.testing_connection = false;
                self.connection_test_result = Some(result);
                None
            }
            SettingsMessage::TestConnectionFailed(error) => {
                self.testing_connection = false;
                self.connection_test_result = Some(error);
                None
            }
            SettingsMessage::ResetAllSettings => Some(SettingsEvent::ResetAllSettings),
        }
    }

    pub fn view(
        &self,
        current_theme: AppTheme,
        _font_scale: f64,
        _high_contrast: bool,
        compact_mode: bool,
    ) -> Element<'_, SettingsMessage> {
        // CRITICAL: Use self.font_scale instead of parameter for real-time slider updates
        // The parameter is from App which is updated AFTER event handling
        // self.font_scale is updated IMMEDIATELY in update() for real-time feedback
        let scale = self.font_scale;
        let s = |size: u16| -> u16 { (size as f64 * scale).round() as u16 };
        let title = text(t("Cài đặt", "Settings"))
            .size(s(32))
            .style(text_primary_color());

        // Apply compact mode to main container padding and spacing
        let main_padding = if compact_mode { 16 } else { 32 };
        let main_spacing = if compact_mode { 12 } else { 20 };

        let mut content = column![title]
            .spacing((main_spacing as f64 * scale) as f32)
            .padding(s(main_padding));

        // Appearance / Theme Section
        let theme_options: Vec<AppTheme> = vec![AppTheme::Dark, AppTheme::Light, AppTheme::System];
        content = content.push(
            container(column![
                text(t("Giao diện", "Appearance"))
                    .size(s(18))
                    .style(text_primary_color()),
                Space::with_height(8),
                pick_list(
                    theme_options,
                    Some(current_theme),
                    SettingsMessage::ThemeSelected
                )
                .padding(main_padding / 2)
                .style(pick_list_style())
                .menu_style(pick_list_menu_style()),
            ])
            .style(card_style())
            .padding(main_padding / 2)
            .width(Length::Fill),
        );

        // Data Storage Section
        content = content.push(
            container(column![
                text(t("Dữ liệu", "Data Storage"))
                    .size(s(18))
                    .style(text_primary_color()),
                Space::with_height(4),
                row![
                    text(t("Thư mục:", "Folder:"))
                        .size(s(12))
                        .style(text_secondary_color()),
                    Space::with_width(8),
                    text(&self.data_folder_path)
                        .size(s(11))
                        .style(text_muted_color()),
                ]
                .align_y(Alignment::Center),
                Space::with_height(4),
                row![
                    text(t("Dung lượng:", "Size:"))
                        .size(s(12))
                        .style(text_secondary_color()),
                    Space::with_width(8),
                    text(&self.data_folder_size)
                        .size(s(12))
                        .style(text_color(Colors::ACCENT_TEAL)),
                ]
                .align_y(Alignment::Center),
            ])
            .style(card_style())
            .padding(main_padding / 2)
            .width(Length::Fill),
        );

        // Accessibility Section
        let font_scale_percent = (self.font_scale * 100.0) as i32;
        content = content.push(
            container(column![
                text(t("Trợ năng", "Accessibility"))
                    .size(s(18))
                    .style(text_primary_color()),
                Space::with_height(12),
                row![
                    text(t("Cỡ chữ:", "Font Size:"))
                        .size(s(12))
                        .style(text_secondary_color()),
                    Space::with_width(8),
                    text(format!("{}%", font_scale_percent))
                        .size(s(13))
                        .style(text_color(Colors::ACCENT_TEAL)),
                ]
                .align_y(Alignment::Center),
                Space::with_height(8),
                slider(
                    0.8_f64..=1.5_f64,
                    self.font_scale,
                    SettingsMessage::FontScaleChanged
                )
                .step(0.05_f64)
                .height(20)
                .width(Length::Fill),
                Space::with_height(12),
                row![
                    text(t("Tương phản cao", "High Contrast"))
                        .size(s(13))
                        .style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    button(
                        text(if self.high_contrast {
                            t("Bật", "ON")
                        } else {
                            t("Tắt", "OFF")
                        })
                        .size(s(12))
                    )
                    .on_press(SettingsMessage::HighContrastToggled(!self.high_contrast))
                    .padding([8, 16])
                    .style(if self.high_contrast {
                        primary_button_style()
                    } else {
                        secondary_button_style()
                    }),
                ]
                .align_y(Alignment::Center),
            ])
            .style(card_style())
            .padding(16)
            .width(Length::Fill),
        );

        // Network Settings Section
        content = content.push(
            container(column![
                text(t("Mạng lưới", "Network"))
                    .size(s(18))
                    .style(text_primary_color()),
                Space::with_height(12),
                text(t("Esplora Endpoint", "Esplora Endpoint"))
                    .size(s(12))
                    .style(text_secondary_color()),
                Space::with_height(4),
                text_input(
                    t("Nhập URL Esplora...", "Enter Esplora URL..."),
                    &self.esplora_endpoint
                )
                .on_input(SettingsMessage::EsploraEndpointChanged)
                .padding(10)
                .size(s(13))
                .style(input_style()),
                Space::with_height(8),
                row![
                    text(t("Timeout:", "Timeout:"))
                        .size(s(12))
                        .style(text_secondary_color()),
                    Space::with_width(8),
                    pick_list(
                        vec![5u64, 10, 15, 30],
                        Some(self.timeout_secs),
                        SettingsMessage::TimeoutSecsChanged
                    )
                    .padding(8)
                    .style(pick_list_style()),
                    Space::with_width(Length::Fill),
                    button(
                        text(if self.testing_connection {
                            t("Đang test...", "Testing...")
                        } else {
                            t("Test", "Test")
                        })
                        .size(s(11))
                    )
                    .on_press(SettingsMessage::TestConnection)
                    .padding([6, 10])
                    .style(secondary_button_style()),
                ]
                .align_y(Alignment::Center),
                Space::with_height(4),
                if self.testing_connection {
                    text(t("⏳ Đang kiểm tra kết nối...", "⏳ Testing connection..."))
                        .size(s(11))
                        .style(text_color(Colors::ACCENT_TEAL))
                } else if let Some(result) = &self.connection_test_result {
                    text(result).size(s(11)).style(text_muted_color())
                } else {
                    text("").size(s(11))
                }
            ])
            .style(card_style())
            .padding(16)
            .width(Length::Fill),
        );

        // Advanced Options Section
        let card_padding = if self.compact_mode { 12 } else { 16 };
        let row_spacing = if self.compact_mode { 4 } else { 8 };
        content = content.push(
            container(column![
                text(t("Nâng cao", "Advanced"))
                    .size(s(18))
                    .style(text_primary_color()),
                Space::with_height(12),
                row![
                    text(t("Debug logging", "Debug logging"))
                        .size(s(13))
                        .style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    button(
                        text(if self.debug_logging {
                            t("Bật", "ON")
                        } else {
                            t("Tắt", "OFF")
                        })
                        .size(s(12))
                    )
                    .on_press(SettingsMessage::DebugLoggingToggled(!self.debug_logging))
                    .padding([6, 12])
                    .style(if self.debug_logging {
                        primary_button_style()
                    } else {
                        secondary_button_style()
                    }),
                ]
                .align_y(Alignment::Center),
                Space::with_height(row_spacing),
                row![
                    text(t("Tự động refresh", "Auto-refresh"))
                        .size(s(13))
                        .style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    button(
                        text(if self.auto_refresh {
                            t("Bật", "ON")
                        } else {
                            t("Tắt", "OFF")
                        })
                        .size(s(12))
                    )
                    .on_press(SettingsMessage::AutoRefreshToggled(!self.auto_refresh))
                    .padding([6, 12])
                    .style(if self.auto_refresh {
                        primary_button_style()
                    } else {
                        secondary_button_style()
                    }),
                ]
                .align_y(Alignment::Center),
                Space::with_height(row_spacing),
                row![
                    text(t("Hiện satoshi", "Show satoshis"))
                        .size(s(13))
                        .style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    button(
                        text(if self.show_satoshis {
                            t("Bật", "ON")
                        } else {
                            t("Tắt", "OFF")
                        })
                        .size(s(12))
                    )
                    .on_press(SettingsMessage::ShowSatoshisToggled(!self.show_satoshis))
                    .padding([6, 12])
                    .style(if self.show_satoshis {
                        primary_button_style()
                    } else {
                        secondary_button_style()
                    }),
                ]
                .align_y(Alignment::Center),
                Space::with_height(row_spacing),
                row![
                    text(t("Chế độ gọn", "Compact mode"))
                        .size(s(13))
                        .style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    button(
                        text(if self.compact_mode {
                            t("Bật", "ON")
                        } else {
                            t("Tắt", "OFF")
                        })
                        .size(s(12))
                    )
                    .on_press(SettingsMessage::CompactModeToggled(!self.compact_mode))
                    .padding([6, 12])
                    .style(if self.compact_mode {
                        primary_button_style()
                    } else {
                        secondary_button_style()
                    }),
                ]
                .align_y(Alignment::Center),
            ])
            .style(card_style())
            .padding(card_padding)
            .width(Length::Fill),
        );

        // Security Section
        let change_passphrase_btn =
            button(text(t("Đổi passphrase", "Change Passphrase")).size(s(16)))
                .on_press(SettingsMessage::ToggleChangePassphrase)
                .padding(12)
                .style(secondary_button_style());
        content = content.push(
            container(column![
                text(t("Bảo mật", "Security"))
                    .size(s(18))
                    .style(text_primary_color()),
                Space::with_height(12),
                change_passphrase_btn,
            ])
            .style(card_style())
            .padding(16)
            .width(Length::Fill),
        );

        if self.show_change_passphrase {
            content = content.push(
                container(column![
                    column![
                        text(t("Passphrase hiện tại", "Current Passphrase"))
                            .size(s(12))
                            .style(text_secondary_color()),
                        Space::with_height(4),
                        text_input(
                            t("Nhập passphrase hiện tại...", "Enter current passphrase..."),
                            &self.current_passphrase
                        )
                        .on_input(SettingsMessage::CurrentPassphraseChanged)
                        .secure(true)
                        .padding(10)
                        .size(s(14))
                        .style(input_style())
                    ]
                    .spacing(2),
                    Space::with_height(12),
                    column![
                        text(t("Passphrase mới", "New Passphrase"))
                            .size(s(12))
                            .style(text_secondary_color()),
                        Space::with_height(4),
                        text_input(
                            t("Nhập passphrase mới...", "Enter new passphrase..."),
                            &self.new_passphrase
                        )
                        .on_input(SettingsMessage::NewPassphraseChanged)
                        .secure(true)
                        .padding(10)
                        .size(s(14))
                        .style(input_style())
                    ]
                    .spacing(2),
                    Space::with_height(12),
                    column![
                        text(t("Xác nhận passphrase mới", "Confirm New Passphrase"))
                            .size(s(12))
                            .style(text_secondary_color()),
                        Space::with_height(4),
                        text_input(
                            t("Xác nhận passphrase mới...", "Confirm new passphrase..."),
                            &self.confirm_passphrase
                        )
                        .on_input(SettingsMessage::ConfirmPassphraseChanged)
                        .secure(true)
                        .padding(10)
                        .size(s(14))
                        .style(input_style())
                    ]
                    .spacing(2),
                    Space::with_height(12),
                    button(text(t("Cập nhật passphrase", "Update Passphrase")).size(s(14)))
                        .on_press(SettingsMessage::SubmitPassphraseChange)
                        .padding(12)
                        .style(primary_button_style()),
                ])
                .style(card_style())
                .padding(16)
                .width(Length::Fill),
            );
        }

        // Export Backup Section
        content = content.push(
            container(column![
                text(t("Xuất backup", "Export Backup"))
                    .size(s(18))
                    .style(text_primary_color()),
                Space::with_height(8),
                text(t(
                    "Backup sẽ được mã hóa bằng passphrase hiện tại",
                    "Backup will be encrypted with the current passphrase"
                ))
                .size(s(12))
                .style(text_secondary_color()),
                Space::with_height(10),
                button(text(t("Xuất backup ví", "Export Wallet Backup")).size(s(14)))
                    .on_press(SettingsMessage::ExportWallet)
                    .padding(12)
                    .style(secondary_button_style()),
            ])
            .style(card_style())
            .padding(16)
            .width(Length::Fill),
        );

        // Danger Zone
        let clear_data_button =
            button(text(t("Xóa toàn bộ dữ liệu ví", "Clear All Wallet Data")).size(s(13)))
                .on_press(SettingsMessage::ToggleClearDataConfirm)
                .padding([10, 18])
                .style(warning_style());
        let reset_settings_btn = button(text(t("Đặt lại cài đặt", "Reset Settings")).size(s(13)))
            .on_press(SettingsMessage::ResetAllSettings)
            .padding([10, 18])
            .style(secondary_button_style());
        let mut clear_data_col = column![
            text(t("Vùng nguy hiểm", "Danger Zone"))
                .size(s(16))
                .style(text_color(Colors::ERROR)),
            Space::with_height(16),
            row![clear_data_button, reset_settings_btn,]
                .spacing(10)
                .align_y(Alignment::Center),
        ]
        .spacing(0);

        if self.show_clear_data_confirm {
            clear_data_col = clear_data_col.push(
                column![
                    Space::with_height(12),
                    text(t("Thao tác này sẽ xóa toàn bộ ví khỏi máy hiện tại.", "This action will remove every wallet from the current device.")).size(s(13)).style(text_color(Colors::ERROR)),
                    text(t("Bạn sẽ cần app backup hoặc các secret backup riêng của từng ví để khôi phục lại sau này.", "You will need the app backup or each wallet's own secret backup to restore later.")).size(s(12)).style(text_secondary_color()),
                ].spacing(4),
            );
        }
        content = content.push(
            container(clear_data_col)
                .style(card_style())
                .padding(20)
                .width(Length::Fill),
        );

        // Information Section
        let tour_btn = button(text(t("Xem hướng dẫn", "Show Onboarding Tour")).size(s(13)))
            .on_press(SettingsMessage::ShowOnboardingTour)
            .padding([10, 18])
            .style(secondary_button_style());
        let about_btn = button(text(t("Giới thiệu", "About")).size(s(13)))
            .on_press(SettingsMessage::ToggleAbout)
            .padding([10, 18])
            .style(info_style());

        let mut info_col = column![
            text(t("Thông tin", "Information"))
                .size(s(16))
                .style(text_primary_color()),
            Space::with_height(16),
            row![tour_btn, about_btn]
                .spacing(10)
                .align_y(Alignment::Center),
        ]
        .spacing(0);

        if self.show_about {
            info_col = info_col
                .push(Space::with_height(12))
                .push(
                    text("Bitcoin Wallet GUI v0.1.0")
                        .size(s(12))
                        .style(text_muted_color()),
                )
                .push(
                    text(t("Xây dựng với iced.rs", "Built with iced.rs"))
                        .size(s(12))
                        .style(text_muted_color()),
                );
        }
        content = content.push(
            container(info_col)
                .style(card_style())
                .padding(20)
                .width(Length::Fill),
        );

        // Error/Success messages
        if let Some(err) = &self.error {
            content = content.push(
                container(text(err.as_str()).size(s(13)).style(text_primary_color()))
                    .style(notice_style(NoticeTone::Error))
                    .padding(12)
                    .width(Length::Fill),
            );
        }
        if let Some(succ) = &self.success {
            content = content.push(
                container(text(succ.as_str()).size(s(13)).style(text_primary_color()))
                    .style(notice_style(NoticeTone::Success))
                    .padding(12)
                    .width(Length::Fill),
            );
        }

        let base: Element<'_, SettingsMessage> = scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        if self.show_clear_data_confirm {
            let clear_content = column![
                text(t(
                    "Dữ liệu ví trên máy này sẽ bị xóa vĩnh viễn.",
                    "Wallet data on this device will be permanently deleted."
                ))
                .size(s(14))
                .style(text_primary_color()),
                Space::with_height(8),
                text_input(
                    t("Nhập passphrase hiện tại...", "Enter current passphrase..."),
                    &self.clear_data_passphrase
                )
                .on_input(SettingsMessage::ClearDataPassphraseChanged)
                .secure(true)
                .padding(12)
                .size(s(14)),
                Space::with_height(12),
                container(
                    row![
                        button(text(t("Hủy", "Cancel")).size(s(14)))
                            .on_press(SettingsMessage::CancelClearData)
                            .padding(10)
                            .style(secondary_button_style()),
                        Space::with_width(10),
                        button(text(t("Xóa toàn bộ ngay", "Delete Everything")).size(s(14)))
                            .on_press(SettingsMessage::ConfirmClearData)
                            .padding(10)
                            .style(danger_button_style()),
                    ]
                    .spacing(8)
                )
                .width(Length::Fill)
                .align_x(Alignment::Center),
            ]
            .padding(0)
            .spacing(0);
            return modal(
                base,
                t("Xác nhận xóa toàn bộ", "Confirm Full Data Deletion"),
                clear_content.into(),
                SettingsMessage::CancelClearData,
                compact_mode,
                true, // close_on_backdrop: true
            );
        }
        base
    }
}
