use anyhow::Result;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::key::Secp256k1;
use bitcoin::{Address, PrivateKey};

use super::network::{AddressChain, WalletNetwork};
use super::structure::{AddressEntry, Wallet};
use crate::wallet::secrets::WalletSecrets;

// ─── Address derivation ──────────────────────────────────────────────────

impl Wallet {
    /// Derive next external (receive) addresses
    pub fn derive_next_addresses(
        &mut self,
        secrets: &WalletSecrets,
        count: u32,
    ) -> Result<Vec<String>> {
        self.derive_addresses(secrets, AddressChain::External, count)
    }

    /// Derive next internal (change) addresses
    pub(super) fn derive_next_change_addresses(
        &mut self,
        secrets: &WalletSecrets,
        count: u32,
    ) -> Result<Vec<String>> {
        self.derive_addresses(secrets, AddressChain::Internal, count)
    }

    /// Derive addresses for a specific chain
    fn derive_addresses(
        &mut self,
        secrets: &WalletSecrets,
        chain: AddressChain,
        count: u32,
    ) -> Result<Vec<String>> {
        let secp = Secp256k1::new();
        let account_xprv = Self::parse_account_xprv(secrets)?;
        let next_index = match chain {
            AddressChain::External => &mut self.next_external_index,
            AddressChain::Internal => &mut self.next_internal_index,
        };

        let mut new_addresses = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let index = *next_index;
            let (address, _, _) =
                Self::derive_address_and_keys(&secp, &account_xprv, self.network, chain, index)?;

            self.addresses.push(AddressEntry {
                index,
                address: address.to_string(),
                chain,
            });
            *next_index += 1;
            new_addresses.push(address.to_string());
        }

        Ok(new_addresses)
    }

    /// Derive a single address and its keys
    pub fn derive_address_and_keys(
        secp: &Secp256k1<bitcoin::secp256k1::All>,
        account_xprv: &Xpriv,
        network: WalletNetwork,
        chain: AddressChain,
        index: u32,
    ) -> Result<(Address, PrivateKey, bitcoin::PublicKey)> {
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_normal_idx(chain.branch_index())?,
            ChildNumber::from_normal_idx(index)?,
        ]);

        let child_xprv = account_xprv.derive_priv(secp, &derivation_path)?;
        let private_key = PrivateKey::new(child_xprv.private_key, network.bitcoin_network());
        let public_key = private_key.public_key(secp);
        let compressed_pubkey = bitcoin::CompressedPublicKey::from_slice(&public_key.to_bytes())?;
        let address = Address::p2wpkh(&compressed_pubkey, network.bitcoin_network());

        Ok((address, private_key, public_key))
    }
}
