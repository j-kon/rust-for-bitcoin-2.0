# Lab 10 — Deterministic recovery across BIP44, BIP49, and BIP84

## Commands used

```bash
cargo test --test lab_10 -- --nocapture
```

## Terminal output

```text
running 4 tests
test identical_recovery_inputs_repeat ... ok
test format_selection_changes_the_lock_target ... ok
test changing_only_the_index_changes_the_address ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

Regtest deterministic recovery results verified at `account 0`, `index 0`:
- BIP44 (`m/44'/1'/0'/0/0`, P2PKH): Legacy Base58Check address (`m...` / `n...`)
- BIP49 (`m/49'/1'/0'/0/0`, P2SH-P2WPKH): Wrapped SegWit Base58Check address (`2...`)
- BIP84 (`m/84'/1'/0'/0/0`, P2WPKH): Native SegWit Bech32 address (`bcrt1q...`)
- Repeatability: Identical inputs produce identical addresses; incrementing index to `1` produces a distinct address.

## Evidence references

- Source implementation: [`src/labs/lab10_recovery.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab10_recovery.rs)
- Test suite: [`tests/lab_10.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_10.rs)
- Deterministic derivation functions `derive_address_set`, `recovery_is_repeatable`, and `changing_index_changes_address`.

## Explanation

The mechanics of deterministic wallet recovery demonstrate both the power and constraints of BIP32/BIP39 HD hierarchies:

1. **Determinism from Recovery Inputs**:
   - Because cryptographic functions (PBKDF2-HMAC-SHA512 seed derivation and secp256k1 HMAC-SHA512 key child derivation) are strictly deterministic, supplying the exact same inputs (mnemonic words, passphrase, derivation path, network, and script convention) will **always generate the exact same set of private keys, public keys, and Bitcoin addresses**.

2. **Why a Mnemonic Alone Is Not Sufficient**:
   - Holding a 12-word or 24-word BIP39 mnemonic is necessary but **not sufficient** on its own to reconstruct a user's wallet state or history.
   - Complete wallet recovery requires knowledge of five distinct parameters:
     a. **Mnemonic Words**: The raw 128..256 bits of initial entropy.
     b. **BIP39 Passphrase**: An optional salt string; if omitted or mistyped, an entirely different wallet tree is derived.
     c. **Derivation Path Standards**: Different standards (BIP44 for legacy `m/44'`, BIP49 for nested SegWit `m/49'`, BIP84 for native SegWit `m/84'`, BIP86 for Taproot `m/86'`) route the master seed to different key subtrees.
     d. **Network Context**: Deriving for Mainnet (`coin_type 0'`) vs Testnet/Regtest (`coin_type 1'`) yields different child keys and address prefixes.
     e. **Script / Address Convention**: Even given the exact same public key, interpreting it as P2PKH (`1...`), P2SH-P2WPKH (`3...`), or P2WPKH (`bc1q...`) produces completely different address strings and on-chain scriptPubKeys.
