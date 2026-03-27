use anyhow::{anyhow, Context, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use getrandom::fill;
use serde::{Deserialize, Serialize};

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

const ARGON2_M_COST_KIB: u32 = 64 * 1024;
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    version: u8,
    salt_b64: String,
    nonce_b64: String,
    ciphertext_b64: String,
}

pub fn encrypt_blob(plaintext: &[u8], passphrase: &str) -> Result<EncryptedEnvelope> {
    let passphrase = normalize_passphrase(passphrase)?;

    let mut salt = [0u8; SALT_LEN];
    fill(&mut salt).context("Không tạo được salt ngẫu nhiên")?;

    let mut nonce = [0u8; NONCE_LEN];
    fill(&mut nonce).context("Không tạo được nonce ngẫu nhiên")?;

    let key_bytes = derive_key(passphrase, &salt)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|_| anyhow!("Mã hóa dữ liệu thất bại"))?;

    Ok(EncryptedEnvelope {
        version: 1,
        salt_b64: STANDARD_NO_PAD.encode(salt),
        nonce_b64: STANDARD_NO_PAD.encode(nonce),
        ciphertext_b64: STANDARD_NO_PAD.encode(ciphertext),
    })
}

pub fn decrypt_blob(envelope: &EncryptedEnvelope, passphrase: &str) -> Result<Vec<u8>> {
    if envelope.version != 1 {
        return Err(anyhow!(
            "Version encrypted payload không hỗ trợ: {}",
            envelope.version
        ));
    }

    let passphrase = normalize_passphrase(passphrase)?;

    let salt = STANDARD_NO_PAD
        .decode(&envelope.salt_b64)
        .context("Salt trong file encrypted không hợp lệ")?;
    let nonce = STANDARD_NO_PAD
        .decode(&envelope.nonce_b64)
        .context("Nonce trong file encrypted không hợp lệ")?;
    let ciphertext = STANDARD_NO_PAD
        .decode(&envelope.ciphertext_b64)
        .context("Ciphertext trong file encrypted không hợp lệ")?;

    if salt.len() != SALT_LEN {
        return Err(anyhow!("Salt length không hợp lệ"));
    }

    if nonce.len() != NONCE_LEN {
        return Err(anyhow!("Nonce length không hợp lệ"));
    }

    let key_bytes = derive_key(passphrase, &salt)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));

    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| anyhow!("Giải mã thất bại. Sai passphrase hoặc file bị hỏng"))?;

    Ok(plaintext)
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; KEY_LEN]> {
    let params = Params::new(
        ARGON2_M_COST_KIB,
        ARGON2_T_COST,
        ARGON2_P_COST,
        Some(KEY_LEN),
    )
    .map_err(|err| anyhow!("Argon2 params không hợp lệ: {err:?}"))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|err| anyhow!("Không derive được key từ passphrase: {err:?}"))?;

    Ok(key)
}

fn normalize_passphrase(passphrase: &str) -> Result<&str> {
    if passphrase.trim().is_empty() {
        return Err(anyhow!("Passphrase không được để trống"));
    }

    Ok(passphrase)
}
