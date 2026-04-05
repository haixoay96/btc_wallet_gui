mod toast;
mod strength;
mod unit;
mod skeleton;
mod tooltip;
mod keyboard;

// Re-export all component types
pub use toast::{Toast, ToastManager, ToastType};
pub use strength::{PassphraseStrength, calculate_strength, strength_bar, requirements_checklist};
pub use unit::{BtcUnit, format_amount_with_unit};
pub use skeleton::{skeleton, skeleton_list, skeleton_wallet_cards, skeleton_transactions, SkeletonType};
pub use tooltip::{help_tooltip, help_icon, info_box, warning_box};
pub use keyboard::{KeyboardShortcut, shortcuts_help_popup};

/// Format BTC amount with space separators
pub fn format_btc_with_spaces(amount_sat: u64) -> String {
    let amount_btc = amount_sat as f64 / 100_000_000.0;
    let formatted = format!("{:.8}", amount_btc);
    let parts: Vec<&str> = formatted.split('.').collect();
    if parts.len() != 2 { return formatted; }
    let integer_part = parts[0];
    let decimal_part = parts[1];
    let grouped_decimal: String = decimal_part.chars().enumerate()
        .flat_map(|(i, c)| {
            if i > 0 && i % 3 == 0 { Some(' ') } else { None }
                .into_iter().chain(std::iter::once(c))
        }).collect();
    format!("{}.{}", integer_part, grouped_decimal)
}

pub fn format_number_with_spaces(amount: u64, group_size: usize) -> String {
    let s = amount.to_string();
    let len = s.len();
    let first_group = len % group_size;
    let mut result = String::new();
    if first_group > 0 {
        result.push_str(&s[..first_group]);
        if first_group < len { result.push(' '); }
    }
    let mut i = first_group;
    while i < len {
        let end = (i + group_size).min(len);
        result.push_str(&s[i..end]);
        i = end;
        if i < len { result.push(' '); }
    }
    result
}
