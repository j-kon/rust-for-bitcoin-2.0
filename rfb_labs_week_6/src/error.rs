use crate::config::ConfigError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Wallet is not initialized. Please run `cargo run -- init` first.")]
    WalletNotInitialized,

    #[error(
        "Wallet is already initialized at '{0}'. Use other commands or specify a different database path."
    )]
    WalletAlreadyInitialized(String),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Bitcoin Core node is offline or unreachable at '{url}': {details}")]
    NodeOffline { url: String, details: String },

    #[error("Bitcoin Core authentication failed. Please check RPC credentials.")]
    NodeAuthFailed,

    #[error(
        "Network mismatch: connected Bitcoin node is on '{node_network}', but wallet is configured for '{expected_network}'."
    )]
    NetworkMismatch {
        node_network: String,
        expected_network: String,
    },

    #[error("Bitcoin Core RPC error: {0}")]
    Rpc(#[from] bitcoincore_rpc::Error),

    #[error("Invalid Bitcoin address '{address}': {reason}")]
    InvalidAddress { address: String, reason: String },

    #[error(
        "Address network mismatch: address '{address}' is for network '{actual}', expected '{expected}'."
    )]
    AddressNetworkMismatch {
        address: String,
        expected: String,
        actual: String,
    },

    #[error("Transaction amount must be strictly greater than 0 satoshis.")]
    ZeroAmount,

    #[error(
        "Insufficient funds: needed {needed} satoshis (including fees), but only {available} satoshis are available."
    )]
    InsufficientFunds { needed: u64, available: u64 },

    #[error("Failed to sign transaction: {0}")]
    SigningError(String),

    #[error("Transaction could not be finalized: PSBT missing required signatures.")]
    TransactionNotFinalized,

    #[error("Broadcast rejected by node: {0}")]
    BroadcastRejected(String),

    #[error("Database persistence error: {0}")]
    Persistence(String),

    #[error("Blockchain sync error: {0}")]
    SyncError(String),

    #[error("Key derivation error: {0}")]
    KeyDerivation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<bdk_wallet::rusqlite::Error> for AppError {
    fn from(err: bdk_wallet::rusqlite::Error) -> Self {
        AppError::Persistence(err.to_string())
    }
}
