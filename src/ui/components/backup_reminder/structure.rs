/// Message for backup reminder interactions
#[derive(Debug, Clone)]
pub enum BackupReminderMessage {
    NavigateToWallets,
    DismissReminder,
}

/// Number of days to wait before showing backup reminder again after dismissal
pub const REMIND_AFTER_DAYS: i64 = 7;

/// Check if backup reminder should be shown
pub fn should_show_backup_reminder(
    wallets_needing_backup: usize,
    last_dismissed: Option<i64>,
) -> bool {
    if wallets_needing_backup == 0 {
        return false;
    }

    match last_dismissed {
        Some(ts) => {
            let dismissed = chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default();
            let now = chrono::Local::now();
            let elapsed = now.signed_duration_since(dismissed);
            // Show reminder again only if 7+ days have passed since dismissal
            elapsed.num_days() >= REMIND_AFTER_DAYS
        }
        None => true,
    }
}

/// Get current timestamp as i64
pub fn current_timestamp() -> i64 {
    chrono::Local::now().timestamp()
}
