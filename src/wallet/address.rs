use super::*;

pub fn create_new_wallet(name: &str, network: WalletNetwork) -> Result<Wallet> {
    let mnemonic = Mnemonic::generate_in(Language::English, 12)?;
    create_wallet_from_mnemonic(name, network, mnemonic)
}

pub fn import_wallet_from_mnemonic(
    name: &str,
    network: WalletNetwork,
    mnemonic_phrase: &str,
) -> Result<Wallet> {
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
        .context("Mnemonic không hợp lệ")?;
    create_wallet_from_mnemonic(name, network, mnemonic)
}

pub fn import_wallet_from_account_xprv(
    name: &str,
    network: WalletNetwork,
    account_xprv: &str,
) -> Result<Wallet> {
    let secp = Secp256k1::new();
    let parsed_xprv = Xpriv::from_str(account_xprv).context("xprv không hợp lệ")?;
    let account_xpub = Xpub::from_priv(&secp, &parsed_xprv);

    let mut entry = WalletEntry {
        name: name.trim().to_string(),
        network,
        mnemonic: None,
        account_xprv: parsed_xprv.to_string(),
        account_xpub: account_xpub.to_string(),
        next_index: 0,
        addresses: Vec::new(),
        history: Vec::new(),
    };

    derive_next_addresses(&mut entry, DEFAULT_GAP_LIMIT)?;
    Ok(Wallet { entry })
}

fn create_wallet_from_mnemonic(
    name: &str,
    network: WalletNetwork,
    mnemonic: Mnemonic,
) -> Result<Wallet> {
    let secp = Secp256k1::new();
    let seed = mnemonic.to_seed_normalized("");
    let root_xprv = Xpriv::new_master(network.bitcoin_network(), &seed)?;

    let account_path = DerivationPath::from(vec![
        ChildNumber::from_hardened_idx(84)?,
        ChildNumber::from_hardened_idx(network.coin_type())?,
        ChildNumber::from_hardened_idx(0)?,
    ]);

    let account_xprv = root_xprv.derive_priv(&secp, &account_path)?;
    let account_xpub = Xpub::from_priv(&secp, &account_xprv);

    let mut entry = WalletEntry {
        name: name.trim().to_string(),
        network,
        mnemonic: Some(mnemonic.to_string()),
        account_xprv: account_xprv.to_string(),
        account_xpub: account_xpub.to_string(),
        next_index: 0,
        addresses: Vec::new(),
        history: Vec::new(),
    };

    derive_next_addresses(&mut entry, DEFAULT_GAP_LIMIT)?;
    Ok(Wallet { entry })
}

pub fn derive_next_addresses(entry: &mut WalletEntry, count: u32) -> Result<Vec<String>> {
    let secp = Secp256k1::new();
    let account_xprv = Xpriv::from_str(&entry.account_xprv)?;

    let mut new_addresses = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let index = entry.next_index;
        let (address, _, _) = derive_address_and_keys(&secp, &account_xprv, entry.network, index)?;

        entry.addresses.push(AddressEntry {
            index,
            address: address.to_string(),
        });
        entry.next_index += 1;
        new_addresses.push(address.to_string());
    }

    Ok(new_addresses)
}

pub(super) fn derive_address_and_keys(
    secp: &Secp256k1<bitcoin::secp256k1::All>,
    account_xprv: &Xpriv,
    network: WalletNetwork,
    index: u32,
) -> Result<(Address, PrivateKey, bitcoin::PublicKey)> {
    let derivation_path = DerivationPath::from(vec![
        ChildNumber::from_normal_idx(0)?,
        ChildNumber::from_normal_idx(index)?,
    ]);

    let child_xprv = account_xprv.derive_priv(secp, &derivation_path)?;
    let private_key = PrivateKey::new(child_xprv.private_key, network.bitcoin_network());
    let public_key = private_key.public_key(secp);
    let compressed_pubkey = CompressedPublicKey::from_slice(&public_key.to_bytes())?;
    let address = Address::p2wpkh(&compressed_pubkey, network.bitcoin_network());

    Ok((address, private_key, public_key))
}