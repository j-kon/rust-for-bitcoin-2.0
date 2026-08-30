# Lab 01 — Address and network identification

## Commands used

```bash
cargo test --test lab_01 -- --nocapture
```

## Terminal output

```text
running 4 tests
test maps_regtest_prefixes ... ok
test identifies_human_readable_prefixes ... ok
test rejects_an_address_for_the_wrong_network ... ok
test inspects_a_network_checked_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Sample inspection outputs verified:
- Mainnet P2PKH: `1BoatSLRHtKNngkdXEeobR76b53LETtpyT` -> Format `P2pkh`, Prefix `1`
- Mainnet P2SH: `3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy` -> Format `P2sh`, Prefix `3`
- Mainnet P2WPKH: `bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080` -> Format `P2wpkh`, Prefix `bc1q`
- Mainnet P2TR: `bc1pexample` -> Format `P2tr`, Prefix `bc1p`
- Regtest P2PKH: `m/n` prefix family, verified with network checking against `Network::Regtest` returning scriptPubKey `76a914c4...88ac`.

## Evidence references

- Source implementation: [`src/labs/lab01_addresses.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab01_addresses.rs)
- Test suite: [`tests/lab_01.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_01.rs)
- `inspect_address` enforcing `require_network` and extracting `script_pubkey` via `rust-bitcoin`.

## Explanation

Prefix inspection alone is insufficient for complete Bitcoin address validation for several key reasons:

1. **Prefix is only a visual classification clue**: A string can easily begin with a valid prefix character (such as `1`, `3`, or `bc1q`) while containing typographical errors or completely invalid data in the remaining characters.
2. **Checksum validation is required**: Both Base58Check (used by P2PKH and P2SH) and Bech32/Bech32m (used by SegWit and Taproot) append cryptographic checksums to detect transposition errors or mistyped characters. Simple prefix checking bypasses checksum verification.
3. **Network safety enforcement**: Address prefixes overlap across networks (e.g., testnet and regtest share `m`/`n` prefixes for P2PKH and `2` for P2SH). Full address parsing validates that the address matches the node's configured target network, preventing cross-network fund loss.
4. **Payload and scriptPubKey parsing**: Full parsing decodes the underlying 20-byte or 32-byte payload and constructs the authoritative `scriptPubKey` locking script, ensuring the address represents a valid, spendable Bitcoin script program.
