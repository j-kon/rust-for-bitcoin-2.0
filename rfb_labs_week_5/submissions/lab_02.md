# Lab 02 — Legacy P2PKH construction and ScriptSig

## Commands used

```bash
cargo test --test lab_02 -- --nocapture
```

## Terminal output

```text
running 4 tests
test commits_to_hash160_of_the_public_key ... ok
test derives_the_expected_p2pkh_address ... ok
test builds_the_standard_p2pkh_lock ... ok
test puts_unlocking_data_in_scriptsig ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

P2PKH output details verified:
- `scriptPubKey`: `OP_DUP OP_HASH160 <20-byte pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG` (`76a914...88ac`)
- `ScriptSig`: push signature, push public key
- `Witness`: empty (`[]`)

## Evidence references

- Source implementation: [`src/labs/lab02_p2pkh.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab02_p2pkh.rs)
- Test suite: [`tests/lab_02.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_02.rs)
- Derived P2PKH address and HASH160 public key hash commitment using `Address::p2pkh`.

## Explanation

The distinction between **key identity / pubKeyHash commitment** and **spend authorization** is central to Bitcoin's P2PKH locking and unlocking model:

1. **Key Identity / Hash Commitment (`scriptPubKey`)**:
   - The UTXO locking script (`OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG`) commits only to the 20-byte `HASH160` (RIPEMD160 of SHA256) of the recipient's public key.
   - At UTXO creation time, neither the network nor the transaction scriptPubKey contains the full unhashed public key or a signature. The 20-byte hash identifies who is allowed to spend the output without revealing the public key on-chain until spend time.

2. **Spend Authorization (`ScriptSig`)**:
   - Spending the P2PKH output requires the spender to provide two items in the input's `ScriptSig`: the full `public_key` and a valid ECDSA `signature`.
   - Execution occurs in two phases:
     a. **Identity Verification**: `OP_DUP OP_HASH160` hashes the provided public key and `OP_EQUALVERIFY` asserts it matches the 20-byte `<pubKeyHash>` committed in the `scriptPubKey`.
     b. **Authorization Proving**: `OP_CHECKSIG` verifies that the provided ECDSA signature is valid for the transaction under the revealed `public_key`, cryptographically proving ownership of the corresponding private key.
