use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::io::BufWriter;

use printpdf::{BuiltinFont, Mm, PdfDocument};
use chrono::DateTime;

use crate::wallet::{TxRecord, TxDirection, Wallet, WalletNetwork};
use crate::i18n::t;

/// Export transaction history to CSV
pub fn export_history_csv(wallet: &Wallet, path: &Path) -> Result<(), String> {
    let mut wtr = String::new();
    // UTF-8 BOM for Excel compatibility
    wtr.push_str("\u{FEFF}");
    
    // Header
    wtr.push_str(&format!(
        "Date,Time,Type,Amount BTC,Amount Sat,Confirmations,TxID\n"
    ));

    for tx in &wallet.history {
        let date_str = if let Some(ts) = tx.block_time {
            let dt = DateTime::from_timestamp(ts as i64, 0).unwrap_or_default();
            format!("{},{}", dt.format("%Y-%m-%d"), dt.format("%H:%M:%S"))
        } else {
            ",Pending".to_string()
        };

        let type_str = match tx.direction {
            TxDirection::Incoming => "IN",
            TxDirection::Outgoing => "OUT",
            TxDirection::SelfTransfer => "SELF",
        };

        let amount_btc = tx.amount_sat as f64 / 100_000_000.0;
        let txid = &tx.txid;

        wtr.push_str(&format!(
            "{},{},{:.8},{},{},{}\n",
            date_str,
            type_str,
            amount_btc,
            tx.amount_sat,
            tx.confirmations,
            txid
        ));
    }

    std::fs::write(path, wtr)
        .map_err(|e| format!("{}: {}", t("Không thể ghi file", "Failed to write file"), e))
}

/// Export transaction history to PDF
pub fn export_history_pdf(wallet: &Wallet, path: &Path) -> Result<(), String> {
    let (doc, page, layer) = PdfDocument::new(
        "Transaction History",
        Mm(210.0),
        Mm(297.0),
        "History Layer",
    );
    let mut current_layer = doc.get_page(page).get_layer(layer);

    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|err| format!("Font error: {}", err))?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|err| format!("Font error: {}", err))?;

    // Title
    current_layer.use_text(
        format!("Bitcoin Wallet - {}", wallet.name),
        18.0,
        Mm(15.0),
        Mm(280.0),
        &font_bold,
    );
    
    current_layer.use_text(
        format!("Network: {} | Total Txs: {}", wallet.network.as_str(), wallet.history.len()),
        10.0,
        Mm(15.0),
        Mm(272.0),
        &font_regular,
    );

    // Table Headers
    let mut y_pos = 260.0;
    let col_date = 15.0;
    let col_type = 65.0;
    let col_amount = 115.0;
    let col_txid = 165.0;

    current_layer.use_text("Date", 10.0, Mm(col_date), Mm(y_pos), &font_bold);
    current_layer.use_text("Type", 10.0, Mm(col_type), Mm(y_pos), &font_bold);
    current_layer.use_text("Amount (BTC)", 10.0, Mm(col_amount), Mm(y_pos), &font_bold);
    current_layer.use_text("TxID", 10.0, Mm(col_txid), Mm(y_pos), &font_bold);
    
    y_pos -= 8.0;

    // Rows
    for tx in &wallet.history {
        if y_pos < 15.0 {
            // Add new page
            let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 2");
            current_layer = doc.get_page(new_page).get_layer(new_layer);
            y_pos = 280.0;
        }

        let date_str = if let Some(ts) = tx.block_time {
            let dt = DateTime::from_timestamp(ts as i64, 0).unwrap_or_default();
            dt.format("%d/%m/%Y").to_string()
        } else {
            "Pending".to_string()
        };

        let type_str = match tx.direction {
            TxDirection::Incoming => "IN",
            TxDirection::Outgoing => "OUT",
            TxDirection::SelfTransfer => "SELF",
        };

        let amount_btc = tx.amount_sat as f64 / 100_000_000.0;
        let amount_str = format!("{:.8}", amount_btc);
        let txid_short = if tx.txid.len() > 16 { &tx.txid[..16] } else { &tx.txid };

        // Color based on type (optional, but printpdf colors are complex to apply per text easily without layer switches)
        // Sticking to black for simplicity and reliability.
        
        current_layer.use_text(date_str, 8.0, Mm(col_date), Mm(y_pos), &font_regular);
        current_layer.use_text(type_str, 8.0, Mm(col_type), Mm(y_pos), &font_regular);
        current_layer.use_text(amount_str, 8.0, Mm(col_amount), Mm(y_pos), &font_regular);
        current_layer.use_text(txid_short, 8.0, Mm(col_txid), Mm(y_pos), &font_regular);

        y_pos -= 7.0;
    }

    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer).map_err(|e| e.to_string())
}
