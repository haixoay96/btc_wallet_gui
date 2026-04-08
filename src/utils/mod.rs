mod dialogs;
mod filenames;

// Re-export dialogs
pub use dialogs::{
    pick_import_backup_path, pick_encrypted_secret_import_path,
    pick_export_backup_path, pick_mnemonic_pdf_path, pick_encrypted_export_path,
    pick_slip39_export_directory,
};

// Re-export format functions from ui/formatters
pub use crate::ui::formatters::{format_btc_with_spaces, format_number_with_spaces};

// Re-export shared text utilities
pub use crate::shared::text::{
    short_txid, wallet_count_text, address_count_text,
    resolve_user_path, normalize_nickname,
    sanitize_filename, default_mnemonic_pdf_filename, default_mnemonic_encrypted_filename,
    default_slip39_directory_name, ensure_pdf_extension, ensure_enc_extension,
};

// Re-export export types and functions from infra/export
pub use crate::infra::export::{
    DecryptedSecretExport, Slip39PdfExport,
    export_mnemonic_to_pdf, export_slip39_shares_to_pdf_directory,
};

// Re-export write/decode as-is
pub use crate::infra::export::{
    write_encrypted_export, decode_encrypted_secret_export,
};

// Wrapper: load_encrypted_secret_export - reads file then decrypts
pub fn load_encrypted_secret_export(
    path: &std::path::Path,
    passphrase: &str,
) -> Result<DecryptedSecretExport, String> {
    let encoded = std::fs::read(path).map_err(|err| {
        format!("{} {}: {err}", crate::i18n::t("Không đọc được file backup mã hóa", "Could not read encrypted backup file"), path.display())
    })?;
    decode_encrypted_secret_export(&encoded, passphrase)
}

// Wrapper: export_mnemonic_to_encrypted_file - writes mnemonic to encrypted file
pub fn export_mnemonic_to_encrypted_file(
    path: &std::path::Path,
    wallet_name: &str,
    network: &str,
    mnemonic: &str,
    encryption_passphrase: &str,
) -> Result<(), String> {
    #[derive(serde::Serialize)]
    struct Payload<'a> { wallet_name: &'a str, network: &'a str, mnemonic: &'a str }
    write_encrypted_export(path, "mnemonic_backup", &Payload { wallet_name, network, mnemonic }, encryption_passphrase)
}
