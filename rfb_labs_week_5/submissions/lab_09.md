# Lab 09 — BIP44 path decoding and address derivation

## Commands used

```bash
cargo test --test lab_09 -- --nocapture
```

## Terminal output

```text
running 4 tests
test decodes_every_bip44_level ... ok
test explains_zero_based_account_and_chain ... ok
test changes_only_the_final_index ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

BIP44 path decoding details verified (`m/44'/0'/2'/1/5`):
- `purpose`: `44'` (BIP44 standard multi-account hierarchy)
- `coin_type`: `0'` (Bitcoin mainnet; `1'` for testnet/regtest)
- `account`: `2'` (Third zero-based account: 0=1st, 1=2nd, 2=3rd)
- `change`: `1` (Internal/change branch; 0=external/receive)
- `index`: `5` (Sixth address position: 0=1st, ..., 5=6th)

## Evidence references

- Source implementation: [`src/labs/lab09_bip44.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab09_bip44.rs)
- Test suite: [`tests/lab_09.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_09.rs)
- `decode_bip44_path`, `describe_bip44_path`, and `derive_bip44_address` functions.

## Explanation

The BIP44 derivation path notation (`m/purpose'/coin_type'/account'/change/address_index`) enforces a standard organizational structure for multi-account Bitcoin HD wallets:

1. **Hardened Notation (`'`)**:
   - The apostrophe (`'`) or `h` signifies **hardened derivation** ($i \ge 2^{31}$).
   - Hardened derivation uses the parent private key in the HMAC input, isolating accounts and coin types so that a compromised child key/xpub on one branch cannot expose the parent master key or other accounts.

2. **Zero-Based Indexing**:
   - Account and address indexes are zero-based.
   - `account 2'` specifies the **third account** (0 = first account, 1 = second account, 2 = third account).
   - `index 5` specifies the **sixth address position** in sequence (0 = 1st, 1 = 2nd, 2 = 3rd, 3 = 4th, 4 = 5th, 5 = 6th).

3. **External vs. Internal Branching**:
   - `change = 0` represents the **external / receive branch** used for generating new addresses presented to outside payers.
   - `change = 1` represents the **internal / change branch** used by the wallet to generate change addresses for unspent transaction outputs returned to the wallet owner.
