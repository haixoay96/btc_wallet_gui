use std::time::{Duration, Instant};

use iced::{
    widget::{column, container, row, text, Space},
    Alignment, Color, Element, Length,
};

use crate::theme::{popup_dialog_style, text_color, Colors};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};

/// Toast notification types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Info,
    Warning,
}

impl ToastType {
    pub fn icon_char(&self) -> String {
        match self {
            Self::Success => Bootstrap::CheckCircle,
            Self::Error => Bootstrap::XCircle,
            Self::Info => Bootstrap::InfoCircle,
            Self::Warning => Bootstrap::ExclamationTriangle,
        }
        .to_string()
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Success => Colors::SUCCESS,
            Self::Error => Colors::ERROR,
            Self::Info => Colors::ACCENT_BLUE,
            Self::Warning => Colors::WARNING,
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

    pub fn error(message: String) -> Self {
        Self::new(ToastType::Error, message, 5)
    }

    pub fn info(message: String) -> Self {
        Self::new(ToastType::Info, message, 3)
    }

    pub fn warning(message: String) -> Self {
        Self::new(ToastType::Warning, message, 4)
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    pub fn remaining_secs(&self) -> u64 {
        let remaining = self.duration.saturating_sub(self.created_at.elapsed());
        remaining.as_secs()
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
            let icon = text(toast.toast_type.icon_char())
                .size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_color(toast.toast_type.color()));

            let message_text = text(&toast.message)
                .size(13)
                .style(text_color(Colors::TEXT_PRIMARY));

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

/// Format BTC amount with space separators
/// Example: 0.00123456 -> "0.001 234 56"
pub fn format_btc_with_spaces(amount_sat: u64) -> String {
    let amount_btc = amount_sat as f64 / 100_000_000.0;
    let formatted = format!("{:.8}", amount_btc);
    
    // Split at decimal point
    let parts: Vec<&str> = formatted.split('.').collect();
    if parts.len() != 2 {
        return formatted;
    }
    
    let integer_part = parts[0];
    let decimal_part = parts[1];
    
    // Group decimal part in groups of 3 from left
    let grouped_decimal: String = decimal_part
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if i > 0 && i % 3 == 0 {
                Some(' ')
            } else {
                None
            }
            .into_iter()
            .chain(std::iter::once(c))
        })
        .collect();
    
    format!("{}.{}", integer_part, grouped_decimal)
}

/// Format any number with space separators (for satoshi amounts)
pub fn format_number_with_spaces(amount: u64, group_size: usize) -> String {
    let s = amount.to_string();
    let len = s.len();
    let first_group = len % group_size;
    
    let mut result = String::new();
    
    if first_group > 0 {
        result.push_str(&s[..first_group]);
        if first_group < len {
            result.push(' ');
        }
    }
    
    let mut i = first_group;
    while i < len {
        let end = (i + group_size).min(len);
        result.push_str(&s[i..end]);
        i = end;
        if i < len {
            result.push(' ');
        }
    }
    
    result
}
