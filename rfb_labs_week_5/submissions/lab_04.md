# Lab 04 — Native P2WPKH witness programs

## Commands used

```bash
cargo test --test lab_04 -- --nocapture
```

## Terminal output

```text
running 4 tests
test leaves_scriptsig_empty_and_uses_witness ... ok
test derives_a_native_regtest_address ... ok
test reports_a_twenty_byte_program ... ok
test builds_a_version_zero_witness_lock ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Native SegWit output details verified:
- Regtest Address prefix: `bcrt1q...`
- `scriptPubKey`: `OP_0 <20-byte pubKeyHash>` (`0014...`)
- `ScriptSig`: empty (`""`)
- `Witness`: `[signature, public_key]`

## Evidence references

- Source implementation: [`src/labs/lab04_p2wpkh.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab04_p2wpkh.rs)
- Test suite: [`tests/lab_04.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_04.rs)
- Derived native P2WPKH regtest address and version-0 20-byte witness program report.

## Explanation

Native P2WPKH (BIP141/BIP173) differs fundamentally from both legacy P2PKH and P2SH-wrapped SegWit (P2SH-P2WPKH) in its script structure, unlocking location, and wire format:

1. **Comparison with Legacy P2PKH**:
   - **scriptPubKey**: Legacy P2PKH uses `OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG` (25 bytes), whereas native P2WPKH uses a version-0 witness program `OP_0 <20-byte pubKeyHash>` (22 bytes).
   - **Unlocking Location**: Legacy P2PKH places the ECDSA signature and public key inside `ScriptSig` (in the core tx transaction payload, contributing directly to size at 1x weight). Native P2WPKH leaves `ScriptSig` completely empty (`""`) and places signature and public key in the separate `witness` stack (benefiting from the SegWit 75% weight discount).
   - **Encoding**: P2PKH uses Base58Check (`1...` or `m/n...`), whereas native P2WPKH uses Bech32 (`bc1q...` or `bcrt1q...`).

2. **Comparison with P2SH-wrapped SegWit (P2SH-P2WPKH)**:
   - **scriptPubKey**: P2SH-wrapped SegWit uses an outer P2SH locking script `OP_HASH160 <20-byte scriptHash> OP_EQUAL` (`a914...`), making it look like a legacy script to older nodes. Native P2WPKH uses the native `OP_0 <20-byte pubKeyHash>` scriptPubKey directly.
   - **ScriptSig**: P2SH-wrapped SegWit requires a non-empty `ScriptSig` containing a 22-byte push of the redeemScript (`0014<pubKeyHash>`) so legacy nodes can validate the outer P2SH hash. Native P2WPKH requires no `ScriptSig` at all.
   - **Witness**: Both wrapped and native P2WPKH place the actual unlocking data (`signature` and `public_key`) in the witness array, but native P2WPKH avoids the 24-byte `ScriptSig` overhead required for backward compatibility.
