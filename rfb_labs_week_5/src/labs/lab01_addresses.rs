//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use std::str::FromStr;

use bitcoin::{Address, Network};

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    if address.starts_with('1') || address.starts_with('m') || address.starts_with('n') {
        AddressFormat::P2pkh
    } else if address.starts_with('3') || address.starts_with('2') {
        AddressFormat::P2sh
    } else if address.starts_with("bc1q")
        || address.starts_with("tb1q")
        || address.starts_with("bcrt1q")
    {
        AddressFormat::P2wpkh
    } else if address.starts_with("bc1p")
        || address.starts_with("tb1p")
        || address.starts_with("bcrt1p")
    {
        AddressFormat::P2tr
    } else {
        AddressFormat::Unknown
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    match (format, network) {
        (AddressFormat::P2pkh, Network::Bitcoin) => Some("1"),
        (AddressFormat::P2pkh, Network::Testnet | Network::Signet | Network::Regtest) => {
            Some("m/n")
        }
        (AddressFormat::P2sh, Network::Bitcoin) => Some("3"),
        (AddressFormat::P2sh, Network::Testnet | Network::Signet | Network::Regtest) => Some("2"),
        (AddressFormat::P2wpkh, Network::Bitcoin) => Some("bc1q"),
        (AddressFormat::P2wpkh, Network::Testnet | Network::Signet) => Some("tb1q"),
        (AddressFormat::P2wpkh, Network::Regtest) => Some("bcrt1q"),
        (AddressFormat::P2tr, Network::Bitcoin) => Some("bc1p"),
        (AddressFormat::P2tr, Network::Testnet | Network::Signet) => Some("tb1p"),
        (AddressFormat::P2tr, Network::Regtest) => Some("bcrt1p"),
        _ => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address_str: &str, network: Network) -> LabResult<AddressReport> {
    let unchecked_addr = Address::from_str(address_str)
        .map_err(|e| LabError::InvalidAddress(format!("failed to parse address: {e}")))?;

    let checked_addr = unchecked_addr.require_network(network).map_err(|e| {
        LabError::WrongNetwork(format!("address network mismatch for '{address_str}': {e}"))
    })?;

    let format = match checked_addr.address_type() {
        Some(bitcoin::AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(bitcoin::AddressType::P2sh) => AddressFormat::P2sh,
        Some(bitcoin::AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(bitcoin::AddressType::P2tr) => AddressFormat::P2tr,
        _ => AddressFormat::Unknown,
    };

    let script_pubkey_hex = checked_addr.script_pubkey().to_hex_string();

    Ok(AddressReport {
        address: checked_addr.to_string(),
        network: network.to_string(),
        format,
        script_pubkey_hex,
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    inspect_address(address, network).map(|report| report.script_pubkey_hex)
}
