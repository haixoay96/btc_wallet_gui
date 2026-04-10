use crate::core::wallet::{TxRecord, Wallet};
use chrono::{DateTime, Local};

/// Data point for sparkline chart
#[derive(Debug, Clone, Copy)]
pub struct BalancePoint {
    /// Unix timestamp (start of day)
    #[allow(dead_code)]
    pub timestamp: i64,
    /// Balance in satoshis at this point
    pub balance_sat: i64,
}

/// Calculate 7-day balance history across all wallets
/// Shows cumulative net change over the last 7 days
pub fn calculate_7day_balance_history(wallets: &[Wallet]) -> Vec<BalancePoint> {
    let now = Local::now();
    let today_start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap_or_default()
        .and_local_timezone(Local)
        .earliest()
        .unwrap_or_default()
        .timestamp() as u64;

    // Gather all transactions with timestamps
    let mut all_txs: Vec<&TxRecord> = Vec::new();
    for wallet in wallets {
        all_txs.extend(wallet.history.iter().filter(|tx| tx.block_time.is_some()));
    }
    all_txs.sort_by_key(|tx| tx.block_time.unwrap_or(0));

    // Calculate daily net changes for the last 7 days
    let mut daily_changes: Vec<i64> = vec![0; 7];

    for tx in &all_txs {
        let tx_ts = tx.block_time.unwrap_or(0);
        // Skip future transactions
        if tx_ts > today_start {
            continue;
        }
        // Only include transactions from the last 7 days
        let seven_days_ago = today_start.saturating_sub(7 * 86400);
        if tx_ts >= seven_days_ago {
            let days_ago = ((today_start - tx_ts) / 86400) as usize;
            if days_ago < 7 {
                daily_changes[6 - days_ago] += tx.amount_sat;
            }
        }
    }

    // Build cumulative points
    let mut points = Vec::new();
    let mut cumulative = 0i64;

    for day_offset in 0..7 {
        cumulative += daily_changes[day_offset];
        let day_ts = (today_start as i64) - ((6 - day_offset) as i64 * 86400);

        points.push(BalancePoint {
            timestamp: day_ts,
            balance_sat: cumulative,
        });
    }

    tracing::debug!(
        sparkline_points = points.len(),
        daily_changes = ?daily_changes,
        "7-day balance history calculated"
    );

    points
}

/// Format a short date label (e.g., "Mon", "Tue")
#[allow(dead_code)]
pub fn format_day_label(timestamp: i64) -> String {
    let dt = DateTime::from_timestamp(timestamp, 0).unwrap_or_default();
    dt.format("%a").to_string()
}
