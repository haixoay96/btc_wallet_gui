use std::{
    env,
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use printpdf::{BuiltinFont, Mm, PdfDocument};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::i18n::t;
use crate::storage::{decrypt_blob, encrypt_blob, EncryptedEnvelope};

const ENCRYPTED_SECRET_EXPORT_FORMAT: &str = "btc_wallet_gui_encrypted_export";
const ENCRYPTED_SECRET_EXPORT_VERSION: u8 = 1;

// Re-export format functions from components
pub use crate::components::{format_btc_with_spaces, format_number_with_spaces};

#[derive(Debug, PartialEq, Eq)]
pub enum DecryptedSecretExport {
    Mnemonic {
        wallet_name: String,
        network: String,
        mnemonic: String,
    },
    Slip39 {
        wallet_name: String,
        network: String,
        threshold: u8,
        share_count: u8,
        slip39_passphrase: String,
        shares: Vec<String>,
    },
}

// Utility functions

pub fn short_txid(txid: &str) -> String {
    let prefix = txid.get(..12).unwrap_or(txid);
    format!("{prefix}...")
}

pub fn wallet_count_text(count: usize) -> String {
    format!("{} {}", count, t("ví", "wallet(s)"))
}

pub fn address_count_text(count: usize) -> String {
    format!("{} {}", count, t("địa chỉ mới", "new address(es)"))
}

pub fn resolve_user_path(raw_path: &str) -> PathBuf {
    let trimmed = raw_path.trim();
    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return Path::new(&home).join(rest);
        }
    }

    PathBuf::from(trimmed)
}

pub fn pick_import_backup_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t(
            "Chọn file backup để import",
            "Choose backup file to import",
        ))
        .add_filter(t("File backup", "Backup files"), &["enc", "json"])
        .pick_file()
}

pub fn pick_encrypted_secret_import_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t(
            "Chọn file mnemonic backup mã hóa (.enc)",
            "Choose encrypted mnemonic backup file (.enc)",
        ))
        .add_filter(t("File mã hóa", "Encrypted file"), &["enc"])
        .pick_file()
}

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

pub fn pick_mnemonic_pdf_path(default_file_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t("Lưu mnemonic ra PDF", "Save mnemonic as PDF"))
        .add_filter(t("File PDF", "PDF file"), &["pdf"])
        .set_file_name(default_file_name)
        .save_file()
}

pub fn pick_encrypted_export_path(title: &str, default_file_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter(t("File mã hóa", "Encrypted file"), &["enc"])
        .set_file_name(default_file_name)
        .save_file()
}

pub fn pick_slip39_export_directory() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(t(
            "Chọn thư mục chứa backup SLIP-0039",
            "Choose folder for SLIP-0039 backup",
        ))
        .pick_folder()
}

pub fn default_mnemonic_pdf_filename(wallet_name: &str) -> String {
    format!("{}_mnemonic_backup.pdf", sanitize_filename(wallet_name))
}

pub fn default_mnemonic_encrypted_filename(wallet_name: &str) -> String {
    format!("{}_mnemonic_backup.enc", sanitize_filename(wallet_name))
}

pub fn default_slip39_directory_name(wallet_name: &str, threshold: u8, share_count: u8) -> String {
    format!(
        "{}_slip39_{}of{}",
        sanitize_filename(wallet_name),
        threshold,
        share_count
    )
}

pub fn sanitize_filename(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            result.push(ch);
        } else if ch.is_whitespace() {
            result.push('_');
        }
    }

    let trimmed = result.trim_matches('_');
    if trimmed.is_empty() {
        "wallet".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn ensure_pdf_extension(mut path: PathBuf) -> PathBuf {
    let has_pdf = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false);

    if !has_pdf {
        path.set_extension("pdf");
    }

    path
}

pub fn ensure_enc_extension(mut path: PathBuf) -> PathBuf {
    let has_enc = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("enc"))
        .unwrap_or(false);

    if !has_enc {
        path.set_extension("enc");
    }

    path
}

pub fn normalize_nickname(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub struct Slip39PdfExport<'a> {
    pub wallet_name: &'a str,
    pub network: &'a str,
    pub threshold: u8,
    pub share_count: u8,
    pub has_slip39_passphrase: bool,
}

#[derive(Serialize)]
struct EncryptedSecretExport {
    format: &'static str,
    version: u8,
    kind: &'static str,
    envelope: EncryptedEnvelope,
}

#[derive(Deserialize)]
struct StoredEncryptedSecretExport {
    format: String,
    version: u8,
    kind: String,
    envelope: EncryptedEnvelope,
}

#[derive(Serialize)]
struct MnemonicEncryptedPayload<'a> {
    wallet_name: &'a str,
    network: &'a str,
    mnemonic: &'a str,
}

#[derive(Deserialize)]
struct StoredMnemonicEncryptedPayload {
    wallet_name: String,
    network: String,
    mnemonic: String,
}

#[derive(Deserialize)]
struct StoredSlip39EncryptedPayload {
    wallet_name: String,
    network: String,
    threshold: u8,
    share_count: u8,
    slip39_passphrase: String,
    shares: Vec<String>,
}

// PDF export functions

pub fn export_mnemonic_to_pdf(
    path: &Path,
    wallet_name: &str,
    network: &str,
    mnemonic: &str,
) -> Result<(), String> {
    let (doc, page, layer) =
        PdfDocument::new("Mnemonic Backup", Mm(210.0), Mm(297.0), "Mnemonic Layer");
    let current_layer = doc.get_page(page).get_layer(layer);

    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|err| {
            format!(
                "{}: {err}",
                t("Không tải được font PDF", "Could not load PDF font")
            )
        })?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|err| {
            format!(
                "{}: {err}",
                t("Không tải được font PDF", "Could not load PDF font")
            )
        })?;

    current_layer.use_text(
        "Bitcoin Wallet - Mnemonic Backup",
        18.0,
        Mm(18.0),
        Mm(280.0),
        &font_bold,
    );
    current_layer.use_text(
        format!("Wallet: {wallet_name}"),
        12.0,
        Mm(18.0),
        Mm(268.0),
        &font_regular,
    );
    current_layer.use_text(
        format!("Network: {network}"),
        12.0,
        Mm(18.0),
        Mm(260.0),
        &font_regular,
    );
    current_layer.use_text(
        "Keep this file offline and private. Anyone with these words can spend your funds.",
        10.0,
        Mm(18.0),
        Mm(250.0),
        &font_regular,
    );

    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    for (idx, word) in words.iter().enumerate() {
        let row = idx / 2;
        let col = idx % 2;
        let x = if col == 0 { 18.0 } else { 110.0 };
        let y = 236.0 - (row as f32 * 10.0);

        current_layer.use_text(
            format!("{:02}. {}", idx + 1, word),
            12.0,
            Mm(x),
            Mm(y),
            &font_regular,
        );
    }

    let file = File::create(path).map_err(|err| {
        format!(
            "{} {}: {err}",
            t("Không tạo được file PDF", "Could not create PDF file"),
            path.display()
        )
    })?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer).map_err(|err| {
        format!(
            "{}: {err}",
            t("Không ghi được nội dung PDF", "Could not write PDF content")
        )
    })?;

    Ok(())
}

pub fn export_mnemonic_to_encrypted_file(
    path: &Path,
    wallet_name: &str,
    network: &str,
    mnemonic: &str,
    encryption_passphrase: &str,
) -> Result<(), String> {
    let payload = MnemonicEncryptedPayload {
        wallet_name,
        network,
        mnemonic,
    };

    write_encrypted_export(path, "mnemonic_backup", &payload, encryption_passphrase)
}

pub fn export_slip39_shares_to_pdf_directory(
    base_directory: &Path,
    directory_name: &str,
    export: &Slip39PdfExport<'_>,
    shares: &[String],
) -> Result<PathBuf, String> {
    if shares.is_empty() {
        return Err(t(
            "Không có SLIP-0039 share nào để export",
            "No SLIP-0039 shares available to export",
        )
        .to_string());
    }

    let export_dir = create_unique_export_directory(base_directory, directory_name)?;

    for (index, share) in shares.iter().enumerate() {
        let file_name = format!("share_{:02}_of_{:02}.pdf", index + 1, shares.len());
        let share_path = export_dir.join(file_name);

        export_slip39_share_to_pdf(&share_path, export, index + 1, shares.len(), share)?;
    }

    Ok(export_dir)
}

pub fn load_encrypted_secret_export(
    path: &Path,
    decryption_passphrase: &str,
) -> Result<DecryptedSecretExport, String> {
    let encoded = fs::read(path).map_err(|err| {
        format!(
            "{} {}: {err}",
            t(
                "Không đọc được file backup mã hóa",
                "Could not read encrypted backup file",
            ),
            path.display()
        )
    })?;

    decode_encrypted_secret_export(&encoded, decryption_passphrase)
}

fn create_unique_export_directory(
    base_directory: &Path,
    directory_name: &str,
) -> Result<PathBuf, String> {
    if !base_directory.exists() {
        return Err(format!(
            "{}: {}",
            t(
                "Thư mục đích không tồn tại",
                "Destination directory does not exist"
            ),
            base_directory.display()
        ));
    }

    for attempt in 0..1000 {
        let candidate_name = if attempt == 0 {
            directory_name.to_string()
        } else {
            format!("{directory_name}_{attempt}")
        };
        let candidate = base_directory.join(candidate_name);

        if !candidate.exists() {
            fs::create_dir_all(&candidate).map_err(|err| {
                format!(
                    "{} {}: {err}",
                    t(
                        "Không thể tạo thư mục export SLIP-0039",
                        "Could not create SLIP-0039 export directory",
                    ),
                    candidate.display()
                )
            })?;
            return Ok(candidate);
        }
    }

    Err(t(
        "Không thể tạo thư mục export SLIP-0039 (đã thử quá nhiều lần)",
        "Could not create SLIP-0039 export directory (too many attempts)",
    )
    .to_string())
}

fn export_slip39_share_to_pdf(
    path: &Path,
    export: &Slip39PdfExport<'_>,
    share_index: usize,
    share_total: usize,
    share_phrase: &str,
) -> Result<(), String> {
    let (doc, page, layer) = PdfDocument::new(
        "SLIP-0039 Share Backup",
        Mm(210.0),
        Mm(297.0),
        "Share Layer",
    );
    let current_layer = doc.get_page(page).get_layer(layer);

    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|err| {
            format!(
                "{}: {err}",
                t("Không tải được font PDF", "Could not load PDF font")
            )
        })?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|err| {
            format!(
                "{}: {err}",
                t("Không tải được font PDF", "Could not load PDF font")
            )
        })?;

    current_layer.use_text(
        "Bitcoin Wallet - SLIP-0039 Share",
        18.0,
        Mm(18.0),
        Mm(280.0),
        &font_bold,
    );
    current_layer.use_text(
        format!("Wallet: {}", export.wallet_name),
        12.0,
        Mm(18.0),
        Mm(268.0),
        &font_regular,
    );
    current_layer.use_text(
        format!("Network: {}", export.network),
        12.0,
        Mm(18.0),
        Mm(260.0),
        &font_regular,
    );
    current_layer.use_text(
        format!("Scheme: {}-of-{}", export.threshold, export.share_count),
        12.0,
        Mm(18.0),
        Mm(252.0),
        &font_regular,
    );
    current_layer.use_text(
        format!("Share: {share_index}/{share_total}"),
        12.0,
        Mm(18.0),
        Mm(244.0),
        &font_regular,
    );
    current_layer.use_text(
        format!(
            "SLIP39 passphrase: {}",
            if export.has_slip39_passphrase {
                "SET (required for restore)"
            } else {
                "EMPTY"
            }
        ),
        11.0,
        Mm(18.0),
        Mm(236.0),
        &font_regular,
    );
    current_layer.use_text(
        "Keep this PDF offline. Whoever has enough shares can recover your wallet.",
        10.0,
        Mm(18.0),
        Mm(228.0),
        &font_regular,
    );

    let words: Vec<&str> = share_phrase.split_whitespace().collect();
    for (idx, word) in words.iter().enumerate() {
        let row = idx / 2;
        let col = idx % 2;
        let x = if col == 0 { 18.0 } else { 110.0 };
        let y = 214.0 - (row as f32 * 10.0);

        current_layer.use_text(
            format!("{:02}. {}", idx + 1, word),
            12.0,
            Mm(x),
            Mm(y),
            &font_regular,
        );
    }

    let file = File::create(path).map_err(|err| {
        format!(
            "{} {}: {err}",
            t("Không tạo được file PDF", "Could not create PDF file"),
            path.display()
        )
    })?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer).map_err(|err| {
        format!(
            "{}: {err}",
            t("Không ghi được nội dung PDF", "Could not write PDF content")
        )
    })?;

    Ok(())
}

fn write_encrypted_export<T: Serialize>(
    path: &Path,
    kind: &'static str,
    payload: &T,
    encryption_passphrase: &str,
) -> Result<(), String> {
    let mut plaintext = serde_json::to_vec(payload).map_err(|err| {
        format!(
            "{}: {err}",
            t(
                "Không serialize được dữ liệu secret",
                "Could not serialize secret data"
            )
        )
    })?;

    let envelope = encrypt_blob(&plaintext, encryption_passphrase).map_err(|err| err.to_string());
    plaintext.zeroize();
    let envelope = envelope?;

    let export = EncryptedSecretExport {
        format: ENCRYPTED_SECRET_EXPORT_FORMAT,
        version: ENCRYPTED_SECRET_EXPORT_VERSION,
        kind,
        envelope,
    };

    let encoded = serde_json::to_vec_pretty(&export).map_err(|err| {
        format!(
            "{}: {err}",
            t(
                "Không serialize được file export mã hóa",
                "Could not serialize encrypted export file",
            )
        )
    })?;

    let parent = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|err| {
        format!(
            "{} {}: {err}",
            t(
                "Không tạo được thư mục export",
                "Could not create export directory"
            ),
            parent.display()
        )
    })?;

    let tmp_path = path.with_extension("enc.tmp");
    fs::write(&tmp_path, encoded).map_err(|err| {
        format!(
            "{} {}: {err}",
            t(
                "Không ghi được file export tạm",
                "Could not write temporary export file"
            ),
            tmp_path.display()
        )
    })?;

    fs::rename(&tmp_path, path).map_err(|err| {
        format!(
            "{} {}: {err}",
            t(
                "Không thể hoàn tất file export",
                "Could not finalize export file"
            ),
            path.display()
        )
    })?;

    Ok(())
}

fn decode_encrypted_secret_export(
    encoded: &[u8],
    decryption_passphrase: &str,
) -> Result<DecryptedSecretExport, String> {
    let export: StoredEncryptedSecretExport = serde_json::from_slice(encoded).map_err(|err| {
        format!(
            "{}: {err}",
            t(
                "File backup mã hóa không đúng định dạng JSON",
                "Encrypted backup file is not valid JSON",
            )
        )
    })?;

    if export.format != ENCRYPTED_SECRET_EXPORT_FORMAT {
        return Err(t(
            "Định dạng file backup mã hóa không được hỗ trợ",
            "Unsupported encrypted backup file format",
        )
        .to_string());
    }

    if export.version != ENCRYPTED_SECRET_EXPORT_VERSION {
        return Err(format!(
            "{}: {}",
            t(
                "Version file backup mã hóa không được hỗ trợ",
                "Unsupported encrypted backup file version",
            ),
            export.version
        ));
    }

    let mut plaintext =
        decrypt_blob(&export.envelope, decryption_passphrase).map_err(|err| err.to_string())?;

    let decoded = match export.kind.as_str() {
        "mnemonic_backup" => {
            let payload: StoredMnemonicEncryptedPayload = serde_json::from_slice(&plaintext)
                .map_err(|err| {
                    format!(
                        "{}: {err}",
                        t(
                            "Payload mnemonic mã hóa không hợp lệ",
                            "Encrypted mnemonic payload is invalid",
                        )
                    )
                })?;

            Ok(DecryptedSecretExport::Mnemonic {
                wallet_name: payload.wallet_name,
                network: payload.network,
                mnemonic: payload.mnemonic,
            })
        }
        "slip39_backup" => {
            let payload: StoredSlip39EncryptedPayload = serde_json::from_slice(&plaintext)
                .map_err(|err| {
                    format!(
                        "{}: {err}",
                        t(
                            "Payload SLIP-0039 mã hóa không hợp lệ",
                            "Encrypted SLIP-0039 payload is invalid",
                        )
                    )
                })?;

            if payload.shares.is_empty() {
                plaintext.zeroize();
                return Err(t(
                    "Backup SLIP-0039 mã hóa không chứa share nào",
                    "Encrypted SLIP-0039 backup does not contain any shares",
                )
                .to_string());
            }

            if usize::from(payload.share_count) != payload.shares.len() {
                plaintext.zeroize();
                return Err(t(
                    "Số lượng share trong backup SLIP-0039 không khớp metadata",
                    "SLIP-0039 share count does not match backup metadata",
                )
                .to_string());
            }

            Ok(DecryptedSecretExport::Slip39 {
                wallet_name: payload.wallet_name,
                network: payload.network,
                threshold: payload.threshold,
                share_count: payload.share_count,
                slip39_passphrase: payload.slip39_passphrase,
                shares: payload.shares,
            })
        }
        _ => Err(format!(
            "{}: {}",
            t(
                "Loại backup mã hóa không được hỗ trợ",
                "Unsupported encrypted backup kind",
            ),
            export.kind
        )),
    };

    plaintext.zeroize();
    decoded
}

impl Drop for DecryptedSecretExport {
    fn drop(&mut self) {
        match self {
            Self::Mnemonic { mnemonic, .. } => mnemonic.zeroize(),
            Self::Slip39 {
                slip39_passphrase,
                shares,
                ..
            } => {
                slip39_passphrase.zeroize();
                shares.iter_mut().for_each(Zeroize::zeroize);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        export_mnemonic_to_encrypted_file, load_encrypted_secret_export, DecryptedSecretExport,
    };

    fn unique_temp_path(file_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "btc_wallet_gui_test_{}_{}_{}",
            std::process::id(),
            timestamp,
            file_name
        ))
    }

    #[test]
    fn encrypted_mnemonic_export_round_trips() {
        let path = unique_temp_path("mnemonic.enc");
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        export_mnemonic_to_encrypted_file(
            &path,
            "Primary",
            "testnet",
            mnemonic,
            "correct horse battery staple",
        )
        .expect("encrypted mnemonic export should succeed");

        let decoded = load_encrypted_secret_export(&path, "correct horse battery staple")
            .expect("encrypted mnemonic import should succeed");

        assert_eq!(
            decoded,
            DecryptedSecretExport::Mnemonic {
                wallet_name: "Primary".to_string(),
                network: "testnet".to_string(),
                mnemonic: mnemonic.to_string(),
            }
        );

        fs::remove_file(path).expect("temporary export file should be removable");
    }
}
