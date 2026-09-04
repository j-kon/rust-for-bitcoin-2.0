use crate::config::AppConfig;
use crate::error::AppError;
use bdk_wallet::{
    bitcoin::{
        bip32::{DerivationPath, Xpriv},
        secp256k1::Secp256k1,
        Network,
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

pub struct AppWallet {
    pub wallet: PersistedWallet<Connection>,
    pub conn: Connection,
    pub external_descriptor: String,
    pub internal_descriptor: String,
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
        })
    }

    /// Persists wallet changeset to SQLite.
    pub fn persist(&mut self) -> Result<bool, AppError> {
        self.wallet
            .persist(&mut self.conn)
            .map_err(|e| AppError::Persistence(format!("Failed to persist wallet: {e}")))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

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
}
