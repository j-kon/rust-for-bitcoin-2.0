# Assignment: Building a Bitcoin Wallet in Rust

## Goal

Build a functioning Bitcoin wallet in Rust (regtest) that demonstrates you can use the libraries covered in class effectively.

## Minimum Requirements

Your wallet must be able to:

1. **Generate or import keys** and derive a wallet from a descriptor.
2. **Generate addresses** from both an external (receiving) and internal (change) keychain.
3. **Track UTXOs and calculate balance** for the wallet.
4. **Persist wallet state locally** (e.g. with SQLite) so the wallet can be closed and reopened without losing track of its own state.
5. **Construct, sign, and broadcast a transaction** on testnet / regtest.
6. **Connect to a Bitcoin node** (via `bitcoincore-rpc`) to sync wallet state or broadcast transactions, i.e., your wallet should not be purely offline.

---

# Week 6 Bitcoin Wallet

A functional descriptor-based Regtest CLI wallet with real Bitcoin Core integration, built using the modern Rust Bitcoin ecosystem (`bdk_wallet`, `bdk_bitcoind_rpc`, `bitcoincore-rpc`, `rust-bitcoin`, and SQLite).

## Features

- **BIP84 SegWit (P2WPKH) Descriptors**: Native descriptor derivation separating receiving (`m/84'/1'/0'/0/*`) and change (`m/84'/1'/0'/1/*`) keychains.
- **Cryptographically Secure Key Generation**: Disposable test keys generated using `rand::rngs::OsRng` and BIP39 mnemonics.
- **Robust SQLite Persistence**: Persists keychain indices, local chain checkpoints, UTXOs, and wallet descriptors into a local SQLite database (`./data/wallet.sqlite`). Survives application restarts seamlessly.
- **Bitcoin Core RPC Integration**: Connects to bitcoind for chain state verification, block height tracking, and raw transaction broadcasting.
- **Blockchain Synchronization**: Emits new blocks and mempool events from Bitcoin Core using `bdk_bitcoind_rpc::Emitter` directly into `bdk_wallet`.
- **Accurate Balance Reporting**: Integer satoshi accounting reporting confirmed, trusted pending, untrusted pending, immature, and total balances.
- **UTXO Tracking**: Inspects individual unspent transaction outputs with outpoints (`txid:vout`), amounts, keychains, derivation indices, and confirmation block heights.
- **Transaction Construction & Signing**: Automatic coin selection, change output routing to the internal keychain, configurable fee rates, full PSBT signing, and final transaction extraction.
- **Strict Regtest Safety**: Enforces network validation to protect against accidental mainnet usage.

---

## Architecture

```
                               +-----------------------------------+
                               |             CLI Layer             |
                               |    (clap / src/cli.rs & main.rs)  |
                               +-----------------+-----------------+
                                                 |
                                                 v
                               +-----------------------------------+
                               |          Commands Layer           |
                               |         (src/commands.rs)         |
                               +--------+-----------------+--------+
                                        |                 |
                   +--------------------+                 +--------------------+
                   |                                                           |
                   v                                                           v
+------------------------------------+                       +------------------------------------+
|            Wallet Engine           |                       |            Node Backend            |
|          (src/wallet.rs)           |                       |           (src/node.rs)            |
|                                    |                       |                                    |
| - bdk_wallet (v1.2.0)              |                       | - bitcoincore-rpc (v0.19.0)        |
| - BIP84 descriptor management      |                       | - bdk_bitcoind_rpc (v0.18.0)       |
| - Coin selection & TxBuilder       |                       | - RPC connectivity & node-info     |
| - PSBT signing & extraction        |                       | - Block & mempool emitter sync     |
+------------------+-----------------+                       +-----------------+------------------+
                   |                                                           |
                   v                                                           v
+------------------------------------+                       +------------------------------------+
|         SQLite Persistence         |                       |            Bitcoin Core            |
|     (rusqlite / bdk_chain)         |                       |           (bitcoind node)          |
|                                    |                       |                                    |
| - bdk_wallet tables (UTXO, chain)  |                       | - Blocks & chain tip               |
| - _wallet_secrets table            |                       | - Mempool                          |
| - data/wallet.sqlite (gitignored)  |                       | - sendrawtransaction endpoint      |
+------------------------------------+                       +------------------------------------+
```

### Library Roles and Rationales

1. **`bdk_wallet` (v1.2.0)**:
   - *Role*: Core descriptor wallet engine, address derivation, keychain index management, UTXO tracking, coin selection, PSBT construction, and transaction signing.
   - *Why*: Writing safe, standard-compliant descriptor logic, script verification, coin selection, and PSBT signing by hand is extremely error-prone. BDK provides a battle-tested, modular framework specifically designed for descriptor-first wallets.
2. **`bdk_bitcoind_rpc` (v0.18.0)**:
   - *Role*: Blockchain data emitter connecting `bitcoincore-rpc` to `bdk_wallet`.
   - *Why*: Emits blocks and mempool transactions directly into BDK's `LocalChain` and `TxGraph` without requiring Bitcoin Core's built-in wallet. This ensures the wallet state remains decoupled from Bitcoin Core's internal wallet.
3. **`bitcoincore-rpc` (v0.19.0)**:
   - *Role*: Direct JSON-RPC communication with the Bitcoin Core daemon.
   - *Why*: Standard Rust client for Bitcoin Core. Used for verifying node connectivity, querying blockchain statistics, and broadcasting raw signed transactions via `sendrawtransaction`.
4. **`bitcoin` / rust-bitcoin (v0.32.102)**:
   - *Role*: Core data primitives (`Amount`, `Address`, `Network`, `Transaction`, `TxIn`, `TxOut`, `FeeRate`, `OutPoint`, `Witness`).
   - *Why*: The canonical foundation of the Rust Bitcoin ecosystem, guaranteeing exact wire format compliance and cryptographic correctness.
5. **`rusqlite` (v0.31.0 via `bdk_wallet`)**:
   - *Role*: Local state persistence.
   - *Why*: Provides robust, transactional storage on disk. BDK's native `rusqlite` feature enables atomic changesets for UTXOs and chain state, while an isolated internal table (`_wallet_secrets`) securely persists private descriptors locally without polluting Git.

---

## Requirements

- **Rust**: 1.85.0+ (Edition 2024 / 2021 compatible; compiled with Rust 1.95.0)
- **Bitcoin Core**: `bitcoind` / `bitcoin-cli` (v24.0+) running in regtest mode
- **Operating System**: macOS or Linux

---

## Bitcoin Core Regtest Setup

You can run Bitcoin Core locally either natively or via Docker.

### Option A: Native `bitcoind`

Create a local configuration file (e.g. `~/.bitcoin/bitcoin.conf`):
```ini
regtest=1
server=1
txindex=1
rpcuser=regtest_user
rpcpassword=regtest_password
rpcport=18443
rpcallowip=127.0.0.1
[regtest]
rpcbind=127.0.0.1
```

Start the daemon:
```bash
bitcoind -regtest -daemon
```

Verify connection:
```bash
bitcoin-cli -regtest -rpcuser=regtest_user -rpcpassword=regtest_password getblockchaininfo
```

### Option B: Docker

```bash
docker run -d --name bitcoin-regtest \
  -p 18443:18443 \
  -e BITCOIN_EXTRA_ARGS="-regtest=1 -server=1 -txindex=1 -rpcuser=regtest_user -rpcpassword=regtest_password -rpcallowip=0.0.0.0/0 -rpcbind=0.0.0.0" \
  ruimarinho/bitcoin-core:26
```

---

## Environment Configuration

Copy `.env.example` to `.env` (note that `.env` is gitignored):
```bash
cp .env.example .env
```

`.env` configuration parameters:
```bash
BITCOIN_RPC_URL=http://127.0.0.1:18443
BITCOIN_RPC_USER=regtest_user
BITCOIN_RPC_PASSWORD=regtest_password
WALLET_DB_PATH=./data/wallet.sqlite
BITCOIN_NETWORK=regtest
```

---

## Build

Compile the project and run all quality checks:
```bash
cargo build
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

---

## CLI Usage Walkthrough (Illustrative Examples)

> [!NOTE]
> The outputs in this section illustrate general command syntax and sample schema formatting.
> For full, actual evidence with genuine transaction IDs from an active Regtest node, see the [Verified Live Regtest Proof](#verified-live-regtest-proof) section below.

### 1. Initialize Wallet
Generates fresh disposable key material, constructs BIP84 descriptors, initializes the database, and persists the wallet state:
```bash
cargo run -- init
```
*Output:*
```
=== Wallet Initialized Successfully ===
Database Location:           ./data/wallet.sqlite
Network:                     regtest
External Public Descriptor:  wpkh([5c.../84'/1'/0']tpub.../0/*)#...
Internal Public Descriptor:  wpkh([5c.../84'/1'/0']tpub.../1/*)#...

Run `cargo run -- new-address` to get your first receiving address.
```

### 2. Wallet Information
Reopens the existing wallet and prints public metadata (without exposing secret keys):
```bash
cargo run -- info
```
*Output:*
```
=== Wallet Information ===
Database:                    ./data/wallet.sqlite
Network:                     regtest
Next External Address Index: 0
Next Internal Address Index: 0
Local Chain Tip Height:      0
Local Chain Tip Hash:        0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206
External Public Descriptor:  wpkh([5c.../84'/1'/0']tpub.../0/*)#...
Internal Public Descriptor:  wpkh([5c.../84'/1'/0']tpub.../1/*)#...
```

### 3. Generate External Receiving Address
Generates and reveals the next address on the receiving keychain (`m/84'/1'/0'/0/*`):
```bash
cargo run -- new-address
```
*Output:*
```
=== New External Receiving Address ===
Address:   bcrt1q6w8n7f3w7t2wvx4s2zg9x6r2zfk3r5p2qlw8t3
Keychain:  External
Index:     0
Network:   regtest
```

### 4. Generate Internal Change Address
Generates and reveals the next address on the internal change keychain (`m/84'/1'/0'/1/*`):
```bash
cargo run -- new-change-address
```
*Output:*
```
=== New Internal Change Address ===
Address:   bcrt1q8d3p4l5k7j6h5g4f3d2s1a0z9y8x7w6v5u4t3
Keychain:  Internal
Index:     0
Network:   regtest
```

### 5. Check Node Connectivity
Queries the connected Bitcoin Core regtest node:
```bash
cargo run -- node-info
```
*Output:*
```
=== Bitcoin Core Node Info ===
RPC URL:                 http://127.0.0.1:18443
Chain:                   regtest
Blocks:                  101
Headers:                 101
Best Block Hash:         2e7b...
Difficulty:              0.000001
Verification Progress:   100.0000%
Initial Block Download:  false
Node Version / Client:   /Satoshi:26.0.0/
```

### 6. Synchronize Wallet
Scans new blocks and mempool transactions from Bitcoin Core:
```bash
cargo run -- sync
```
*Output:*
```
Connecting to Bitcoin Core RPC at http://127.0.0.1:18443...
=== Sync Complete ===
Blocks Scanned & Applied: 101
Mempool Txs Applied:      0
Wallet Tip Height:        0 -> 101
Best Block Hash:          2e7b...
```

### 7. Check Balance
Displays confirmed, pending, immature, and total balances in integer satoshis:
```bash
cargo run -- balance
```
*Output:*
```
=== Wallet Balance ===
Confirmed:              5000000000 sats
Trusted Pending:                 0 sats
Untrusted Pending:               0 sats
Immature:                        0 sats
--------------------------------------
Total Balance:          5000000000 sats
```

### 8. Inspect UTXOs
Lists unspent transaction outputs tracked by the wallet:
```bash
cargo run -- utxos
```
*Output:*
```
=== Wallet UTXOs (Unspent Outputs) ===
OutPoint (txid:vout)                                                 Amount (sat) Keychain   Index  Status         
-------------------------------------------------------------------------------------------------------------------
a8f9c0e2...01:0                                                        5000000000 External   0      Block 101      
```

### 9. Send Transaction
Constructs, signs, finalizes, and broadcasts a transaction:
```bash
cargo run -- send --to bcrt1q9v8t6s5r4q3p2o1n0m9l8k7j6h5g4f3d2s1a --amount 500000 --fee-rate 2
```
*Output:*
```
Constructing and signing transaction to bcrt1q9v8t6s5r4q3p2o1n0m9l8k7j6h5g4f3d2s1a for 500000 sats...
=== Transaction Broadcast Successful ===
Transaction ID (txid): c4d8e7b1a2f3...
Recipient:             bcrt1q9v8t6s5r4q3p2o1n0m9l8k7j6h5g4f3d2s1a
Amount:                500000 sats
Network Fee:           282 sats
Status:                Broadcast to Bitcoin Core node mempool
```

---

## Persistence Demonstration

To verify that the wallet survives process restarts without loss of state:
1. Initialize the wallet: `cargo run -- init`.
2. Reveal an external address: `cargo run -- new-address` (returns index `0`).
3. Reveal a change address: `cargo run -- new-change-address` (returns index `0`).
4. Terminate the application completely.
5. Reopen the wallet: `cargo run -- info`. Notice the next external index is `1` and internal is `1`.
6. Reveal another address: `cargo run -- new-address` (returns index `1` with a new, distinct address).
7. Chain state, checkpoint heights, and UTXO history remain intact in SQLite without re-deriving or resetting.

Automated integration test verifying this exact lifecycle:
```bash
cargo test --test persistence_tests
```

---

## Verified Live Regtest Proof

> [!IMPORTANT]
> **Verified Live Regtest Execution Evidence**
> The following proof was captured live from an active, isolated **Regtest** node running Bitcoin Core **v29.2.0** via Docker (`btcpayserver/bitcoin:29.2`). No real funds, mainnet, or fabricated identifiers were used.

### 1. Environment & Node State
- **Network**: `regtest` (strictly enforced, isolated local environment)
- **Bitcoin Core Version**: `/Satoshi:29.2.0/` (RPC port: `18443`)
- **Initial Block Height**: 0

### 2. Receiving Address & Initial Funding Workflow
- **Wallet Initialization**: Fresh wallet initialized into `./data/wallet.sqlite`.
- **First External Receiving Address** (index 0): `bcrt1qn7va4jyft5sntpfxrugvud69ymz9pslnvgf3f6`
- **First Internal Change Address** (index 0): `bcrt1qgcnvly7kgc0aldx3z82q9ezntu7359tmdrpmsz`
- **Coinbase Funding**: Mined 101 blocks directly to `bcrt1qn7va4jyft5sntpfxrugvud69ymz9pslnvgf3f6` using Bitcoin Core as a faucet.
- **Initial Sync Result**:
  - Blocks Scanned & Applied: 101
  - Wallet Tip Height: 0 -> 101
  - Best Block Hash: `3bf17202acc7cf05c96e38f8a07eee70129d8971b492817c9c6967eaf44a0688`

### 3. Wallet Balance & UTXO State Before Spend
- **Confirmed Balance**: `10000000000` sats (100 BTC from Blocks 1 and 2 reaching 100-block coinbase maturity)
- **Immature Balance**: `495000000000` sats (99 coinbase outputs awaiting maturity)
- **Total Balance**: `505000000000` sats
- **Mature UTXO Selected for Spend**:
  - OutPoint: `a75f55c0813d004a8e21191a455019c94b8003c81660a21ad66532c0c2c38adf:0`
  - Value: 5,000,000,000 sats (Block 2 coinbase)

### 4. Transaction Construction, Signing, and Broadcast
- **Destination Address**: `bcrt1q2rc5005xpgrsmx8yjzm6ahed7nvkmnuw993npm` (independent address from Bitcoin Core faucet wallet `rfb_faucet`)
- **Amount Sent**: `100000` sats
- **Fee Rate Requested**: `2` sat/vB
- **Actual Transaction Size**: 222 bytes (virtual size: `141` vB)
- **Actual Network Fee**: `281` sats (effective rate: ~1.99 sat/vB)
- **Change Output**: `4999899719` sats routed automatically to internal change address `bcrt1qgcnvly7kgc0aldx3z82q9ezntu7359tmdrpmsz`
- **ACTUAL FULL TXID**:
  ```text
  66bead0ad58746eb3005c7a5e9ab3f305b8f2ae630d5f189ee32402ac94b0c01
  ```

### 5. Mempool & Block Confirmation Verification
- **Mempool Verification**:
  Querying `getrawmempool` immediately confirmed the transaction was accepted into the Bitcoin Core node's memory pool:
  ```json
  [
    "66bead0ad58746eb3005c7a5e9ab3f305b8f2ae630d5f189ee32402ac94b0c01"
  ]
  ```
- **Block Mining & Confirmation**:
  Mined 1 block to a fresh mining address (`bcrt1q304f97u66jm2r00e3r20c456yfxzvx5gl2yt9r`).
  - Confirmation Block Hash: `2f2690e61bdd2fc5cbd4ba224e363e1ad4d57c175550630c7a17919eabd6fca4` (height 102)
  - Confirmations: `1`
  - Verified via `getrawtransaction "66bead0ad58746eb3005c7a5e9ab3f305b8f2ae630d5f189ee32402ac94b0c01" true`.

### 6. Post-Confirmation Resync & Balance
Running `cargo run -- sync` updated the wallet tip to height 102:
- **Confirmed Balance**: `14999899719` sats
  - 5,000,000,000 sats (Block 1 mature coinbase)
  - 5,000,000,000 sats (Block 3 newly matured coinbase at height 102)
  - 4,999,899,719 sats (Internal change output from TXID `66bead0a...`)
- **Immature Balance**: `490000000000` sats (98 immature coinbases)
- **Total Balance**: `504999899719` sats (exact accounting: 505,000,000,000 initial - 100,000 sent - 281 fee)
- **New Confirmed UTXO**:
  `66bead0ad58746eb3005c7a5e9ab3f305b8f2ae630d5f189ee32402ac94b0c01:0` | `4999899719` sats | Keychain: `Internal` | Index: `0` | Status: `Block 102`

### 7. Restart Persistence After Live Chain Activity
Exited the Rust CLI process completely and reopened (`cargo run -- info`, `cargo run -- balance`, `cargo run -- utxos`):
- Next external address index remained advanced at `2`.
- Next internal address index remained advanced at `1`.
- Local chain tip height remained at `102` with hash `2f2690e61bdd2fc5cbd4ba224e363e1ad4d57c175550630c7a17919eabd6fca4`.
- Confirmed balance remained `14999899719` sats without requiring another blockchain synchronization.

### Step-by-Step Instructions to Reproduce Live Regtest Proof:
```bash
# 1. Initialize fresh wallet
cargo run -- init

# 2. Get receiving address
RECV_ADDR=$(cargo run -- new-address | grep "Address:" | awk '{print $2}')

# 3. Fund address using Bitcoin Core test faucet (101 blocks for coinbase maturity)
bitcoin-cli -regtest -rpcuser=rfb_regtest -rpcpassword=rfb_regtest_only generatetoaddress 101 $RECV_ADDR

# 4. Sync wallet and check balance
cargo run -- sync
cargo run -- balance
cargo run -- utxos

# 5. Generate independent destination address
DEST_ADDR=$(bitcoin-cli -regtest -rpcuser=rfb_regtest -rpcpassword=rfb_regtest_only -rpcwallet=rfb_faucet getnewaddress)

# 6. Send funds with our Rust wallet
cargo run -- send --to $DEST_ADDR --amount 100000 --fee-rate 2

# 7. Mine a block and verify confirmation
MINING_ADDR=$(bitcoin-cli -regtest -rpcuser=rfb_regtest -rpcpassword=rfb_regtest_only -rpcwallet=rfb_faucet getnewaddress)
bitcoin-cli -regtest -rpcuser=rfb_regtest -rpcpassword=rfb_regtest_only generatetoaddress 1 $MINING_ADDR
cargo run -- sync
cargo run -- balance
cargo run -- utxos
```

---

## Educational Comparison: Low-Level `rust-bitcoin` vs. BDK

We include a runnable educational example in [`examples/raw_tx_comparison.rs`](examples/raw_tx_comparison.rs).

Run the example:
```bash
cargo run --example raw_tx_comparison
```

### Key Differences:
- **`rust-bitcoin`**: Provides direct access to individual consensus data structures (`Transaction`, `TxIn`, `TxOut`, `Witness`, `SighashCache`). You must manually select UTXOs, calculate change, determine virtual size, manage sighash flags (`EcdsaSighashType::All`), sign with `secp256k1`, and push DER signatures onto the witness stack.
  - *When to use*: Custom off-chain protocols (Lightning channel state transitions, DLCs, MuSig2, ark/covenants), non-standard transaction templates, and performance-critical minimal clients.
- **BDK (`bdk_wallet`)**: Higher-level descriptor wallet architecture that abstracts away coin selection, fee estimation, change derivation, and PSBT orchestration through `TxBuilder`.
  - *When to use*: End-user wallets, automated services, and multi-keychain descriptor management where standard address tracking and coin management are needed.

---

## Known Limitations

1. **Fee Estimation**: Default feerate is fixed at 1 sat/vB unless passed via `--fee-rate`. Dynamic feerate estimation via `estimatesmartfee` is not yet hooked up.
2. **Key Storage Encryption**: Private descriptors in `./data/wallet.sqlite` (`_wallet_secrets`) are stored locally in plaintext SQLite with standard file permissions. Production wallet software should encrypt and protect signing material more strongly using robust key derivation (e.g. Argon2id) and authenticated encryption (e.g. ChaCha20-Poly1305).
3. **Single Output Sending**: Currently supports one recipient per `send` invocation. Batch transactions (multiple outputs) are not yet exposed on the CLI.

---

## Improvements with More Time

- **Encrypted Keystore**: AES-GCM or ChaCha20-Poly1305 passphrase encryption for secret keys in SQLite.
- **Taproot (`tr`) Descriptors**: Add BIP86 Taproot descriptor support and compare witness size savings with P2WPKH.
- **Coin Control & Manual Selection**: CLI flags to spend specific outpoints (`--utxo <txid:vout>`).
- **Fee Bumping (RBF / CPFP)**: Integrate `wallet.build_fee_bump` to accelerate pending transactions.
- **Hardware Wallet Integration**: Connect with HWI / Ledger / Trezor via standard PSBT export/import.