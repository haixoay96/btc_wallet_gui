use iced::{
    widget::{button, column, container, row, text, Space},
    Alignment, Element, Length,
};

use crate::i18n::t;
use crate::theme::{
    card_style, primary_button_style, secondary_button_style,
    text_primary_color, text_secondary_color,
    get_theme_colors, Colors,
};

#[derive(Debug, Clone)]
pub enum OnboardingMessage {
    Next,
    Previous,
    Skip,
    Complete,
}

#[derive(Debug, Clone)]
pub enum OnboardingEvent {
    Finished,
    Skipped,
}

pub struct OnboardingStep {
    pub title_vi: &'static str,
    pub title_en: &'static str,
    pub description_vi: &'static str,
    pub description_en: &'static str,
}

pub struct OnboardingView {
    pub current_step: u8,
    pub total_steps: u8,
}

impl OnboardingView {
    pub fn new() -> Self {
        Self {
            current_step: 0,
            total_steps: 5,
        }
    }

    pub fn steps() -> Vec<OnboardingStep> {
        vec![
            OnboardingStep {
                title_vi: "Chào mừng!",
                title_en: "Welcome!",
                description_vi: "Chào mừng đến với Bitcoin Wallet! Hãy cùng khám phá các tính năng.",
                description_en: "Welcome to Bitcoin Wallet! Let's explore the features together.",
            },
            OnboardingStep {
                title_vi: "Dashboard",
                title_en: "Dashboard",
                description_vi: "Đây là Dashboard - xem tổng số dư và số ví của bạn",
                description_en: "This is the Dashboard - view your total balance and wallet count",
            },
            OnboardingStep {
                title_vi: "Quản lý ví",
                title_en: "Wallet Management",
                description_vi: "Quản lý ví tại đây - tạo, import, backup ví Bitcoin",
                description_en: "Manage wallets here - create, import, and backup Bitcoin wallets",
            },
            OnboardingStep {
                title_vi: "Gửi & Nhận",
                title_en: "Send & Receive",
                description_vi: "Gửi và nhận BTC dễ dàng với địa chỉ và QR code",
                description_en: "Send and receive BTC easily with addresses and QR codes",
            },
            OnboardingStep {
                title_vi: "Bảo mật",
                title_en: "Security",
                description_vi: "Đừng quên backup mnemonic! Đây là chìa khóa khôi phục ví",
                description_en: "Don't forget to backup your mnemonic! It's the key to recover your wallets",
            },
        ]
    }

    pub fn update(&mut self, message: OnboardingMessage) -> Option<OnboardingEvent> {
        match message {
            OnboardingMessage::Next => {
                if self.current_step < self.total_steps - 1 {
                    self.current_step += 1;
                } else {
                    return Some(OnboardingEvent::Finished);
                }
                None
            }
            OnboardingMessage::Previous => {
                if self.current_step > 0 {
                    self.current_step -= 1;
                }
                None
            }
            OnboardingMessage::Skip => Some(OnboardingEvent::Skipped),
            OnboardingMessage::Complete => Some(OnboardingEvent::Finished),
        }
    }

    pub fn view(&self) -> Element<'_, OnboardingMessage> {
        let steps = Self::steps();
        let step = &steps[self.current_step as usize];

        let title = text(t(step.title_vi, step.title_en))
            .size(28)
            .style(text_primary_color());

        let description = text(t(step.description_vi, step.description_en))
            .size(16)
            .style(text_secondary_color());

        // Progress dots
        let mut dots = row![];
        for i in 0..self.total_steps {
            let dot = container(Space::with_width(12).height(12))
                .style(move |_| {
                    let color = if i == self.current_step {
                        Colors::ACCENT_PURPLE
                    } else if i < self.current_step {
                        Colors::ACCENT_TEAL
                    } else {
                        Colors::BORDER
                    };
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(color)),
                        border: iced::border::rounded(6),
                        ..Default::default()
                    }
                });
            dots = dots.push(dot);
            if i < self.total_steps - 1 {
                dots = dots.push(Space::with_width(8));
            }
        }

        let progress_indicator = container(dots)
            .padding(12)
            .style(|theme: &iced::Theme| {
                let colors = get_theme_colors(theme);
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(colors.bg_input)),
                    border: iced::border::rounded(16),
                    ..Default::default()
                }
            });

        // Navigation buttons
        let mut nav_buttons = row![];
        
        if self.current_step > 0 {
            nav_buttons = nav_buttons.push(
                button(text(t("Quay lại", "Back")).size(14))
                    .on_press(OnboardingMessage::Previous)
                    .padding(12)
                    .style(secondary_button_style()),
            );
            nav_buttons = nav_buttons.push(Space::with_width(12));
        } else {
            nav_buttons = nav_buttons.push(
                button(text(t("Bỏ qua", "Skip")).size(14))
                    .on_press(OnboardingMessage::Skip)
                    .padding(12)
                    .style(secondary_button_style()),
            );
            nav_buttons = nav_buttons.push(Space::with_width(12));
        }

        if self.current_step < self.total_steps - 1 {
            nav_buttons = nav_buttons.push(
                button(text(t("Tiếp theo", "Next")).size(14))
                    .on_press(OnboardingMessage::Next)
                    .padding(12)
                    .style(primary_button_style()),
            );
        } else {
            nav_buttons = nav_buttons.push(
                button(text(t("Bắt đầu", "Get Started")).size(14))
                    .on_press(OnboardingMessage::Complete)
                    .padding(12)
                    .style(primary_button_style()),
            );
        }

        let content = column![
            progress_indicator,
            Space::with_height(24),
            title,
            Space::with_height(16),
            description,
            Space::with_height(40),
            nav_buttons.align_y(Alignment::Center),
        ]
        .align_x(Alignment::Center)
        .padding(40)
        .spacing(0);

        container(content)
            .style(card_style())
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}
