use crate::i18n::AppLanguage;
use crate::ui::components::language_selector::LanguageSelector;

#[derive(Debug, Clone)]
pub enum LoginMessage {
    LanguageChanged(AppLanguage),
    NicknameChanged(String),
    PassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    BrowseBackupPath,
    Submit,
    SetMode(LoginMode),
    TogglePassphraseVisibility,
    ToggleConfirmPassphraseVisibility,
}

pub enum LoginEvent {
    ChangeLanguage(AppLanguage),
    BrowseBackupPath,
    SubmitExisting {
        passphrase: String,
    },
    SubmitNew {
        passphrase: String,
        nickname: String,
    },
    SubmitImport {
        backup_path: String,
        passphrase: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginMode {
    ExistingWallet,
    NewWallet,
    ImportBackup,
}

pub struct LoginView {
    pub nickname: String,
    pub passphrase: String,
    pub confirm_passphrase: String,
    pub backup_path: String,
    pub mode: LoginMode,
    pub can_create_new_passphrase: bool,
    pub error: Option<String>,
    pub language_selector: LanguageSelector,
    pub show_passphrase: bool,
    pub show_confirm_passphrase: bool,
}
