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

pub const SECRETS_TABLE_NAME: &str = "_wallet_secrets";

pub struct WalletInitResult {
    pub db_path: String,
    pub network: Network,
    pub external_descriptor_public: String,
    pub internal_descriptor_public: String,
}

pub struct AppWallet {
    pub wallet: Wallet,
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
}
