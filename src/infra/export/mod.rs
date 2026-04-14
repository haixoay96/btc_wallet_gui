mod encrypted_file;
mod pdf;
mod structure;

pub use encrypted_file::{decode_encrypted_secret_export, write_encrypted_export};
pub use pdf::{export_mnemonic_to_pdf, export_slip39_shares_to_pdf_directory};
pub use structure::{DecryptedSecretExport, Slip39PdfExport};

#[cfg(test)]
mod pdf_tests {
    use printpdf::{
        BuiltinFont, Color, Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, PdfWarnMsg, Point, Pt,
        Rgb, TextItem,
    };
    use std::fs::File;
    use std::io::BufWriter;

    fn mm_to_pt(mm: f32) -> Pt {
        Pt(mm / 25.4 * 72.0)
    }

    #[test]
    fn test_basic_pdf_export_text_visible() {
        let mut doc = PdfDocument::new("Test");

        let mut ops: Vec<Op> = Vec::new();
        ops.push(Op::StartTextSection);
        ops.push(Op::SetFillColor {
            col: Color::Rgb(Rgb {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                icc_profile: None,
            }),
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: mm_to_pt(50.0),
                y: mm_to_pt(250.0),
            },
        });
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(24.0),
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text("Hello World".to_string())],
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::EndTextSection);

        let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);
        doc.pages.push(page);

        let file = File::create("/tmp/test_pdf_export.pdf").unwrap();
        let mut writer = BufWriter::new(file);
        let mut warnings: Vec<PdfWarnMsg> = Vec::new();
        doc.save_writer(&mut writer, &PdfSaveOptions::default(), &mut warnings);

        if !warnings.is_empty() {
            for w in &warnings {
                eprintln!("PDF Warning/Error: {:?}", w);
            }
        }

        assert!(warnings.is_empty());
        assert_eq!(doc.pages.len(), 1);
    }

    #[test]
    fn test_full_mnemonic_export() {
        use super::pdf::export_mnemonic_to_pdf;

        export_mnemonic_to_pdf(
            std::path::Path::new("/tmp/test_mnemonic_export.pdf"),
            "Test Wallet",
            "Bitcoin",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .unwrap();

        // Check file exists and is non-empty
        let meta = std::fs::metadata("/tmp/test_mnemonic_export.pdf").unwrap();
        assert!(
            meta.len() > 1000,
            "PDF should be at least 1KB, got {}",
            meta.len()
        );

        // Dump PDF content for inspection
        let content = std::fs::read_to_string("/tmp/test_mnemonic_export.pdf").unwrap_or_default();
        eprintln!("=== PDF CONTENT ===");
        eprintln!("{}", content);
        eprintln!("=== END PDF CONTENT ===");
    }
}
