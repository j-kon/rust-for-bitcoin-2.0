//! Lab 08 — derive BIP32 extended private and public keys.

use std::str::FromStr;

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;

use crate::model::ExtendedKeyReport;
use crate::{LabError, LabResult};

fn seed_from_mnemonic(mnemonic_str: &str, passphrase: &str) -> LabResult<Vec<u8>> {
    let m = Mnemonic::parse_in(Language::English, mnemonic_str)
        .map_err(|e| LabError::InvalidMnemonic(format!("invalid mnemonic: {e}")))?;
    Ok(m.to_seed(passphrase).to_vec())
}

/// Create the master extended private key from a BIP39 recovery setup.
pub fn master_xpriv(mnemonic: &str, passphrase: &str, network: Network) -> LabResult<String> {
    let seed = seed_from_mnemonic(mnemonic, passphrase)?;
    let master = Xpriv::new_master(network, &seed)
        .map_err(|e| LabError::Derivation(format!("failed to create master xpriv: {e}")))?;
    Ok(master.to_string())
}

/// Derive an extended private/public key pair at a complete path.
pub fn derive_extended_keys(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    network: Network,
) -> LabResult<ExtendedKeyReport> {
    let seed = seed_from_mnemonic(mnemonic, passphrase)?;
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network, &seed)
        .map_err(|e| LabError::Derivation(format!("failed to create master xpriv: {e}")))?;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|e| LabError::InvalidPath(format!("invalid derivation path '{path}': {e}")))?;

    let derived_xpriv = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| LabError::Derivation(format!("failed to derive xpriv: {e}")))?;

    let derived_xpub = Xpub::from_priv(&secp, &derived_xpriv);

    Ok(ExtendedKeyReport {
        derivation_path: path.to_string(),
        xpriv: derived_xpriv.to_string(),
        xpub: derived_xpub.to_string(),
    })
}

/// Derive a normal public child from an xpub without private key material.
pub fn derive_normal_child_xpub(parent_xpub_str: &str, index: u32) -> LabResult<String> {
    let parent_xpub = Xpub::from_str(parent_xpub_str)
        .map_err(|e| LabError::InvalidKey(format!("invalid parent xpub: {e}")))?;
    let secp = Secp256k1::new();

    let child_num = ChildNumber::from_normal_idx(index)
        .map_err(|e| LabError::InvalidPath(format!("invalid normal child index: {e}")))?;

    let child_xpub = parent_xpub
        .derive_pub(&secp, &[child_num])
        .map_err(|e| LabError::Derivation(format!("failed to derive child xpub: {e}")))?;

    Ok(child_xpub.to_string())
}

/// Return whether a textual path contains at least one hardened step.
pub fn path_contains_hardened_step(path: &str) -> LabResult<bool> {
    let derivation_path = DerivationPath::from_str(path)
        .map_err(|e| LabError::InvalidPath(format!("invalid derivation path '{path}': {e}")))?;

    Ok(derivation_path
        .into_iter()
        .any(|child_num| child_num.is_hardened()))
}
