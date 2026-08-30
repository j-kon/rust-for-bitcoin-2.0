//! Lab 04 — construct and explain native SegWit P2WPKH.

use std::str::FromStr;

use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};

use crate::model::{NativeSegwitSpend, WitnessProgramReport};
use crate::{LabError, LabResult};

fn parse_compressed_pubkey(public_key_hex: &str) -> LabResult<CompressedPublicKey> {
    let pubkey = PublicKey::from_str(public_key_hex)
        .map_err(|e| LabError::InvalidKey(format!("invalid public key '{public_key_hex}': {e}")))?;
    CompressedPublicKey::try_from(pubkey)
        .map_err(|e| LabError::InvalidKey(format!("public key is not compressed: {e}")))
}

/// Derive a native P2WPKH address from a compressed public key.
pub fn derive_p2wpkh_address(public_key_hex: &str, network: Network) -> LabResult<String> {
    let compressed = parse_compressed_pubkey(public_key_hex)?;
    Ok(Address::p2wpkh(&compressed, network).to_string())
}

/// Build the P2WPKH `0 <20-byte-pubkey-hash>` scriptPubKey.
pub fn build_p2wpkh_script_pubkey(public_key_hex: &str) -> LabResult<String> {
    let compressed = parse_compressed_pubkey(public_key_hex)?;
    Ok(Address::p2wpkh(&compressed, Network::Bitcoin)
        .script_pubkey()
        .to_hex_string())
}

/// Report the witness version and program committed by P2WPKH.
pub fn witness_program(public_key_hex: &str) -> LabResult<WitnessProgramReport> {
    let compressed = parse_compressed_pubkey(public_key_hex)?;
    let wpubkey_hash = compressed.wpubkey_hash();
    let program_hex = wpubkey_hash.to_string();

    Ok(WitnessProgramReport {
        version: 0,
        program_length: 20,
        program_hex,
    })
}

/// Put the signature and public key in witness while leaving ScriptSig empty.
pub fn native_spend_template(
    signature_hex: &str,
    public_key_hex: &str,
) -> LabResult<NativeSegwitSpend> {
    let _compressed = parse_compressed_pubkey(public_key_hex)?;
    hex::decode(signature_hex)
        .map_err(|e| LabError::InvalidScript(format!("invalid signature hex: {e}")))?;

    Ok(NativeSegwitSpend {
        script_sig_hex: String::new(),
        witness_items: vec![signature_hex.to_string(), public_key_hex.to_string()],
    })
}
