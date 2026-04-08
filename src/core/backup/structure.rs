use zeroize::Zeroize;

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
                for share in shares {
                    share.zeroize();
                }
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
