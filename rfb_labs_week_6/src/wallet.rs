use crate::config::AppConfig;
use crate::error::AppError;
use bdk_wallet::{
    bitcoin::{
        bip32::{DerivationPath, Xpriv},
        secp256k1::Secp256k1,
        Address, Amount, FeeRate, Network,
    },
    rusqlite::{named_params, Connection},
    KeychainKind, Wallet,
};
use bip39::Mnemonic;
use rand::rngs::OsRng;
use rand::RngCore;
use std::str::FromStr;

use bdk_wallet::PersistedWallet;

pub const SECRETS_TABLE_NAME: &str = "_wallet_secrets";

#[derive(Debug, Clone)]
pub struct WalletInitResult {
    pub db_path: String,
    pub network: Network,
    pub external_descriptor_public: String,
    pub internal_descriptor_public: String,
}

#[derive(Debug, Clone)]
pub struct GeneratedAddress {
    pub address: bdk_wallet::bitcoin::Address,
    pub index: u32,
    pub keychain: KeychainKind,
    pub network: Network,
}

#[derive(Debug, Clone)]
pub struct WalletSummary {
    pub db_path: String,
    pub network: Network,
    pub external_descriptor_public: String,
    pub internal_descriptor_public: String,
    pub next_external_index: u32,
    pub next_internal_index: u32,
    pub tip_height: u32,
    pub tip_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BalanceReport {
    pub confirmed_sats: u64,
    pub trusted_pending_sats: u64,
    pub untrusted_pending_sats: u64,
    pub immature_sats: u64,
    pub total_sats: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtxoEntry {
    pub outpoint: String,
    pub txid: String,
    pub vout: u32,
    pub amount_sats: u64,
    pub keychain: KeychainKind,
    pub derivation_index: u32,
    pub is_confirmed: bool,
    pub confirmation_height: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct TransactionBuildResult {
    pub tx: bdk_wallet::bitcoin::Transaction,
    pub txid: bdk_wallet::bitcoin::Txid,
    pub fee_sats: u64,
    pub recipient: bdk_wallet::bitcoin::Address,
    pub amount_sats: u64,
    pub is_finalized: bool,
}

pub struct AppWallet {
    pub wallet: PersistedWallet<Connection>,
    pub conn: Connection,
    pub external_descriptor: String,
    pub internal_descriptor: String,
    pub db_path: String,
}

impl AppWallet {
    /// Generates fresh key material, derives BIP84 descriptors, creates the BDK wallet,
    /// and persists initial state and descriptors into SQLite.
    pub fn init(config: &AppConfig) -> Result<WalletInitResult, AppError> {
        config.ensure_db_dir()?;

        if config.db_path.exists() {
            // Check if wallet already initialized
            if let Ok(conn) = Connection::open(&config.db_path) {
                let table_exists: Result<i64, _> = conn.query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [SECRETS_TABLE_NAME],
                    |row| row.get(0),
                );
                if let Ok(count) = table_exists {
                    if count > 0 {
                        return Err(AppError::WalletAlreadyInitialized(
                            config.db_path.display().to_string(),
                        ));
                    }
                }
            }
        }

        // Generate fresh 128-bit disposable entropy using OsRng
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| AppError::KeyDerivation(format!("Mnemonic generation failed: {e}")))?;

        let seed = mnemonic.to_seed("");
        let secp = Secp256k1::new();
        let master_xpriv = Xpriv::new_master(config.network, &seed)
            .map_err(|e| AppError::KeyDerivation(format!("Master key derivation failed: {e}")))?;

        // BIP84 Derivation Paths for Regtest / Testnet: m/84'/1'/0'/0 for external, m/84'/1'/0'/1 for internal
        let ext_path = DerivationPath::from_str("m/84'/1'/0'/0")
            .map_err(|e| AppError::KeyDerivation(format!("Invalid external path: {e}")))?;
        let int_path = DerivationPath::from_str("m/84'/1'/0'/1")
            .map_err(|e| AppError::KeyDerivation(format!("Invalid internal path: {e}")))?;

        let ext_xpriv = master_xpriv
            .derive_priv(&secp, &ext_path)
            .map_err(|e| AppError::KeyDerivation(format!("Failed to derive external xpriv: {e}")))?;
        let int_xpriv = master_xpriv
            .derive_priv(&secp, &int_path)
            .map_err(|e| AppError::KeyDerivation(format!("Failed to derive internal xpriv: {e}")))?;

        let external_desc = format!("wpkh({}/*)", ext_xpriv);
        let internal_desc = format!("wpkh({}/*)", int_xpriv);

        let mut conn = Connection::open(&config.db_path)
            .map_err(|e| AppError::Persistence(format!("Failed to open SQLite database: {e}")))?;

        // Initialize secrets table
        conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {SECRETS_TABLE_NAME} (
                    id INTEGER PRIMARY KEY CHECK (id = 0),
                    external_desc TEXT NOT NULL,
                    internal_desc TEXT NOT NULL,
                    network TEXT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )"
            ),
            [],
        )
        .map_err(|e| AppError::Persistence(format!("Failed to create secrets table: {e}")))?;

        // Store descriptors in SQLite
        conn.execute(
            &format!(
                "INSERT INTO {SECRETS_TABLE_NAME} (id, external_desc, internal_desc, network)
                 VALUES (0, :ext, :int, :net)
                 ON CONFLICT(id) DO UPDATE SET external_desc=:ext, internal_desc=:int, network=:net"
            ),
            named_params! {
                ":ext": &external_desc,
                ":int": &internal_desc,
                ":net": config.network.to_string(),
            },
        )
        .map_err(|e| AppError::Persistence(format!("Failed to store wallet secrets: {e}")))?;

        // Create BDK wallet
        let mut wallet = Wallet::create(external_desc.clone(), internal_desc.clone())
            .network(config.network)
            .create_wallet(&mut conn)
            .map_err(|e| AppError::Persistence(format!("Failed to initialize BDK wallet: {e}")))?;

        wallet
            .persist(&mut conn)
            .map_err(|e| AppError::Persistence(format!("Failed to persist BDK wallet: {e}")))?;

        let ext_pub = wallet.public_descriptor(KeychainKind::External).to_string();
        let int_pub = wallet.public_descriptor(KeychainKind::Internal).to_string();

        Ok(WalletInitResult {
            db_path: config.db_path.display().to_string(),
            network: config.network,
            external_descriptor_public: ext_pub,
            internal_descriptor_public: int_pub,
        })
    }

    /// Opens and loads an existing persisted wallet from SQLite.
    pub fn open(config: &AppConfig) -> Result<Self, AppError> {
        if !config.db_path.exists() {
            return Err(AppError::WalletNotInitialized);
        }

        let mut conn = Connection::open(&config.db_path)
            .map_err(|e| AppError::Persistence(format!("Failed to open SQLite database: {e}")))?;

        // Retrieve private descriptors from secrets table
        let (external_descriptor, internal_descriptor, stored_network): (String, String, String) = {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT external_desc, internal_desc, network FROM {SECRETS_TABLE_NAME} WHERE id = 0"
                ))
                .map_err(|_| AppError::WalletNotInitialized)?;

            stmt.query_row([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|_| AppError::WalletNotInitialized)?
        };

        let network = AppConfig::parse_and_validate_network(&stored_network)?;
        if network != config.network {
            return Err(AppError::NetworkMismatch {
                node_network: stored_network,
                expected_network: config.network.to_string(),
            });
        }

        let wallet_opt = Wallet::load()
            .descriptor(KeychainKind::External, Some(external_descriptor.clone()))
            .descriptor(KeychainKind::Internal, Some(internal_descriptor.clone()))
            .extract_keys()
            .check_network(config.network)
            .load_wallet(&mut conn)
            .map_err(|e| AppError::Persistence(format!("Failed to load BDK wallet: {e}")))?;

        let wallet = wallet_opt.ok_or(AppError::WalletNotInitialized)?;

        Ok(Self {
            wallet,
            conn,
            external_descriptor,
            internal_descriptor,
            db_path: config.db_path.display().to_string(),
        })
    }

    /// Persists wallet changeset to SQLite.
    pub fn persist(&mut self) -> Result<bool, AppError> {
        self.wallet
            .persist(&mut self.conn)
            .map_err(|e| AppError::Persistence(format!("Failed to persist wallet: {e}")))
    }

    /// Returns high-level public summary information about the wallet.
    pub fn get_summary(&self) -> WalletSummary {
        let cp = self.wallet.latest_checkpoint();
        let next_ext = self
            .wallet
            .derivation_index(KeychainKind::External)
            .map(|i| i + 1)
            .unwrap_or(0);
        let next_int = self
            .wallet
            .derivation_index(KeychainKind::Internal)
            .map(|i| i + 1)
            .unwrap_or(0);

        WalletSummary {
            db_path: self.db_path.clone(),
            network: self.wallet.network(),
            external_descriptor_public: self
                .wallet
                .public_descriptor(KeychainKind::External)
                .to_string(),
            internal_descriptor_public: self
                .wallet
                .public_descriptor(KeychainKind::Internal)
                .to_string(),
            next_external_index: next_ext,
            next_internal_index: next_int,
            tip_height: cp.height(),
            tip_hash: cp.hash().to_string(),
        }
    }

    /// Returns detailed breakdown of wallet balances in integer satoshis.
    pub fn get_balance(&self) -> BalanceReport {
        let b = self.wallet.balance();
        BalanceReport {
            confirmed_sats: b.confirmed.to_sat(),
            trusted_pending_sats: b.trusted_pending.to_sat(),
            untrusted_pending_sats: b.untrusted_pending.to_sat(),
            immature_sats: b.immature.to_sat(),
            total_sats: b.total().to_sat(),
        }
    }

    /// Lists all unspent transaction outputs (UTXOs) tracked by the wallet.
    pub fn get_utxos(&self) -> Vec<UtxoEntry> {
        self.wallet
            .list_unspent()
            .map(|utxo| {
                let (is_confirmed, height) = match utxo.chain_position {
                    bdk_wallet::chain::ChainPosition::Confirmed { anchor, .. } => {
                        (true, Some(anchor.block_id.height))
                    }
                    bdk_wallet::chain::ChainPosition::Unconfirmed { .. } => (false, None),
                };

                UtxoEntry {
                    outpoint: utxo.outpoint.to_string(),
                    txid: utxo.outpoint.txid.to_string(),
                    vout: utxo.outpoint.vout,
                    amount_sats: utxo.txout.value.to_sat(),
                    keychain: utxo.keychain,
                    derivation_index: utxo.derivation_index,
                    is_confirmed,
                    confirmation_height: height,
                }
            })
            .collect()
    }

    /// Derives and persists the next external receiving address.
    pub fn new_external_address(&mut self) -> Result<GeneratedAddress, AppError> {
        let info = self.wallet.reveal_next_address(KeychainKind::External);
        self.persist()?;
        Ok(GeneratedAddress {
            address: info.address,
            index: info.index,
            keychain: KeychainKind::External,
            network: self.wallet.network(),
        })
    }

    /// Derives and persists the next internal change address.
    pub fn new_internal_address(&mut self) -> Result<GeneratedAddress, AppError> {
        let info = self.wallet.reveal_next_address(KeychainKind::Internal);
        self.persist()?;
        Ok(GeneratedAddress {
            address: info.address,
            index: info.index,
            keychain: KeychainKind::Internal,
            network: self.wallet.network(),
        })
    }

    /// Constructs, signs, and finalizes a transaction using wallet UTXOs.
    pub fn build_and_sign_transaction(
        &mut self,
        recipient_address_str: &str,
        amount_sats: u64,
        fee_rate_sat_per_vb: Option<u64>,
    ) -> Result<TransactionBuildResult, AppError> {
        if amount_sats == 0 {
            return Err(AppError::ZeroAmount);
        }

        let unchecked_addr = Address::from_str(recipient_address_str)
            .map_err(|e| AppError::InvalidAddress {
                address: recipient_address_str.to_string(),
                reason: e.to_string(),
            })?;

        let recipient_address = unchecked_addr
            .require_network(self.wallet.network())
            .map_err(|e| AppError::AddressNetworkMismatch {
                address: recipient_address_str.to_string(),
                expected: self.wallet.network().to_string(),
                actual: e.to_string(),
            })?;

        let feerate = FeeRate::from_sat_per_vb(fee_rate_sat_per_vb.unwrap_or(1))
            .ok_or_else(|| AppError::SigningError("Invalid fee rate".to_string()))?;

        let mut tx_builder = self.wallet.build_tx();
        tx_builder.add_recipient(recipient_address.script_pubkey(), Amount::from_sat(amount_sats));
        tx_builder.fee_rate(feerate);

        let mut psbt = tx_builder.finish().map_err(|err| match err {
            bdk_wallet::error::CreateTxError::CoinSelection(err) => AppError::InsufficientFunds {
                needed: err.needed.to_sat(),
                available: err.available.to_sat(),
            },
            other => AppError::SigningError(format!("Transaction construction failed: {other}")),
        })?;

        let fee_sats = psbt.fee().map(|f| f.to_sat()).unwrap_or(0);

        let finalized = self
            .wallet
            .sign(&mut psbt, bdk_wallet::SignOptions::default())
            .map_err(|e| AppError::SigningError(e.to_string()))?;

        if !finalized {
            return Err(AppError::TransactionNotFinalized);
        }

        let tx = psbt
            .extract_tx()
            .map_err(|e| AppError::SigningError(format!("Failed to extract finalized transaction: {e}")))?;
        let txid = tx.compute_txid();

        self.persist()?;

        Ok(TransactionBuildResult {
            tx,
            txid,
            fee_sats,
            recipient: recipient_address,
            amount_sats,
            is_finalized: finalized,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_keychain_separation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        std::fs::remove_file(&db_path).unwrap();

        let mut config = AppConfig::default();
        config.db_path = db_path;

        AppWallet::init(&config).expect("init should succeed");
        let mut wallet = AppWallet::open(&config).expect("open should succeed");

        let ext_addr = wallet.new_external_address().expect("new external address");
        let int_addr = wallet.new_internal_address().expect("new internal address");

        assert_eq!(ext_addr.keychain, KeychainKind::External);
        assert_eq!(int_addr.keychain, KeychainKind::Internal);
        assert_eq!(ext_addr.index, 0);
        assert_eq!(int_addr.index, 0);
        // Addresses must differ even at same index because derivation paths differ (0/0 vs 0/1)
        assert_ne!(ext_addr.address, int_addr.address);
    }

    #[test]
    fn test_wallet_init_and_reopen() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        // remove the empty file so init can create fresh
        std::fs::remove_file(&db_path).unwrap();

        let mut config = AppConfig::default();
        config.db_path = db_path.clone();

        // 1. Initial init
        let init_res = AppWallet::init(&config).expect("init should succeed");
        assert!(init_res.external_descriptor_public.contains("wpkh"));
        assert!(init_res.internal_descriptor_public.contains("wpkh"));

        // 2. Cannot init again
        let err = AppWallet::init(&config).unwrap_err();
        assert!(matches!(err, AppError::WalletAlreadyInitialized(_)));

        // 3. Reopen existing wallet
        let mut loaded = AppWallet::open(&config).expect("open should succeed");
        assert_eq!(loaded.wallet.network(), Network::Regtest);

        // 4. Reveal addresses and persist
        let addr0 = loaded.wallet.reveal_next_address(KeychainKind::External);
        assert_eq!(addr0.index, 0);
        loaded.persist().expect("persist should succeed");

        // 5. Reopen again and check index continuity
        let mut reloaded = AppWallet::open(&config).expect("second open should succeed");
        let addr1 = reloaded.wallet.reveal_next_address(KeychainKind::External);
        assert_eq!(addr1.index, 1);
        assert_ne!(addr0.address, addr1.address);
    }

    #[test]
    fn test_wallet_balance_initial() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        std::fs::remove_file(&db_path).unwrap();

        let mut config = AppConfig::default();
        config.db_path = db_path;

        AppWallet::init(&config).expect("init should succeed");
        let wallet = AppWallet::open(&config).expect("open should succeed");

        let balance = wallet.get_balance();
        assert_eq!(balance.confirmed_sats, 0);
        assert_eq!(balance.trusted_pending_sats, 0);
        assert_eq!(balance.untrusted_pending_sats, 0);
        assert_eq!(balance.immature_sats, 0);
        assert_eq!(balance.total_sats, 0);
    }

    #[test]
    fn test_wallet_utxos_initial() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        std::fs::remove_file(&db_path).unwrap();

        let mut config = AppConfig::default();
        config.db_path = db_path;

        AppWallet::init(&config).expect("init should succeed");
        let wallet = AppWallet::open(&config).expect("open should succeed");

        let utxos = wallet.get_utxos();
        assert!(utxos.is_empty());
    }

    #[test]
    fn test_transaction_zero_amount() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        std::fs::remove_file(&db_path).unwrap();

        let mut config = AppConfig::default();
        config.db_path = db_path;

        AppWallet::init(&config).expect("init should succeed");
        let mut wallet = AppWallet::open(&config).expect("open should succeed");

        let err = wallet
            .build_and_sign_transaction("bcrt1qqqsyqcyq5rqwzqfpg9mffdx22kf3702xz78rpm", 0, None)
            .unwrap_err();
        assert!(matches!(err, AppError::ZeroAmount));
    }

    #[test]
    fn test_transaction_invalid_address() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        std::fs::remove_file(&db_path).unwrap();

        let mut config = AppConfig::default();
        config.db_path = db_path;

        AppWallet::init(&config).expect("init should succeed");
        let mut wallet = AppWallet::open(&config).expect("open should succeed");

        let err = wallet
            .build_and_sign_transaction("invalid_not_an_address", 1000, None)
            .unwrap_err();
        assert!(matches!(err, AppError::InvalidAddress { .. }));
    }

    #[test]
    fn test_transaction_mainnet_address_rejected() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        std::fs::remove_file(&db_path).unwrap();

        let mut config = AppConfig::default();
        config.db_path = db_path;

        AppWallet::init(&config).expect("init should succeed");
        let mut wallet = AppWallet::open(&config).expect("open should succeed");

        // Valid mainnet bech32 address
        let mainnet_addr = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
        let err = wallet
            .build_and_sign_transaction(mainnet_addr, 1000, None)
            .unwrap_err();
        assert!(matches!(err, AppError::AddressNetworkMismatch { .. }));
    }

    #[test]
    fn test_transaction_insufficient_funds() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        std::fs::remove_file(&db_path).unwrap();

        let mut config = AppConfig::default();
        config.db_path = db_path;

        AppWallet::init(&config).expect("init should succeed");
        let mut wallet = AppWallet::open(&config).expect("open should succeed");

        let regtest_dest = wallet.new_external_address().unwrap().address.to_string();
        let err = wallet
            .build_and_sign_transaction(&regtest_dest, 10_000, Some(1))
            .unwrap_err();
        assert!(matches!(err, AppError::InsufficientFunds { .. }));
    }

    #[test]
    fn test_transaction_sign_and_finalize() {
        use bitcoin::{
            block::{Header, Version as BlockVersion},
            hashes::Hash,
            transaction::Version as TxVersion,
            absolute::LockTime,
            Block, CompactTarget, OutPoint, ScriptBuf, Sequence, TxIn, TxMerkleNode, TxOut, Witness,
        };

        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        std::fs::remove_file(&db_path).unwrap();

        let mut config = AppConfig::default();
        config.db_path = db_path;

        AppWallet::init(&config).expect("init should succeed");
        let mut wallet = AppWallet::open(&config).expect("open should succeed");

        let receiving_addr = wallet.new_external_address().unwrap().address;
        let funding_tx = bitcoin::Transaction {
            version: TxVersion::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 1,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(100_000),
                script_pubkey: receiving_addr.script_pubkey(),
            }],
        };

        let block = Block {
            header: Header {
                version: BlockVersion::from_consensus(1),
                prev_blockhash: wallet.wallet.latest_checkpoint().hash(),
                merkle_root: TxMerkleNode::all_zeros(),
                time: 1700000000,
                bits: CompactTarget::from_consensus(0x207fffff),
                nonce: 0,
            },
            txdata: vec![funding_tx],
        };

        wallet.wallet.apply_block(&block, 1).expect("apply block");
        wallet.persist().expect("persist");

        assert_eq!(wallet.get_balance().confirmed_sats, 100_000);
        assert_eq!(wallet.get_utxos().len(), 1);

        let dest = wallet.new_external_address().unwrap().address.to_string();
        let res = wallet
            .build_and_sign_transaction(&dest, 40_000, Some(2))
            .expect("transaction build and sign should succeed");

        assert!(res.is_finalized);
        assert_eq!(res.amount_sats, 40_000);
        assert!(res.fee_sats > 0);
        assert_eq!(res.tx.compute_txid(), res.txid);
        // Verify that the transaction is fully signed: witness data is populated
        assert!(!res.tx.input.is_empty());
        assert!(!res.tx.input[0].witness.is_empty());
    }
}
