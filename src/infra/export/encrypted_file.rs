use serde::Serialize;
use std::path::Path;
use zeroize::Zeroize;

use super::structure::{
    DecryptedSecretExport, EncryptedSecretExport, StoredEncryptedSecretExport,
    StoredMnemonicEncryptedPayload, StoredSlip39EncryptedPayload,
};

const ENCRYPTED_SECRET_EXPORT_FORMAT: &str = "btc_wallet_gui_encrypted_export";
const ENCRYPTED_SECRET_EXPORT_VERSION: u8 = 1;

pub fn write_encrypted_export<T: Serialize>(
    path: &Path,
    kind: &'static str,
    payload: &T,
    encryption_passphrase: &str,
) -> Result<(), String> {
    use crate::i18n::t;

    let mut plaintext = serde_json::to_vec(payload).map_err(|err| {
        format!(
            "{}: {err}",
            t(
                "Không serialize được dữ liệu secret",
                "Could not serialize secret data"
            )
        )
    })?;

    let envelope = crate::storage::encrypt_blob(&plaintext, encryption_passphrase)
        .map_err(|err| err.to_string());
    plaintext.zeroize();
    let envelope = envelope?;

    let export = EncryptedSecretExport::new(kind, envelope);
    let encoded = serde_json::to_vec_pretty(&export).map_err(|err| {
        format!(
            "{}: {err}",
            t(
                "Không serialize được file export mã hóa",
                "Could not serialize encrypted export file"
            )
        )
    })?;

    let parent: &Path = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|err| {
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
    std::fs::write(&tmp_path, encoded).map_err(|err| {
        format!(
            "{} {}: {err}",
            t(
                "Không ghi được file export tạm",
                "Could not write temporary export file"
            ),
            tmp_path.display()
        )
    })?;

    std::fs::rename(&tmp_path, path).map_err(|err| {
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

pub fn decode_encrypted_secret_export(
    encoded: &[u8],
    decryption_passphrase: &str,
) -> Result<DecryptedSecretExport, String> {
    use crate::i18n::t;

    let export: StoredEncryptedSecretExport = serde_json::from_slice(encoded).map_err(|err| {
        format!(
            "{}: {err}",
            t(
                "File backup mã hóa không đúng định dạng JSON",
                "Encrypted backup file is not valid JSON"
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
                "Unsupported encrypted backup file version"
            ),
            export.version
        ));
    }

    let mut plaintext = crate::storage::decrypt_blob(&export.envelope, decryption_passphrase)
        .map_err(|err| err.to_string())?;

    let decoded = match export.kind.as_str() {
        "mnemonic_backup" => {
            let payload: StoredMnemonicEncryptedPayload = serde_json::from_slice(&plaintext)
                .map_err(|err| {
                    format!(
                        "{}: {err}",
                        t(
                            "Payload mnemonic mã hóa không hợp lệ",
                            "Encrypted mnemonic payload is invalid"
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
                            "Encrypted SLIP-0039 payload is invalid"
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
                "Unsupported encrypted backup kind"
            ),
            export.kind
        )),
    };

    plaintext.zeroize();
    decoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique_temp_path(file_name: &str) -> std::path::PathBuf {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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
        use super::write_encrypted_export;

        let path = unique_temp_path("mnemonic.enc");
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        #[derive(Serialize)]
        struct Payload<'a> {
            wallet_name: &'a str,
            network: &'a str,
            mnemonic: &'a str,
        }

        write_encrypted_export(
            &path,
            "mnemonic_backup",
            &Payload {
                wallet_name: "Primary",
                network: "testnet",
                mnemonic,
            },
            "correct horse battery staple",
        )
        .expect("encrypted mnemonic export should succeed");

        let encoded = fs::read(&path).expect("file should be readable");
        let decoded = decode_encrypted_secret_export(&encoded, "correct horse battery staple")
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
