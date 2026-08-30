//! Lab 03 — wrap a 2-of-3 multisig rule in P2SH.

use std::str::FromStr;

use bitcoin::opcodes::all::OP_CHECKMULTISIG;
use bitcoin::script::Builder;
use bitcoin::{Address, Network, PublicKey, ScriptBuf};

use crate::model::P2shReport;
use crate::{LabError, LabResult};

fn parse_script(redeem_script_hex: &str) -> LabResult<ScriptBuf> {
    let bytes = hex::decode(redeem_script_hex)
        .map_err(|e| LabError::InvalidScript(format!("invalid redeemScript hex: {e}")))?;
    Ok(ScriptBuf::from_bytes(bytes))
}

/// Build `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`.
pub fn build_2_of_3_redeem_script(public_keys: [&str; 3]) -> LabResult<String> {
    let pks: Result<Vec<PublicKey>, _> = public_keys.iter().map(|s| PublicKey::from_str(s)).collect();
    let pks = pks.map_err(|e| LabError::InvalidKey(format!("invalid public key in 2-of-3 multisig: {e}")))?;

    let script = Builder::new()
        .push_int(2)
        .push_key(&pks[0])
        .push_key(&pks[1])
        .push_key(&pks[2])
        .push_int(3)
        .push_opcode(OP_CHECKMULTISIG)
        .into_script();

    Ok(script.to_hex_string())
}

/// Derive the P2SH address that commits to a redeemScript.
pub fn derive_p2sh_address(redeem_script_hex: &str, network: Network) -> LabResult<String> {
    let script = parse_script(redeem_script_hex)?;
    Address::p2sh(&script, network)
        .map(|addr| addr.to_string())
        .map_err(|e| LabError::InvalidAddress(format!("failed to derive P2SH address: {e}")))
}

/// Return the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey.
pub fn build_p2sh_script_pubkey(redeem_script_hex: &str) -> LabResult<String> {
    let script = parse_script(redeem_script_hex)?;
    Address::p2sh(&script, Network::Bitcoin)
        .map(|addr| addr.script_pubkey().to_hex_string())
        .map_err(|e| LabError::InvalidAddress(format!("failed to build P2SH scriptPubKey: {e}")))
}

/// Collect the inner script, outer address, and scriptPubKey in one report.
pub fn inspect_p2sh_multisig(public_keys: [&str; 3], network: Network) -> LabResult<P2shReport> {
    let redeem_script_hex = build_2_of_3_redeem_script(public_keys)?;
    let address = derive_p2sh_address(&redeem_script_hex, network)?;
    let script_pubkey_hex = build_p2sh_script_pubkey(&redeem_script_hex)?;

    Ok(P2shReport {
        redeem_script_hex,
        address,
        script_pubkey_hex,
    })
}
