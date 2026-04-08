use std::time::{Duration, Instant};

use iced::{
    widget::{column, container, row, text, Space},
    Alignment, Color, Element, Length,
};

use crate::theme::{popup_dialog_style, text_color, text_primary_color, Colors};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};

/// Toast notification types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Info,
}

impl ToastType {
    pub fn icon_char(&self) -> String {
        match self {
            Self::Success => Bootstrap::CheckCircle,
            Self::Info => Bootstrap::InfoCircle,
        }
        .to_string()
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Success => Colors::SUCCESS,
            Self::Info => Colors::ACCENT_BLUE,
        }
    }
}

/// Toast notification message
#[derive(Debug, Clone)]
pub struct Toast {
    pub toast_type: ToastType,
    pub message: String,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(toast_type: ToastType, message: String, duration_secs: u64) -> Self {
        Self {
            toast_type,
            message,
            created_at: Instant::now(),
            duration: Duration::from_secs(duration_secs),
        }
    }

    pub fn success(message: String) -> Self {
        Self::new(ToastType::Success, message, 3)
    }

    pub fn info(message: String) -> Self {
        Self::new(ToastType::Info, message, 3)
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }
}

/// Toast notification manager
pub struct ToastManager {
    toasts: Vec<Toast>,
    max_toasts: usize,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            max_toasts: 3,
        }
    }

    pub fn add_toast(&mut self, toast: Toast) {
        // Remove expired toasts first
        self.cleanup_expired();

        // Add new toast
        self.toasts.push(toast);

        // Keep only max_toasts most recent
        if self.toasts.len() > self.max_toasts {
            self.toasts.drain(..self.toasts.len() - self.max_toasts);
        }
    }

    pub fn cleanup_expired(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }

    pub fn has_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }

    pub fn view(&self) -> Option<Element<'_, ()>> {
        if self.toasts.is_empty() {
            return None;
        }

        let mut toast_elements: Vec<Element<'_, ()>> = Vec::new();

        for toast in &self.toasts {
            let icon = text(toast.toast_type.icon_char()).size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_color(toast.toast_type.color()));

            let message_text = text(&toast.message).size(13)
                .style(text_primary_color());

            let toast_element = container(
                row![
                    icon,
                    Space::with_width(8),
                    message_text,
                ]
                .align_y(Alignment::Center),
            )
            .style(popup_dialog_style())
            .padding(12)
            .width(Length::Fill)
            .into();

            toast_elements.push(toast_element);
            toast_elements.push(Space::with_height(6).into());
        }

        Some(
            container(column(toast_elements))
                .padding(8)
                .width(Length::Fixed(350.0))
                .into(),
        )
    }
}
