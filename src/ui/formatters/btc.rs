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
