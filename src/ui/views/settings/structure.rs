use crate::infra::storage::AppTheme;

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    ToggleChangePassphrase,
    CurrentPassphraseChanged(String),
    NewPassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    SubmitPassphraseChange,
    ExportWallet,
    ToggleAbout,
    ToggleClearDataConfirm,
    ClearDataPassphraseChanged(String),
    ConfirmClearData,
    CancelClearData,
    ThemeSelected(AppTheme),
    ShowOnboardingTour,
    FontScaleChanged(f64),
    HighContrastToggled(bool),
    /// Esplora endpoint changed
    #[allow(dead_code)]
    EsploraEndpointChanged(String),
    TimeoutSecsChanged(u64),
    TestConnection,
    DebugLoggingToggled(bool),
    AutoRefreshToggled(bool),
    ShowSatoshisToggled(bool),
    CompactModeToggled(bool),
    ResetAllSettings,
    TestConnectionSuccess(String),
    TestConnectionFailed(String),
}

pub enum SettingsEvent {
    ChangePassphrase {
        current: String,
        new_passphrase: String,
    },
    ExportWallet,
    ClearAllData(String),
    ThemeChanged(AppTheme),
    ShowOnboardingTour,
    FontScaleChanged(f64),
    HighContrastToggled(bool),
    /// Esplora endpoint changed
    #[allow(dead_code)]
    EsploraEndpointChanged(String),
    TimeoutSecsChanged(u64),
    TestConnection,
    DebugLoggingToggled(bool),
    AutoRefreshToggled(bool),
    ShowSatoshisToggled(bool),
    CompactModeToggled(bool),
    ResetAllSettings,
}

pub struct SettingsView {
    pub show_change_passphrase: bool,
    pub current_passphrase: String,
    pub new_passphrase: String,
    pub confirm_passphrase: String,
    pub show_about: bool,
    pub show_clear_data_confirm: bool,
    pub clear_data_passphrase: String,
    pub error: Option<String>,
    pub success: Option<String>,
    pub font_scale: f64,
    pub high_contrast: bool,
    pub esplora_endpoint: String,
    pub timeout_secs: u64,
    pub testing_connection: bool,
    pub debug_logging: bool,
    pub auto_refresh: bool,
    pub show_satoshis: bool,
    pub compact_mode: bool,
    pub data_folder_path: String,
    pub data_folder_size: String,
    pub connection_test_result: Option<String>,
}
