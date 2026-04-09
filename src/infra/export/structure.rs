use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::infra::storage::{decrypt_blob, encrypt_blob, EncryptedEnvelope};

const ENCRYPTED_SECRET_EXPORT_FORMAT: &str = "btc_wallet_gui_encrypted_export";
const ENCRYPTED_SECRET_EXPORT_VERSION: u8 = 1;

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

pub struct Slip39PdfExport<'a> {
    pub wallet_name: &'a str,
    pub network: &'a str,
    pub threshold: u8,
    pub share_count: u8,
    pub has_slip39_passphrase: bool,
}

// Internal serialization structs
#[derive(Serialize)]
pub(crate) struct EncryptedSecretExport<'a> {
    pub(crate) format: &'static str,
    pub(crate) version: u8,
    pub(crate) kind: &'static str,
    pub(crate) envelope: EncryptedEnvelope,
    #[serde(skip)]
    pub(crate) _marker: std::marker::PhantomData<&'a ()>,
}

impl EncryptedSecretExport<'_> {
    pub(crate) fn new(kind: &'static str, envelope: EncryptedEnvelope) -> Self {
        Self {
            format: ENCRYPTED_SECRET_EXPORT_FORMAT,
            version: ENCRYPTED_SECRET_EXPORT_VERSION,
            kind,
            envelope,
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct StoredEncryptedSecretExport {
    pub(crate) format: String,
    pub(crate) version: u8,
    pub(crate) kind: String,
    pub(crate) envelope: EncryptedEnvelope,
}

#[derive(Serialize)]
pub(crate) struct MnemonicEncryptedPayload<'a> {
    pub(crate) wallet_name: &'a str,
    pub(crate) network: &'a str,
    pub(crate) mnemonic: &'a str,
}

#[derive(Deserialize)]
pub(crate) struct StoredMnemonicEncryptedPayload {
    pub(crate) wallet_name: String,
    pub(crate) network: String,
    pub(crate) mnemonic: String,
}

#[derive(Deserialize)]
pub(crate) struct StoredSlip39EncryptedPayload {
    pub(crate) wallet_name: String,
    pub(crate) network: String,
    pub(crate) threshold: u8,
    pub(crate) share_count: u8,
    pub(crate) slip39_passphrase: String,
    pub(crate) shares: Vec<String>,
}
