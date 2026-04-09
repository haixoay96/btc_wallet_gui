/// Format number with space separators
/// Format number with spaces
#[allow(dead_code)]
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

/// Format a short transaction ID
/// Get short transaction ID
#[allow(dead_code)]
pub fn short_txid(txid: &str) -> String {
    let prefix = txid.get(..12).unwrap_or(txid);
    format!("{prefix}...")
}
