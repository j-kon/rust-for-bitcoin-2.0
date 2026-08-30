//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use std::str::FromStr;

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};

use crate::model::{AddressFormat, DerivedAddressSet};
use crate::{LabError, LabResult};

/// Derive one address from an arbitrary full path and selected script family.
pub fn derive_address_for_path(
    mnemonic_str: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<String> {
    let m = Mnemonic::parse_in(Language::English, mnemonic_str)
        .map_err(|e| LabError::InvalidMnemonic(format!("invalid mnemonic: {e}")))?;
    let seed = m.to_seed(passphrase);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network, &seed)
        .map_err(|e| LabError::Derivation(format!("failed to create master xpriv: {e}")))?;

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|e| LabError::InvalidPath(format!("invalid derivation path '{path}': {e}")))?;

    let derived_xpriv = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| LabError::Derivation(format!("failed to derive xpriv: {e}")))?;

    let derived_xpub = Xpub::from_priv(&secp, &derived_xpriv);
    let pubkey = PublicKey::new(derived_xpub.public_key);

    match format {
        AddressFormat::P2pkh => Ok(Address::p2pkh(pubkey, network).to_string()),
        AddressFormat::P2sh => {
            let compressed = CompressedPublicKey::try_from(pubkey)
                .map_err(|e| LabError::InvalidKey(format!("public key is not compressed: {e}")))?;
            Ok(Address::p2shwpkh(&compressed, network).to_string())
        }
        AddressFormat::P2wpkh => {
            let compressed = CompressedPublicKey::try_from(pubkey)
                .map_err(|e| LabError::InvalidKey(format!("public key is not compressed: {e}")))?;
            Ok(Address::p2wpkh(&compressed, network).to_string())
        }
        AddressFormat::P2tr => {
            let (x_only, _) = derived_xpub.public_key.x_only_public_key();
            Ok(Address::p2tr(&secp, x_only, None, network).to_string())
        }
        AddressFormat::Unknown => Err(LabError::InvalidAddress("unknown address format".into())),
    }
}

/// Derive index `n` on the BIP44, BIP49, and BIP84 receive branches.
pub fn derive_address_set(
    mnemonic: &str,
    passphrase: &str,
    account: u32,
    index: u32,
    network: Network,
) -> LabResult<DerivedAddressSet> {
    let coin = if network == Network::Bitcoin { 0 } else { 1 };

    let bip44_path = format!("m/44'/{coin}'/{account}'/0/{index}");
    let bip49_path = format!("m/49'/{coin}'/{account}'/0/{index}");
    let bip84_path = format!("m/84'/{coin}'/{account}'/0/{index}");

    let bip44_p2pkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &bip44_path,
        AddressFormat::P2pkh,
        network,
    )?;
    let bip49_p2sh_p2wpkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &bip49_path,
        AddressFormat::P2sh,
        network,
    )?;
    let bip84_p2wpkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &bip84_path,
        AddressFormat::P2wpkh,
        network,
    )?;

    Ok(DerivedAddressSet {
        bip44_p2pkh,
        bip49_p2sh_p2wpkh,
        bip84_p2wpkh,
    })
}

/// Prove that identical mnemonic, passphrase, path, and network reproduce an address.
pub fn recovery_is_repeatable(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<bool> {
    let addr1 = derive_address_for_path(mnemonic, passphrase, path, format, network)?;
    let addr2 = derive_address_for_path(mnemonic, passphrase, path, format, network)?;
    Ok(addr1 == addr2)
}

/// Prove that changing only the final index selects a different address.
pub fn changing_index_changes_address(
    mnemonic: &str,
    passphrase: &str,
    first_path: &str,
    second_path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<bool> {
    let addr1 = derive_address_for_path(mnemonic, passphrase, first_path, format, network)?;
    let addr2 = derive_address_for_path(mnemonic, passphrase, second_path, format, network)?;
    Ok(addr1 != addr2)
}
