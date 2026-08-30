# Lab 07 — BIP39 mnemonics, seeds, and passphrases

## Commands used

```bash
cargo test --test lab_07 -- --nocapture
```

## Terminal output

```text
running 4 tests
test validates_entropy_and_checksum_structure ... ok
test rejects_an_invalid_checksum ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

BIP39 derivation details verified:
- Public test mnemonic: `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`
- Structure: 12 words, 128 bits entropy, 4 bits SHA256 checksum
- Passphrase `"TREZOR"` 512-bit seed: `c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04`
- Passphrase comparison: Empty vs `"class"` passphrases yield completely distinct seeds.

## Evidence references

- Source implementation: [`src/labs/lab07_bip39.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab07_bip39.rs)
- Test suite: [`tests/lab_07.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_07.rs)
- Mnemonic validation via `bip39::Mnemonic` and HMAC-SHA512 seed derivation (`to_seed`).

## Explanation

The security properties of BIP39 mnemonics, checksums, and optional passphrases involve critical cryptographic mechanisms:

1. **Mnemonic Checksum vs. Encryption**:
   - The final bits of a BIP39 mnemonic (e.g. 4 bits for a 12-word phrase) are an unencrypted SHA256 checksum of the raw initial entropy.
   - The checksum exists purely for **error detection** (detecting mistyped words or invalid word order). It provides zero secrecy or encryption. Anyone who sees the mnemonic words can read the underlying entropy.

2. **Optional BIP39 Passphrase Contribution**:
   - BIP39 derives the 512-bit binary seed using PBKDF2-HMAC-SHA512 with 2,048 iterations. The pseudorandom function uses the mnemonic string as the password and `concat("mnemonic", passphrase)` as the salt.
   - The passphrase is an integral input to the key derivation function, acting as an extra factor ("13th word" or "25th word").

3. **No "Wrong Password" Indicator and Unrecoverable Passphrase**:
   - Because PBKDF2 accepts any arbitrary salt string, **every passphrase string produces a mathematically valid 512-bit seed**.
   - There is no error code or invalid password check in BIP39 algorithmically; entering an incorrect passphrase simply derives a completely different, valid wallet hierarchy containing zero transaction history.
   - If a user forgets their BIP39 passphrase, it **cannot be reconstructed from the mnemonic alone**, as the PBKDF2 hash function is one-way and uninvertible.
