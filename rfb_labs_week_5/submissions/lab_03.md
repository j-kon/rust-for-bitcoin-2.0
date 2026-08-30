# Lab 03 — P2SH 2-of-3 multisig and redeemScripts

## Commands used

```bash
cargo test --test lab_03 -- --nocapture
```

## Terminal output

```text
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Layered P2SH verification details:
- Outer `scriptPubKey`: `OP_HASH160 <20-byte scriptHash> OP_EQUAL` (`a914...87`)
- Inner `redeemScript`: `2 <pubKey1> <pubKey2> <pubKey3> 3 OP_CHECKMULTISIG`
- Regtest Address prefix: `2` (e.g. `2N...`)

## Evidence references

- Source implementation: [`src/labs/lab03_p2sh.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab03_p2sh.rs)
- Test suite: [`tests/lab_03.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_03.rs)
- Constructed 2-of-3 multisig redeemScript and outer `Address::p2sh` commitment.

## Explanation

P2SH (BIP16) separates script commitment from script execution across two distinct validation layers:

1. **Outer Layer Commitment Matching**:
   - The UTXO locking script (`OP_HASH160 <scriptHash> OP_EQUAL`) commits only to the HASH160 hash of the serialized `redeemScript`.
   - When spending, the input's `ScriptSig` pushes the full unhashed `redeemScript` alongside any required unlocking arguments. The Bitcoin interpreter first hashes the provided `redeemScript` and executes `OP_EQUAL`.
   - Matching the P2SH hash proves conclusively that the provided `redeemScript` is the exact script committed to by the UTXO creator.

2. **Inner Layer Execution (Multisig Rule)**:
   - Matching the hash alone is necessary but **not sufficient** to authorize spending.
   - Once `OP_EQUAL` succeeds, the interpreter deserializes and executes the inner `redeemScript` (`2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`) against the remaining items in the stack (an extra dummy item `OP_0` due to the historic CHECKMULTISIG bug, plus 2 valid ECDSA signatures).
   - The transaction is valid only if at least 2 out of the 3 specified public keys have valid signatures present, satisfying the inner multisig spending policy.
