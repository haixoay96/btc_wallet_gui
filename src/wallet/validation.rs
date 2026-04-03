use std::str::FromStr;

use bitcoin::Address;

use crate::i18n::t;

use super::WalletNetwork;

pub fn validate_bitcoin_address(address: &str) -> Result<(), String> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        return Err(t("Địa chỉ không được rỗng", "Address cannot be empty").to_string());
    }

    Address::from_str(trimmed)
        .map(|_| ())
        .map_err(|_| t("Địa chỉ Bitcoin không hợp lệ", "Invalid Bitcoin address").to_string())
}

pub fn validate_address_for_network(address: &str, network: WalletNetwork) -> Result<(), String> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        return Err(t("Địa chỉ không được rỗng", "Address cannot be empty").to_string());
    }

    let unchecked = Address::from_str(trimmed)
        .map_err(|_| t("Địa chỉ Bitcoin không hợp lệ", "Invalid Bitcoin address").to_string())?;

    unchecked
        .require_network(network.bitcoin_network())
        .map(|_| ())
        .map_err(|_| {
            t(
                "Địa chỉ nhận không đúng network của ví",
                "Recipient address does not match wallet network",
            )
            .to_string()
        })
}

#[cfg(test)]
mod tests {
    use super::{validate_address_for_network, validate_bitcoin_address};
    use crate::wallet::WalletNetwork;

    #[test]
    fn accepts_known_mainnet_and_testnet_addresses() {
        assert!(validate_bitcoin_address("bc1qvzvkjn4q3nszqxrv3nraga2r822xjty3ykvkuw").is_ok());
        assert!(validate_bitcoin_address(
            "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7"
        )
        .is_ok());
    }

    #[test]
    fn rejects_invalid_or_wrong_network_addresses() {
        assert!(validate_bitcoin_address("not-an-address").is_err());
        assert!(validate_address_for_network(
            "tb1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
            WalletNetwork::Mainnet
        )
        .is_err());
    }
}
