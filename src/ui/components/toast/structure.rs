use std::time::{Duration, Instant};
use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Info,
}

pub struct Toast {
    pub toast_type: ToastType,
    pub message: String,
    pub created_at: Instant,
    pub duration: Duration,
}

pub struct ToastManager {
    pub toasts: Vec<Toast>,
    pub max_toasts: usize,
}
