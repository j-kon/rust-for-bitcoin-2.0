# Lab 05 — Legacy, wrapped SegWit, native SegWit, and Taproot compatibility

## Commands used

```bash
cargo test --test lab_05 -- --nocapture
```

## Terminal output

```text
running 4 tests
test builds_the_four_format_map ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test selects_the_most_modern_supported_format ... ok
test names_the_required_human_encoding ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Compatibility mapping verified across address generations:
- Base58Check (`1...`, `3...`): P2PKH (`Base58Check`), P2SH / Wrapped SegWit (`Base58Check`)
- Bech32 (`bc1q...`): Native P2WPKH (`Bech32`)
- Bech32m (`bc1p...`): Taproot (`Bech32m`)

## Evidence references

- Source implementation: [`src/labs/lab05_compatibility.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab05_compatibility.rs)
- Test suite: [`tests/lab_05.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_05.rs)
- Modeled `SenderCapabilities` matrix and string encoding requirements.

## Explanation

Address compatibility across Bitcoin protocol upgrades involves distinct encoding mechanisms and operational roles:

1. **Why an Older Wallet Accepts `3...` (P2SH) but Rejects `bc1q...` (Bech32)**:
   - P2SH-wrapped SegWit (P2SH-P2WPKH) was explicitly designed for backward compatibility using Base58Check encoding with a `3...` prefix (mainnet). Older, pre-SegWit software already knew how to parse Base58Check strings and construct outer P2SH `scriptPubKey` locks (`OP_HASH160 <hash> OP_EQUAL`). To an older wallet, sending to a wrapped SegWit address looks identical to sending to a standard P2SH multisig address.
   - Native SegWit (P2WPKH) uses Bech32 encoding (`bc1q...`). Older wallets created before BIP173 lack Bech32 string parsers and checksum algorithms; when a user inputs a `bc1q...` address into an older wallet, the parser rejects it as an invalid or unrecognized string format.

2. **Sending Capability vs. Spending Capability**:
   - **Sending Support** depends entirely on the sending software's ability to **parse the recipient's human-readable address string** and construct the matching `scriptPubKey` locking script. Any node can pay a native SegWit output if its UI/wallet software understands Bech32 addresses.
   - **Spending Support** requires the spending wallet's consensus rules and transaction builder to understand how to **construct valid unlocking data** (`ScriptSig` or `witness` vector) and compute signature hashes according to the target output's rules (e.g., BIP141 SegWit script validation, BIP341 Taproot Schnorr/Tapscript verification).
   - Thus, a wallet might support spending its own SegWit UTXOs while sending to legacy addresses, or vice-versa, because sending is an address-parsing feature whereas spending is a consensus validation feature.
