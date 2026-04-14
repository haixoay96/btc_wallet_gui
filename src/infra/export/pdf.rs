use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use printpdf::{
    BuiltinFont, Color, Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, Point, Pt, Rgb, TextItem,
};

use super::structure::Slip39PdfExport;

/// Helper: mm to points
fn mm_to_pt(mm: f32) -> Pt {
    Pt(mm / 25.4 * 72.0)
}

/// Helper: create text ops with automatic position reset.
/// Each text block is wrapped in BT/ET (text object) which resets text position,
/// and q/Q (graphics state) to ensure clean state. Without BT/ET, text operators
/// don't work properly in PDF.
fn text_at(x_mm: f32, y_mm: f32, size: f32, font: BuiltinFont, text: &str) -> Vec<Op> {
    vec![
        Op::SaveGraphicsState,
        Op::StartTextSection,
        Op::SetTextCursor {
            pos: Point {
                x: mm_to_pt(x_mm),
                y: mm_to_pt(y_mm),
            },
        },
        Op::SetFontSizeBuiltinFont {
            size: Pt(size),
            font,
        },
        Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(text.to_string())],
            font,
        },
        Op::EndTextSection,
        Op::RestoreGraphicsState,
    ]
}

pub fn export_mnemonic_to_pdf(
    path: &Path,
    wallet_name: &str,
    network: &str,
    mnemonic: &str,
) -> Result<(), String> {
    use crate::ui::i18n::t;

    let mut doc = PdfDocument::new("Mnemonic Backup");
    let mut ops: Vec<Op> = Vec::new();

    // Set fill color to black for text visibility
    ops.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });

    // Title
    ops.extend(text_at(
        18.0,
        280.0,
        18.0,
        BuiltinFont::HelveticaBold,
        "Bitcoin Wallet - Mnemonic Backup",
    ));

    // Wallet name
    ops.extend(text_at(
        18.0,
        268.0,
        12.0,
        BuiltinFont::Helvetica,
        &format!("Wallet: {wallet_name}"),
    ));

    // Network
    ops.extend(text_at(
        18.0,
        260.0,
        12.0,
        BuiltinFont::Helvetica,
        &format!("Network: {network}"),
    ));

    // Warning
    ops.extend(text_at(
        18.0,
        250.0,
        10.0,
        BuiltinFont::Helvetica,
        "Keep this file offline and private. Anyone with these words can spend your funds.",
    ));

    // Mnemonic words - two columns
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    for (idx, word) in words.iter().enumerate() {
        let row = idx / 2;
        let col = idx % 2;
        let x = if col == 0 { 18.0 } else { 110.0 };
        let y = 236.0 - (row as f32 * 10.0);
        ops.extend(text_at(
            x,
            y,
            12.0,
            BuiltinFont::Helvetica,
            &format!("{:02}. {}", idx + 1, word),
        ));
    }

    let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);
    doc.pages.push(page);

    let file = File::create(path).map_err(|err| {
        format!(
            "{} {}: {err}",
            t("Không tạo được file PDF", "Could not create PDF file"),
            path.display()
        )
    })?;
    let mut writer = BufWriter::new(file);
    let mut warnings = Vec::new();
    doc.save_writer(&mut writer, &PdfSaveOptions::default(), &mut warnings);

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

    let mut doc = PdfDocument::new("SLIP-0039 Share Backup");
    let mut ops: Vec<Op> = Vec::new();

    ops.push(Op::SetFillColor {
        col: Color::Rgb(Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            icc_profile: None,
        }),
    });

    // Title
    ops.extend(text_at(
        18.0,
        280.0,
        18.0,
        BuiltinFont::HelveticaBold,
        "Bitcoin Wallet - SLIP-0039 Share",
    ));

    // Wallet name
    ops.extend(text_at(
        18.0,
        268.0,
        12.0,
        BuiltinFont::Helvetica,
        &format!("Wallet: {}", export.wallet_name),
    ));

    // Network
    ops.extend(text_at(
        18.0,
        260.0,
        12.0,
        BuiltinFont::Helvetica,
        &format!("Network: {}", export.network),
    ));

    // Scheme
    ops.extend(text_at(
        18.0,
        252.0,
        12.0,
        BuiltinFont::Helvetica,
        &format!("Scheme: {}-of-{}", export.threshold, export.share_count),
    ));

    // Share number
    ops.extend(text_at(
        18.0,
        244.0,
        12.0,
        BuiltinFont::Helvetica,
        &format!("Share: {share_index}/{share_total}"),
    ));

    // SLIP39 passphrase status
    ops.extend(text_at(
        18.0,
        236.0,
        11.0,
        BuiltinFont::Helvetica,
        &format!(
            "SLIP39 passphrase: {}",
            if export.has_slip39_passphrase {
                "SET (required for restore)"
            } else {
                "EMPTY"
            }
        ),
    ));

    // Warning
    ops.extend(text_at(
        18.0,
        228.0,
        10.0,
        BuiltinFont::Helvetica,
        "Keep this PDF offline. Whoever has enough shares can recover your wallet.",
    ));

    // Share words - two columns
    let words: Vec<&str> = share_phrase.split_whitespace().collect();
    for (idx, word) in words.iter().enumerate() {
        let row = idx / 2;
        let col = idx % 2;
        let x = if col == 0 { 18.0 } else { 110.0 };
        let y = 214.0 - (row as f32 * 10.0);
        ops.extend(text_at(
            x,
            y,
            12.0,
            BuiltinFont::Helvetica,
            &format!("{:02}. {}", idx + 1, word),
        ));
    }

    let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);
    doc.pages.push(page);

    let file = File::create(path).map_err(|err| {
        format!(
            "{} {}: {err}",
            t("Không tạo được file PDF", "Could not create PDF file"),
            path.display()
        )
    })?;
    let mut writer = BufWriter::new(file);
    let mut warnings = Vec::new();
    doc.save_writer(&mut writer, &PdfSaveOptions::default(), &mut warnings);

    Ok(())
}
