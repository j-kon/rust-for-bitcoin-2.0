# Bitcoin Transaction Serializer (`serializeTrx`)

A command-line tool written in Rust to construct and serialize raw Bitcoin transactions from command-line arguments without external Bitcoin transaction libraries.

---

## Features

- **CLI-Driven Configuration**: Specify transaction version, SegWit status, inputs, outputs, witness data, and locktime entirely via CLI arguments.
- **Support for SegWit & Legacy**: Handles both traditional legacy transactions and SegWit transactions with witness stacks and 0x00 0x01 markers/flags.
- **Multiple Inputs & Outputs**: Support arbitrary numbers of inputs (`--input`), outputs (`--output`), and witness items (`--witness`).
- **Prevout TXID Byte-Order Handling**: Automatically converts human-readable 64-hex-character TXIDs (Big-Endian display order) into reversed 32-byte wire format (Little-Endian).
- **CompactSize Encoding**: Proper VarInt encoding supporting 1-byte, 3-byte (`0xfd`), 5-byte (`0xfe`), and 9-byte (`0xff`) length encodings.
- **Robust Error Validation**: Result-based error handling rejecting odd-length hex strings, non-hex characters, malformed CLI formats, missing inputs/outputs, and out-of-range witness indexes.

---

## Build & Run

Ensure you have Rust and Cargo installed:

```bash
cargo build --release
```

Run using `cargo run`:

```bash
cargo run -- [OPTIONS]
```

---

## CLI Syntax & Arguments

| Argument | Short | Description | Format / Example |
| :--- | :--- | :--- | :--- |
| `--version` | `-v` | Transaction version (default: `2`) | `--version 2` |
| `--segwit` | `-s` | Flag indicating a SegWit transaction | `--segwit` |
| `--input` | `-i` | Input specification (can be repeated) | `<prev_txid>:<vout>:<script_sig_hex>:<sequence>` |
| `--output` | `-o` | Output specification (can be repeated) | `<value_sats>:<script_pubkey_hex>` |
| `--witness` | `-w` | Witness item (can be repeated) | `<input_index>:<item_hex>` |
| `--locktime` | `-l` | Transaction locktime (default: `0`) | `--locktime 0` |

---

## Format Specifications

### Input Format (`--input` / `-i`)
Format: `<prev_txid>:<vout>:<script_sig_hex>:<sequence>`
- `prev_txid`: 64-hex-character display-order previous transaction ID.
- `vout`: 0-based output index (`u32`).
- `script_sig_hex`: ScriptSig bytes in hex (leave empty for native SegWit inputs).
- `sequence`: Sequence number (`u32` decimal or `0xffffffff` hex).

### Output Format (`--output` / `-o`)
Format: `<value_sats>:<script_pubkey_hex>`
- `value_sats`: Satoshi amount (`u64`).
- `script_pubkey_hex`: ScriptPubKey bytes in hex.

### Witness Format (`--witness` / `-w`)
Format: `<input_index>:<item_hex>`
- `input_index`: 0-based input index this witness item belongs to.
- `item_hex`: Witness item payload in hex.

---

## Note on TXID Byte Order

Bitcoin transaction inputs store previous transaction IDs on the wire in reversed byte order (Little-Endian).
When passing `--input`, enter the **normal displayed TXID** (as shown on block explorers). The tool automatically converts it into Little-Endian wire byte order prior to serialization.

---

## Examples

### 1. Legacy Transaction Example

```bash
cargo run -- \
  --version 1 \
  --input "21c80d2b05c1106360a36e435ad8e418e85d0eb708d9f2bf216476b37bd0b08f:0:473044:ffffffff" \
  --output "100000:76a91488ac" \
  --locktime 0
```

Output:
```text
Serialized transaction:
01000000018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210000000003473044ffffffff01a0860100000000000576a91488ac00000000

Transaction size: 68 bytes
```

---

### 2. SegWit Starter Transaction Example

```bash
cargo run -- \
  --version 2 \
  --segwit \
  --input "21c80d2b05c1106360a36e435ad8e418e85d0eb708d9f2bf216476b37bd0b08f:1::ffffffff" \
  --output "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b" \
  --output "29442:00149831122b93d21715c70db626ccc844d3c21f9687" \
  --witness "0:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301" \
  --witness "0:029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358" \
  --locktime 0
```

Output:
```text
Serialized transaction:
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```
