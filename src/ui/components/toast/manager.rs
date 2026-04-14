use super::structure::{Toast, ToastManager, ToastType};

use std::time::{Duration, Instant};

use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, column, container, row, text},
};

use crate::ui::theme::{
    popup_dialog_style, text_accent_blue_color, text_error_color, text_primary_color,
    text_success_color,
};
use iced_fonts::bootstrap::advanced_text;

/// Toast notification types

impl ToastType {
    pub fn icon_char(&self) -> String {
        match self {
            Self::Success => advanced_text::check_circle().0,
            Self::Info => advanced_text::info_circle().0,
            Self::Error => advanced_text::exclamation_circle().0,
        }
    }

    pub fn text_style(&self) -> Box<dyn Fn(&Theme) -> iced::widget::text::Style> {
        match self {
            Self::Success => text_success_color(),
            Self::Info => text_accent_blue_color(),
            Self::Error => text_error_color(),
        }
    }
}

/// Toast notification message

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

    pub fn error(message: String) -> Self {
        Self::new(ToastType::Error, message, 5)
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }
}

/// Toast notification manager

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
            let icon_style = toast.toast_type.text_style();
            let icon = text(toast.toast_type.icon_char())
                .size(16)
                .style(icon_style);

            let message_text = text(&toast.message).size(13).style(text_primary_color());

            let toast_element = container(
                row![icon, Space::new().width(8), message_text,].align_y(Alignment::Center),
            )
            .style(popup_dialog_style())
            .padding(12)
            .width(Length::Fill)
            .into();

            toast_elements.push(toast_element);
            toast_elements.push(Space::new().height(6).into());
        }

        Some(
            container(column(toast_elements))
                .padding(8)
                .width(Length::Fixed(350.0))
                .into(),
        )
    }
}
