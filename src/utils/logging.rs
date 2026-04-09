use std::sync::OnceLock;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    filter::LevelFilter, fmt, prelude::*, registry, reload, util::SubscriberInitExt,
};

type ReloadHandle = reload::Handle<LevelFilter, registry::Registry>;
static RELOAD_HANDLE: OnceLock<ReloadHandle> = OnceLock::new();

/// Initialize global logging: both console and rolling file.
/// Call once at application startup.
///
/// Log files are stored in `{data_dir}/logs/btc_wallet_gui.log`.
/// Files rotate daily, keeping up to 7 days.
pub fn init(debug_enabled: bool) {
    let level = if debug_enabled {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };
    let (filter, handle) = reload::Layer::new(level);

    // Rolling file appender: daily rotation, keep 7 days
    let log_dir = log_directory();
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("btc_wallet_gui")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&log_dir)
        .expect("Failed to create log file appender");

    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(file_appender)
        .with_target(true)
        .with_thread_ids(false)
        .with_level(true)
        .with_line_number(true);

    let console_layer = fmt::layer()
        .with_ansi(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .compact();

    registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .init();

    let _ = RELOAD_HANDLE.set(handle);

    // Log startup info
    tracing::info!(
        log_dir = %log_dir.display(),
        debug = debug_enabled,
        "Logging initialized"
    );
}

/// Toggle debug logging at runtime without restarting.
/// `true` = DEBUG level, `false` = INFO level.
pub fn set_debug_level(enabled: bool) {
    if let Some(handle) = RELOAD_HANDLE.get() {
        let level = if enabled {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        };
        let _ = handle.modify(|f| *f = level);
        tracing::info!(
            debug = enabled,
            "Log level changed to {}",
            if enabled { "DEBUG" } else { "INFO" }
        );
    }
}

/// Get the log directory path, creating it if needed.
fn log_directory() -> std::path::PathBuf {
    let data_dir = if let Ok(storage) = crate::infra::storage::Storage::new() {
        storage.paths.data_dir.clone()
    } else {
        std::env::current_dir().unwrap_or_default()
    };
    let log_dir = data_dir.join("logs");
    let _ = std::fs::create_dir_all(&log_dir);
    log_dir
}
