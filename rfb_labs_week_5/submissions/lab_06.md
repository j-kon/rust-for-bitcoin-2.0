# Lab 06 — Transaction weight, virtual size, and fees

## Commands used

```bash
cargo test --test lab_06 -- --nocapture
```

## Terminal output

```text
running 4 tests
test calculates_bip141_weight ... ok
test calculates_fee_from_feerate ... ok
test reproduces_the_class_fee_comparison ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Class comparison results verified at 50 sat/vB:
- Legacy 1-in-2-out P2PKH (~226 vB): `226 vB * 50 sat/vB` = 11,300 satoshis fee
- Native 1-in-2-out P2WPKH (~141 vB): `141 vB * 50 sat/vB` = 7,050 satoshis fee
- Fee Savings: 4,250 satoshis (37.6% savings)

## Evidence references

- Source implementation: [`src/labs/lab06_weight_fees.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab06_weight_fees.rs)
- Test suite: [`tests/lab_06.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_06.rs)
- BIP141 weight formula `weight = stripped_size * 3 + total_size` and integer ceiling virtual size calculation.

## Explanation

The BIP141 weight and virtual size mechanism introduced in Segregated Witness applies a specific economic discount to witness data rather than discarding or flat-discounting transactions:

1. **Witness Bytes Are Still Counted**:
   - Bitcoin nodes must store, transmit, and validate witness data (signatures, public keys, scripts). Therefore, witness data is never ignored or "deleted" from transaction size calculation.
   - Under BIP141, non-witness data (header, inputs, outputs, locktime) is weighted at **4 weight units (WU) per byte**, while witness data is weighted at **1 weight unit (WU) per byte**.
   - Mathematically, `Weight = (Stripped_Size * 4) + (Witness_Size * 1)` or `(Stripped_Size * 3) + Total_Size`.

2. **Weight Units vs. Whole-Transaction Discount**:
   - The 75% discount applies **only to witness bytes**, not to the transaction as a whole.
   - Transaction inputs and outputs (e.g. `scriptPubKey`, amount, outpoint TXID) still cost full weight (4 WU/byte).
   - Because virtual size is computed as `ceil(Weight / 4)`, transactions with large witness structures (such as multisig or P2WPKH signatures) see a significant reduction in `vbytes` (e.g., P2WPKH dropping from ~226 vB to ~141 vB), directly reducing required transaction fees while ensuring miners are compensated for full block weight limits (4,000,000 WU).
