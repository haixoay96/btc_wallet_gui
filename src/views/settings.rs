use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, pick_list, slider, Space},
    Alignment, Element, Length,
};

use crate::components::modal;
use crate::i18n::t;
use crate::storage::AppTheme;
use crate::theme::{text_scaled,
    card_style, danger_button_style, info_style, input_style, notice_style, primary_button_style,
    secondary_button_style, text_color, warning_style,
    text_primary_color, text_secondary_color, text_muted_color,
    Colors, NoticeTone,
    pick_list_style,
};

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    ToggleChangePassphrase,
    CurrentPassphraseChanged(String),
    NewPassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    SubmitPassphraseChange,
    ExportWallet,
    ToggleAbout,
    ToggleClearDataConfirm,
    ClearDataPassphraseChanged(String),
    ConfirmClearData,
    CancelClearData,
    ThemeSelected(AppTheme),
    ShowOnboardingTour,
    // Accessibility
    FontScaleChanged(f64),
    HighContrastToggled(bool),
    // Network
    EsploraEndpointChanged(String),
    TimeoutSecsChanged(u64),
    TestConnection,
    // Advanced
    DebugLoggingToggled(bool),
    AutoRefreshToggled(bool),
    ShowSatoshisToggled(bool),
    CompactModeToggled(bool),
}

#[derive(Debug, Clone)]
pub enum SettingsEvent {
    ChangePassphrase { current: String, new_passphrase: String },
    ExportWallet,
    ClearAllData(String),
    ThemeChanged(AppTheme),
    ShowOnboardingTour,
    // Accessibility
    FontScaleChanged(f64),
    HighContrastToggled(bool),
    // Network
    EsploraEndpointChanged(String),
    TimeoutSecsChanged(u64),
    TestConnection,
    // Advanced
    DebugLoggingToggled(bool),
    AutoRefreshToggled(bool),
    ShowSatoshisToggled(bool),
    CompactModeToggled(bool),
}

pub struct SettingsView {
    show_change_passphrase: bool,
    current_passphrase: String,
    new_passphrase: String,
    confirm_passphrase: String,
    show_about: bool,
    pub show_clear_data_confirm: bool,
    clear_data_passphrase: String,
    error: Option<String>,
    success: Option<String>,
    // Accessibility
    pub font_scale: f64,
    pub high_contrast: bool,
    // Network
    pub esplora_endpoint: String,
    pub timeout_secs: u64,
    pub testing_connection: bool,
    // Advanced
    pub debug_logging: bool,
    pub auto_refresh: bool,
    pub show_satoshis: bool,
    pub compact_mode: bool,
}

impl SettingsView {
    pub fn new() -> Self {
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
            font_scale: 1.0,
            high_contrast: false,
            esplora_endpoint: "https://blockstream.info/api".to_string(),
            timeout_secs: 15,
            testing_connection: false,
            debug_logging: false,
            auto_refresh: false,
            show_satoshis: false,
            compact_mode: false,
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
            SettingsMessage::ThemeSelected(theme) => {
                Some(SettingsEvent::ThemeChanged(theme))
            }
            SettingsMessage::ShowOnboardingTour => {
                Some(SettingsEvent::ShowOnboardingTour)
            }
            // Accessibility
            SettingsMessage::FontScaleChanged(scale) => {
                Some(SettingsEvent::FontScaleChanged(scale))
            }
            SettingsMessage::HighContrastToggled(enabled) => {
                Some(SettingsEvent::HighContrastToggled(enabled))
            }
            // Network
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
            // Advanced
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
        }
    }

    pub fn view(&self, current_theme: AppTheme, font_scale: f64, high_contrast: bool) -> Element<'_, SettingsMessage> {
        let title = text_scaled(t("Cài đặt", "Settings"), 32)
            .style(text_primary_color());

        let mut content = column![title].spacing(20).padding(32);

        // Appearance / Theme Section
        let theme_options: Vec<AppTheme> = vec![AppTheme::Dark, AppTheme::Light, AppTheme::System];
        let _theme_display = |theme: &AppTheme| -> &'static str {
            match theme {
                AppTheme::Dark => t("Tối", "Dark"),
                AppTheme::Light => t("Sáng", "Light"),
                AppTheme::System => t("Theo hệ thống", "System Default"),
            }
        };

        content = content.push(
            container(column![
                text_scaled(t("Giao diện", "Appearance"), 18)
                    .style(text_primary_color()),
                Space::with_height(8),
                text_scaled(t("Chọn màu sắc giao diện", "Choose your theme preference"), 12)
                    .style(text_secondary_color()),
                Space::with_height(8),
                pick_list(
                    theme_options,
                    Some(current_theme),
                    SettingsMessage::ThemeSelected
                )
                .padding(10)
                .style(pick_list_style()),
            ])
            .style(card_style())
            .padding(16)
            .width(Length::Fill),
        );

        // Accessibility Section
        let font_scale_percent = (font_scale * 100.0) as i32;
        content = content.push(
            container(column![
                text_scaled(t("Trợ năng", "Accessibility"), 18)
                    .style(text_primary_color()),
                Space::with_height(12),
                row![
                    text_scaled(t("Cỡ chữ:", "Font Size:"), 12).style(text_secondary_color()),
                    Space::with_width(8),
                    text_scaled(format!("{}%", font_scale_percent), 13).style(text_color(Colors::ACCENT_TEAL)),
                ].align_y(Alignment::Center),
                Space::with_height(8),
                slider(0.8_f64..=1.5_f64, font_scale, SettingsMessage::FontScaleChanged)
                    .step(0.05_f64)
                    .height(20)
                    .width(Length::Fill),
                Space::with_height(12),
                row![
                    text_scaled(t("Tương phản cao", "High Contrast"), 13).style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    button(
                        text_scaled(if high_contrast { t("Bật", "ON") } else { t("Tắt", "OFF") }, 12)
                    )
                    .on_press(SettingsMessage::HighContrastToggled(!high_contrast))
                    .padding([8, 16])
                    .style(if high_contrast { primary_button_style() } else { secondary_button_style() }),
                ].align_y(Alignment::Center),
            ])
            .style(card_style())
            .padding(16)
            .width(Length::Fill),
        );

        // Network Settings Section
        content = content.push(
            container(column![
                text_scaled(t("Mạng lưới", "Network"), 18)
                    .style(text_primary_color()),
                Space::with_height(12),
                text_scaled(t("Esplora Endpoint", "Esplora Endpoint"), 12)
                    .style(text_secondary_color()),
                Space::with_height(4),
                text_input(
                    t("Nhập URL Esplora...", "Enter Esplora URL..."),
                    &self.esplora_endpoint
                )
                .on_input(SettingsMessage::EsploraEndpointChanged)
                .padding(10)
                .size(13)
                .style(input_style()),
                Space::with_height(8),
                row![
                    text_scaled(t("Timeout:", "Timeout:"), 12).style(text_secondary_color()),
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
                        text_scaled(if self.testing_connection { t("Đang test...", "Testing...") } else { t("Test", "Test") }, 11)
                    )
                    .on_press(SettingsMessage::TestConnection)
                    .padding([6, 10])
                    .style(secondary_button_style()),
                ].align_y(Alignment::Center),
            ])
            .style(card_style())
            .padding(16)
            .width(Length::Fill),
        );

        // Advanced Options Section
        content = content.push(
            container(column![
                text_scaled(t("Nâng cao", "Advanced"), 18)
                    .style(text_primary_color()),
                Space::with_height(12),
                row![
                    text_scaled(t("Debug logging", "Debug logging"), 13).style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    button(
                        text_scaled(if self.debug_logging { t("Bật", "ON") } else { t("Tắt", "OFF") }, 12)
                    )
                    .on_press(SettingsMessage::DebugLoggingToggled(!self.debug_logging))
                    .padding([6, 12])
                    .style(if self.debug_logging { primary_button_style() } else { secondary_button_style() }),
                ].align_y(Alignment::Center),
                Space::with_height(8),
                row![
                    text_scaled(t("Tự động refresh", "Auto-refresh"), 13).style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    button(
                        text_scaled(if self.auto_refresh { t("Bật", "ON") } else { t("Tắt", "OFF") }, 12)
                    )
                    .on_press(SettingsMessage::AutoRefreshToggled(!self.auto_refresh))
                    .padding([6, 12])
                    .style(if self.auto_refresh { primary_button_style() } else { secondary_button_style() }),
                ].align_y(Alignment::Center),
                Space::with_height(8),
                row![
                    text_scaled(t("Hiện satoshi", "Show satoshis"), 13).style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    button(
                        text_scaled(if self.show_satoshis { t("Bật", "ON") } else { t("Tắt", "OFF") }, 12)
                    )
                    .on_press(SettingsMessage::ShowSatoshisToggled(!self.show_satoshis))
                    .padding([6, 12])
                    .style(if self.show_satoshis { primary_button_style() } else { secondary_button_style() }),
                ].align_y(Alignment::Center),
                Space::with_height(8),
                row![
                    text_scaled(t("Chế độ gọn", "Compact mode"), 13).style(text_primary_color()),
                    Space::with_width(Length::Fill),
                    button(
                        text_scaled(if self.compact_mode { t("Bật", "ON") } else { t("Tắt", "OFF") }, 12)
                    )
                    .on_press(SettingsMessage::CompactModeToggled(!self.compact_mode))
                    .padding([6, 12])
                    .style(if self.compact_mode { primary_button_style() } else { secondary_button_style() }),
                ].align_y(Alignment::Center),
            ])
            .style(card_style())
            .padding(16)
            .width(Length::Fill),
        );

        let change_passphrase_btn = button(text_scaled(t("Đổi passphrase", "Change Passphrase"), 16))
            .on_press(SettingsMessage::ToggleChangePassphrase)
            .padding(12)
            .style(secondary_button_style());

        content = content.push(
            container(column![
                text_scaled(t("Bảo mật", "Security"), 18)
                    .style(text_primary_color()),
                Space::with_height(12),
                change_passphrase_btn,
            ])
            .style(card_style())
            .padding(16)
            .width(Length::Fill),
        );

        if self.show_change_passphrase {
            let current_input = column![
                text_scaled(t("Passphrase hiện tại", "Current Passphrase"), 12)
                    .style(text_secondary_color()),
                Space::with_height(4),
                text_input(
                    t("Nhập passphrase hiện tại...", "Enter current passphrase..."),
                    &self.current_passphrase
                )
                .on_input(SettingsMessage::CurrentPassphraseChanged)
                .secure(true)
                .padding(10)
                .size(14)
                .style(input_style())
            ]
            .spacing(2);

            let new_input = column![
                text_scaled(t("Passphrase mới", "New Passphrase"), 12)
                    .style(text_secondary_color()),
                Space::with_height(4),
                text_input(
                    t("Nhập passphrase mới...", "Enter new passphrase..."),
                    &self.new_passphrase
                )
                .on_input(SettingsMessage::NewPassphraseChanged)
                .secure(true)
                .padding(10)
                .size(14)
                .style(input_style())
            ]
            .spacing(2);

            let confirm_input = column![
                text_scaled(t("Xác nhận passphrase mới", "Confirm New Passphrase"), 12)
                    .style(text_secondary_color()),
                Space::with_height(4),
                text_input(
                    t("Xác nhận passphrase mới...", "Confirm new passphrase..."),
                    &self.confirm_passphrase
                )
                .on_input(SettingsMessage::ConfirmPassphraseChanged)
                .secure(true)
                .padding(10)
                .size(14)
                .style(input_style())
            ]
            .spacing(2);

            content = content.push(
                container(column![
                    current_input,
                    Space::with_height(12),
                    new_input,
                    Space::with_height(12),
                    confirm_input,
                    Space::with_height(12),
                    button(text_scaled(t("Cập nhật passphrase", "Update Passphrase"), 14))
                        .on_press(SettingsMessage::SubmitPassphraseChange)
                        .padding(12)
                        .style(primary_button_style()),
                ])
                .style(card_style())
                .padding(16)
                .width(Length::Fill),
            );
        }

        let export_section = container(column![
            text_scaled(t("Xuất backup", "Export Backup"), 18)
                .style(text_primary_color()),
            Space::with_height(8),
            text_scaled(t(
                "Backup sẽ được mã hóa bằng passphrase hiện tại",
                "Backup will be encrypted with the current passphrase"
            ), 12)
                .style(text_secondary_color()),
            text_scaled(t(
                "Khuyến nghị: ưu tiên backup mnemonic cho từng wallet thay vì backup toàn app.",
                "Recommended: backup each wallet mnemonic instead of full app backup."
            ), 12)
                .style(text_color(Colors::WARNING)),
            text_scaled(t(
                "Khôi phục app backup chỉ dùng ở màn hình khởi tạo. Khôi phục ví .enc riêng lẻ dùng tại Wallets > Import.",
                "Full app backup restore is only for the startup screen. Individual .enc wallet restore is available in Wallets > Import."
            ), 12)
                .style(text_secondary_color()),
            Space::with_height(10),
            button(text_scaled(t("Xuất backup ví", "Export Wallet Backup"), 14))
                .on_press(SettingsMessage::ExportWallet)
                .padding(12)
                .style(secondary_button_style()),
        ])
        .style(card_style())
        .padding(16)
        .width(Length::Fill);

        content = content.push(export_section);

        let clear_data_button =
            button(text_scaled(t("Xóa toàn bộ dữ liệu ví", "Clear All Wallet Data"), 14))
                .on_press(SettingsMessage::ToggleClearDataConfirm)
                .padding(12)
                .style(warning_style());

        let mut clear_data_col = column![
            text_scaled(t("Vùng nguy hiểm", "Danger Zone"), 18)
                .style(text_color(Colors::ERROR)),
            Space::with_height(8),
            text_scaled(t(
                "Xóa toàn bộ ví và dữ liệu đã lưu trong ứng dụng",
                "Delete all wallets and saved app data"
            ), 12)
            .style(text_color(Colors::WARNING)),
            Space::with_height(10),
            clear_data_button,
        ]
        .spacing(6);

        if self.show_clear_data_confirm {
            clear_data_col = clear_data_col.push(
                column![
                    text_scaled(t(
                        "Thao tác này sẽ xóa toàn bộ ví khỏi máy hiện tại.",
                        "This action will remove every wallet from the current device."
                    ), 13)
                    .style(text_color(Colors::ERROR)),
                    text_scaled(t(
                        "Bạn sẽ cần app backup hoặc các secret backup riêng của từng ví để khôi phục lại sau này.",
                        "You will need the app backup or each wallet's own secret backup to restore later.",
                    ), 12)
                    .style(text_secondary_color()),
                ]
                .spacing(6),
            );
        }

        content = content.push(
            container(clear_data_col)
                .style(card_style())
                .padding(16)
                .width(Length::Fill),
        );

        let tour_btn = button(text_scaled(t("Xem hướng dẫn", "Show Onboarding Tour"), 14))
            .on_press(SettingsMessage::ShowOnboardingTour)
            .padding(12)
            .style(secondary_button_style());

        let about_btn = button(text_scaled(t("Giới thiệu", "About"), 16))
            .on_press(SettingsMessage::ToggleAbout)
            .padding(12)
            .style(info_style());

        let mut info_col = column![
            text_scaled(t("Thông tin", "Information"), 18)
                .style(text_primary_color()),
            Space::with_height(12),
            tour_btn,
            Space::with_height(8),
            about_btn,
        ]
        .spacing(8);

        if self.show_about {
            info_col = info_col
                .push(
                    text_scaled("Bitcoin Wallet GUI v0.1.0", 12)
                        .style(text_muted_color()),
                )
                .push(
                    text_scaled(t("Xây dựng với iced.rs", "Built with iced.rs"), 12)
                        .style(text_muted_color()),
                )
                .push(
                    text(t(
                        "Lưu trữ: backup mã hóa (ChaCha20-Poly1305 + Argon2id)",
                        "Storage: encrypted backup (ChaCha20-Poly1305 + Argon2id)",
                    ))
                    .size(12)
                    .style(text_muted_color()),
                );
        }

        content = content.push(
            container(info_col)
                .style(card_style())
                .padding(16)
                .width(Length::Fill),
        );

        if let Some(err) = &self.error {
            content = content.push(
                container(
                    text_scaled(err.as_str(), 13)
                        .style(text_primary_color()),
                )
                .style(notice_style(NoticeTone::Error))
                .padding(12)
                .width(Length::Fill),
            );
        }

        if let Some(succ) = &self.success {
            content = content.push(
                container(
                    text_scaled(succ.as_str(), 13)
                        .style(text_primary_color()),
                )
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
                text_scaled(t(
                    "Dữ liệu ví trên máy này sẽ bị xóa vĩnh viễn.",
                    "Wallet data on this device will be permanently deleted.",
                ), 14)
                .style(text_primary_color()),
                Space::with_height(8),
                text_input(
                    t("Nhập passphrase hiện tại...", "Enter current passphrase..."),
                    &self.clear_data_passphrase
                )
                .on_input(SettingsMessage::ClearDataPassphraseChanged)
                .secure(true)
                .padding(12)
                .size(14)
                .style(input_style()),
                Space::with_height(12),
                container(
                    row![
                        button(text_scaled(t("Hủy", "Cancel"), 14))
                            .on_press(SettingsMessage::CancelClearData)
                            .padding(10)
                            .style(secondary_button_style()),
                        Space::with_width(10),
                        button(text_scaled(t("Xóa toàn bộ ngay", "Delete Everything"), 14))
                            .on_press(SettingsMessage::ConfirmClearData)
                            .padding(10)
                            .style(danger_button_style()),
                    ]
                    .spacing(8),
                )
                .width(Length::Fill)
                .align_x(Alignment::Center),
            ]
            .padding(0)
            .spacing(0);

            return modal(
                base.into(),
                t("Xác nhận xóa toàn bộ", "Confirm Full Data Deletion"),
                clear_content.into(),
                SettingsMessage::CancelClearData,
            );
        }

        base
    }
}
