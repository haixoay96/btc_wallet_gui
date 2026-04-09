use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use printpdf::{BuiltinFont, Mm, PdfDocument};

use super::structure::Slip39PdfExport;

// ─── PDF Export ──────────────────────────────────────────────────────────

pub fn export_mnemonic_to_pdf(
    path: &Path,
    wallet_name: &str,
    network: &str,
    mnemonic: &str,
) -> Result<(), String> {
    use crate::ui::i18n::t;

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
    export: &Slip39PdfExport<'_>,
    shares: &[String],
) -> Result<PathBuf, String> {
    use crate::ui::i18n::t;

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

// ─── Private helpers ─────────────────────────────────────────────────────

fn create_unique_export_directory(
    base_directory: &Path,
    directory_name: &str,
) -> Result<PathBuf, String> {
    use crate::ui::i18n::t;

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
            std::fs::create_dir_all(&candidate).map_err(|err| {
                format!(
                    "{} {}: {err}",
                    t(
                        "Không thể tạo thư mục export SLIP-0039",
                        "Could not create SLIP-0039 export directory"
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
    use crate::ui::i18n::t;

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
