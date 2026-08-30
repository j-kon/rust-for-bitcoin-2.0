//! Lab 07 — validate BIP39 recovery words and derive seeds safely.

use bip39::{Language, Mnemonic};

use crate::model::{MnemonicReport, PassphraseComparison};
use crate::{LabError, LabResult};

fn parse_mnemonic(mnemonic_str: &str) -> LabResult<Mnemonic> {
    Mnemonic::parse_in(Language::English, mnemonic_str)
        .map_err(|e| LabError::InvalidMnemonic(format!("invalid mnemonic: {e}")))
}

/// Validate an English mnemonic and report its entropy/checksum structure.
pub fn inspect_mnemonic(mnemonic: &str) -> LabResult<MnemonicReport> {
    let m = parse_mnemonic(mnemonic)?;
    let word_count = m.word_count();
    let total_bits = word_count * 11;
    let entropy_bits = (total_bits * 32) / 33;
    let checksum_bits = total_bits - entropy_bits;

    Ok(MnemonicReport {
        word_count,
        entropy_bits,
        checksum_bits,
    })
}

/// Derive the 512-bit BIP39 seed from words plus an optional passphrase.
pub fn mnemonic_seed_hex(mnemonic: &str, passphrase: &str) -> LabResult<String> {
    let m = parse_mnemonic(mnemonic)?;
    let seed = m.to_seed(passphrase);
    Ok(hex::encode(seed))
}

/// Demonstrate that the same words with a different passphrase make a different seed.
pub fn compare_passphrases(
    mnemonic: &str,
    protected_passphrase: &str,
) -> LabResult<PassphraseComparison> {
    let empty_passphrase_seed_hex = mnemonic_seed_hex(mnemonic, "")?;
    let protected_seed_hex = mnemonic_seed_hex(mnemonic, protected_passphrase)?;
    let seeds_differ = empty_passphrase_seed_hex != protected_seed_hex;

    Ok(PassphraseComparison {
        empty_passphrase_seed_hex,
        protected_seed_hex,
        seeds_differ,
    })
}

/// Recognize the public BIP39 test mnemonic used in the class labs.
pub fn is_public_test_mnemonic(mnemonic: &str) -> bool {
    let normalized = mnemonic.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized
        == "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
}
