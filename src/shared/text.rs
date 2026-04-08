use std::path::PathBuf;

use crate::i18n::t;

/// Format a short transaction ID
pub fn short_txid(txid: &str) -> String {
    let prefix = txid.get(..12).unwrap_or(txid);
    format!("{prefix}...")
}

/// Localized wallet count text
pub fn wallet_count_text(count: usize) -> String {
    format!("{} {}", count, t("ví", "wallet(s)"))
}

/// Localized address count text
pub fn address_count_text(count: usize) -> String {
    format!("{} {}", count, t("địa chỉ mới", "new address(es)"))
}

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

/// Format number with space separators
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

/// Resolve user path (~ → home directory)
pub fn resolve_user_path(raw_path: &str) -> PathBuf {
    let trimmed = raw_path.trim();
    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return std::path::Path::new(&home).join(rest);
        }
    }
    std::path::PathBuf::from(trimmed)
}

/// Trim and validate nickname
pub fn normalize_nickname(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim).filter(|value| !value.is_empty()).map(ToOwned::to_owned)
}

/// Sanitize filename for export
pub fn sanitize_filename(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' { result.push(ch); }
        else if ch.is_whitespace() { result.push('_'); }
    }
    let trimmed = result.trim_matches('_');
    if trimmed.is_empty() { "wallet".to_string() } else { trimmed.to_string() }
}

/// Default mnemonic PDF filename
pub fn default_mnemonic_pdf_filename(wallet_name: &str) -> String {
    format!("{}_mnemonic_backup.pdf", sanitize_filename(wallet_name))
}

/// Default encrypted backup filename
pub fn default_mnemonic_encrypted_filename(wallet_name: &str) -> String {
    format!("{}_mnemonic_backup.enc", sanitize_filename(wallet_name))
}

/// Default SLIP-0039 export directory name
pub fn default_slip39_directory_name(wallet_name: &str, threshold: u8, share_count: u8) -> String {
    format!("{}_slip39_{}of{}", sanitize_filename(wallet_name), threshold, share_count)
}

/// Ensure path has .pdf extension
pub fn ensure_pdf_extension(mut path: PathBuf) -> PathBuf {
    let has_pdf = path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.eq_ignore_ascii_case("pdf")).unwrap_or(false);
    if !has_pdf { path.set_extension("pdf"); }
    path
}

/// Ensure path has .enc extension
pub fn ensure_enc_extension(mut path: PathBuf) -> PathBuf {
    let has_enc = path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.eq_ignore_ascii_case("enc")).unwrap_or(false);
    if !has_enc { path.set_extension("enc"); }
    path
}
