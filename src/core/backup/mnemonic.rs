use anyhow::{anyhow, Context, Result};
use bip39::{Language, Mnemonic};
use sssmc39::{combine_mnemonics, generate_mnemonics};

/// Split mnemonic to SLIP39 shares
#[allow(dead_code)]
pub fn split_mnemonic_to_slip39_shares(
    mnemonic_phrase: &str,
    threshold: u8,
    share_count: u8,
    slip39_passphrase: &str,
) -> Result<Vec<String>> {
    if threshold == 0 {
        return Err(anyhow!("Ngưỡng K phải >= 1"));
    }

    if share_count < threshold {
        return Err(anyhow!("Tổng số share N phải >= ngưỡng K"));
    }

    let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
        .context("Mnemonic không hợp lệ")?;
    let entropy = mnemonic.to_entropy();

    let groups = [(threshold, share_count)];
    let generated = generate_mnemonics(1, &groups, &entropy, slip39_passphrase, 0)
        .map_err(|err| anyhow!("Không thể tạo SLIP-0039 shares: {err}"))?;

    let group = generated
        .first()
        .ok_or_else(|| anyhow!("Không tạo được group share SLIP-0039"))?;

    group
        .member_shares
        .iter()
        .map(|share| {
            share
                .to_mnemonic()
                .map(|words| words.join(" "))
                .map_err(|err| anyhow!("Không thể encode SLIP-0039 share: {err}"))
        })
        .collect()
}

/// Combine SLIP39 shares
#[allow(dead_code)]
pub fn combine_slip39_shares(share_phrases: &[String], slip39_passphrase: &str) -> Result<String> {
    if share_phrases.is_empty() {
        return Err(anyhow!("Vui lòng nhập ít nhất một SLIP-0039 share"));
    }

    let parsed_shares = parse_slip39_shares(share_phrases)?;
    let entropy = combine_mnemonics(&parsed_shares, slip39_passphrase)
        .map_err(|err| anyhow!("Không thể khôi phục SLIP-0039 shares: {err}"))?;

    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .context("Entropy khôi phục từ SLIP-0039 không hợp lệ với BIP39")?;

    Ok(mnemonic.to_string())
}

/// Parse SLIP39 shares
#[allow(dead_code)]
fn parse_slip39_shares(share_phrases: &[String]) -> Result<Vec<Vec<String>>> {
    let mut shares = Vec::with_capacity(share_phrases.len());
    for (index, phrase) in share_phrases.iter().enumerate() {
        let normalized = phrase.trim();
        let phrase_body = normalized
            .split_once(':')
            .and_then(|(prefix, rest)| {
                if prefix.trim().to_ascii_lowercase().starts_with("share_") {
                    Some(rest.trim())
                } else {
                    None
                }
            })
            .unwrap_or(normalized);

        let words = phrase_body
            .split_whitespace()
            .map(|word| word.trim().to_ascii_lowercase())
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();

        if words.is_empty() {
            return Err(anyhow!("SLIP-0039 share #{} đang để trống", index + 1));
        }

        shares.push(words);
    }

    Ok(shares)
}
