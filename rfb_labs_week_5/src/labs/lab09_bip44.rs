//! Lab 09 — decode BIP44 paths and derive the selected address.

use std::str::FromStr;

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, PublicKey};

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

fn child_number_value(cn: ChildNumber) -> (u32, bool) {
    match cn {
        ChildNumber::Hardened { index } => (index, true),
        ChildNumber::Normal { index } => (index, false),
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path = DerivationPath::from_str(path)
        .map_err(|e| LabError::InvalidPath(format!("invalid derivation path '{path}': {e}")))?;

    let elements: Vec<ChildNumber> = derivation_path.into_iter().copied().collect();
    if elements.len() != 5 {
        return Err(LabError::InvalidPath(format!(
            "BIP44 path must have exactly 5 elements, got {}",
            elements.len()
        )));
    }

    let (purpose, p_hardened) = child_number_value(elements[0]);
    let (coin_type, c_hardened) = child_number_value(elements[1]);
    let (account, a_hardened) = child_number_value(elements[2]);
    let (change, ch_hardened) = child_number_value(elements[3]);
    let (index, idx_hardened) = child_number_value(elements[4]);

    if !p_hardened || !c_hardened || !a_hardened || ch_hardened || idx_hardened {
        return Err(LabError::InvalidPath(format!(
            "invalid BIP44 hardening flags for path '{path}'"
        )));
    }

    Ok(Bip44PathInfo {
        purpose,
        coin_type,
        account,
        change,
        index,
    })
}

fn ordinal_string(n: u32) -> String {
    match n {
        1 => "first".to_string(),
        2 => "second".to_string(),
        3 => "third".to_string(),
        4 => "fourth".to_string(),
        5 => "fifth".to_string(),
        6 => "sixth".to_string(),
        7 => "seventh".to_string(),
        8 => "eighth".to_string(),
        9 => "ninth".to_string(),
        10 => "tenth".to_string(),
        other => format!("{other}th"),
    }
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    let account_ord = ordinal_string(info.account + 1);
    let address_ord = ordinal_string(info.index + 1);
    let branch_str = if info.change == 0 {
        "external/receive branch"
    } else {
        "internal/change branch"
    };

    format!(
        "BIP44 path for purpose {}', coin type {}', {} account, {}, {} address (index {})",
        info.purpose, info.coin_type, account_ord, branch_str, address_ord, info.index
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    let _info = decode_bip44_path(path)?;
    let mut parts: Vec<&str> = path.split('/').collect();
    let new_idx_str = new_index.to_string();
    let last = parts.last_mut().ok_or_else(|| LabError::InvalidPath("empty path".into()))?;
    *last = &new_idx_str;
    Ok(parts.join("/"))
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic_str: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<String> {
    let m = Mnemonic::parse_in(Language::English, mnemonic_str)
        .map_err(|e| LabError::InvalidMnemonic(format!("invalid mnemonic: {e}")))?;
    let seed = m.to_seed(passphrase);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network, &seed)
        .map_err(|e| LabError::Derivation(format!("failed to create master xpriv: {e}")))?;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|e| LabError::InvalidPath(format!("invalid path '{path}': {e}")))?;

    let derived_xpriv = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| LabError::Derivation(format!("failed to derive xpriv: {e}")))?;

    let derived_xpub = Xpub::from_priv(&secp, &derived_xpriv);
    let pubkey = PublicKey::new(derived_xpub.public_key);

    Ok(Address::p2pkh(pubkey, network).to_string())
}
