use std::{
    env,
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use printpdf::{BuiltinFont, Mm, PdfDocument};

use crate::i18n::t;

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

pub fn normalize_nickname(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
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

pub fn export_slip39_shares_to_pdf_directory(
    base_directory: &Path,
    directory_name: &str,
    wallet_name: &str,
    network: &str,
    threshold: u8,
    share_count: u8,
    has_slip39_passphrase: bool,
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

        export_slip39_share_to_pdf(
            &share_path,
            wallet_name,
            network,
            threshold,
            share_count,
            has_slip39_passphrase,
            index + 1,
            shares.len(),
            share,
        )?;
    }

    Ok(export_dir)
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
    wallet_name: &str,
    network: &str,
    threshold: u8,
    share_count: u8,
    has_slip39_passphrase: bool,
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
        format!("Scheme: {threshold}-of-{share_count}"),
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
            if has_slip39_passphrase {
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