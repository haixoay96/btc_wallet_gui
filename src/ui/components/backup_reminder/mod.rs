mod screen;
pub mod structure;

pub use screen::backup_reminder_banner;
pub use structure::{current_timestamp, should_show_backup_reminder, BackupReminderMessage};
