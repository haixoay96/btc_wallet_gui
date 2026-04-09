use std::path::PathBuf;

use crate::shared::text::resolve_user_path;
use crate::ui::i18n::t;

/// Pick file dialog for importing backup
pub fn pick_import_backup_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t(
            "Chọn file backup để import",
            "Choose backup file to import",
        ))
        .add_filter(t("File backup", "Backup files"), &["enc", "json"])
        .pick_file()
}

/// Pick file dialog for encrypted mnemonic
pub fn pick_encrypted_secret_import_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t(
            "Chọn file mnemonic backup mã hóa (.enc)",
            "Choose encrypted mnemonic backup file (.enc)",
        ))
        .add_filter(t("File mã hóa", "Encrypted file"), &["enc"])
        .pick_file()
}

/// Pick save dialog for encrypted backup
pub fn pick_export_backup_path(current_path: &str) -> Option<PathBuf> {
    let resolved = resolve_user_path(current_path);
    let mut dialog = rfd::FileDialog::new()
        .set_title(t("Chọn nơi lưu backup", "Choose where to save backup"))
        .add_filter(t("Backup mã hóa", "Encrypted backup"), &["enc"]);
    if let Some(parent) = resolved.parent() {
        dialog = dialog.set_directory(parent);
    }
    if let Some(file_name) = resolved.file_name().and_then(|name| name.to_str()) {
        dialog = dialog.set_file_name(file_name);
    } else {
        dialog = dialog.set_file_name("wallet_backup.enc");
    }
    dialog.save_file()
}

/// Pick save dialog for mnemonic PDF
pub fn pick_mnemonic_pdf_path(default_file_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t("Lưu mnemonic ra PDF", "Save mnemonic as PDF"))
        .add_filter(t("File PDF", "PDF file"), &["pdf"])
        .set_file_name(default_file_name)
        .save_file()
}

/// Pick save dialog for encrypted export
pub fn pick_encrypted_export_path(title: &str, default_file_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter(t("File mã hóa", "Encrypted file"), &["enc"])
        .set_file_name(default_file_name)
        .save_file()
}

/// Pick folder dialog for SLIP-0039 export
pub fn pick_slip39_export_directory() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t(
            "Chọn thư mục chứa backup SLIP-0039",
            "Choose folder for SLIP-0039 backup",
        ))
        .pick_folder()
}
