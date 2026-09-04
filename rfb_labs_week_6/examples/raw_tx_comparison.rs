//! Educational Comparison: Low-level `rust-bitcoin` primitives vs. high-level BDK abstractions
//!
//! This example illustrates where low-level `rust-bitcoin` is necessary or preferable
//! compared to BDK:
//! - Custom off-chain protocols (Lightning channels, state chains, discreet log contracts)
//! - Non-standard transaction construction
//! - Direct cryptographic control over sighash computation and witness assembly
//! - Lightweight environments where descriptor tracking, SQLite, and coin selection are not needed.

use bitcoin::absolute::LockTime;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{Message, Secp256k1, SecretKey};
use bitcoin::sighash::{EcdsaSighashType, SighashCache};
use bitcoin::transaction::Version;
use bitcoin::{
    Address, Amount, CompressedPublicKey, Network, OutPoint, ScriptBuf, Sequence, Transaction,
    TxIn, TxOut, Witness,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Raw rust-bitcoin Transaction Construction & Signing ===");

    // 1. Generate local key material with secp256k1
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&[0x11; 32])?;
    let pubkey = CompressedPublicKey::from_private_key(&secp, &bitcoin::PrivateKey::new(secret_key, Network::Regtest))?;
    let sender_addr = Address::p2wpkh(&pubkey, Network::Regtest);

    println!("Sender P2WPKH Address: {}", sender_addr);

    // 2. Define UTXO being spent
    let prev_txid = bitcoin::Txid::all_zeros();
    let prev_vout = 0;
    let prev_amount = Amount::from_sat(100_000);

    let recipient_addr = Address::p2wpkh(&pubkey, Network::Regtest);
    let send_amount = Amount::from_sat(90_000);

    // 3. Assemble Transaction Inputs and Outputs
    let tx_in = TxIn {
        previous_output: OutPoint::new(prev_txid, prev_vout),
        script_sig: ScriptBuf::new(), // SegWit v0 uses empty scriptSig
        sequence: Sequence::MAX,
        witness: Witness::new(), // Will be populated after signing
    };

    let tx_out = TxOut {
        value: send_amount,
        script_pubkey: recipient_addr.script_pubkey(),
    };

    let mut unsigned_tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![tx_in],
        output: vec![tx_out],
    };

    println!("Unsigned TXID: {}", unsigned_tx.compute_txid());

    // 4. Compute BIP143 SegWit v0 Sighash using SighashCache
    let mut sighash_cache = SighashCache::new(&mut unsigned_tx);
    let sighash = sighash_cache.p2wpkh_signature_hash(
        0,
        &sender_addr.script_pubkey(),
        prev_amount,
        EcdsaSighashType::All,
    )?;

    // 5. Sign the sighash directly with secp256k1
    let msg = Message::from_digest(sighash.to_byte_array());
    let sig = secp.sign_ecdsa(&msg, &secret_key);

    let mut sig_bytes = sig.serialize_der().to_vec();
    sig_bytes.push(EcdsaSighashType::All as u8);

    // 6. Construct Witness: [ <signature> <compressed_pubkey> ]
    let mut witness = Witness::new();
    witness.push(&sig_bytes);
    witness.push(pubkey.to_bytes());

    // 7. Attach witness to input
    unsigned_tx.input[0].witness = witness;

    let signed_tx = unsigned_tx;
    let final_txid = signed_tx.compute_txid();
    let raw_hex = bitcoin::consensus::encode::serialize_hex(&signed_tx);

    println!("=== Final Signed Transaction ===");
    println!("Signed TXID:           {}", final_txid);
    println!("Witness Stack Elements: {}", signed_tx.input[0].witness.len());
    println!("Serialized Raw Hex:    {}", raw_hex);
    println!();
    println!("Contrast with BDK:");
    println!("- rust-bitcoin: Manual coin selection, manual fee calculation, manual sighash cache, and manual witness assembly.");
    println!("- BDK: High-level TxBuilder handles coin selection, change output derivation from internal keychain, fee rate calculation, and automatic PSBT signing from descriptors.");

    Ok(())
}
