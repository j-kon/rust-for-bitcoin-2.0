use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rfb-wallet")]
#[command(author = "Rust for Bitcoin Student")]
#[command(version = "0.1.0")]
#[command(
    about = "Descriptor-based Bitcoin Regtest CLI Wallet",
    long_about = "A descriptor-based Bitcoin wallet built with bdk_wallet, bitcoincore-rpc, and rusqlite for Week 6 Session 11."
)]
pub struct Cli {
    /// Path to the SQLite wallet database
    #[arg(long, env = "WALLET_DB_PATH")]
    pub db_path: Option<PathBuf>,

    /// Bitcoin Core RPC URL
    #[arg(long, env = "BITCOIN_RPC_URL")]
    pub rpc_url: Option<String>,

    /// Bitcoin Core RPC username
    #[arg(long, env = "BITCOIN_RPC_USER")]
    pub rpc_user: Option<String>,

    /// Bitcoin Core RPC password
    #[arg(long, env = "BITCOIN_RPC_PASSWORD")]
    pub rpc_password: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new disposable regtest wallet with fresh key material
    Init,

    /// Display wallet public descriptors, address derivation indices, and local chain status
    Info,

    /// Generate and reveal the next external receiving address
    NewAddress,

    /// Generate and reveal the next internal change address
    NewChangeAddress,

    /// Query connected Bitcoin Core node status and verification progress
    NodeInfo,

    /// Synchronize wallet chain state and UTXOs against Bitcoin Core
    Sync,

    /// Display wallet balance breakdown in integer satoshis
    Balance,

    /// List all unspent transaction outputs (UTXOs) tracked by the wallet
    Utxos,

    /// Construct, sign, and broadcast a Bitcoin transaction
    Send {
        /// Recipient Bitcoin address (must be valid for regtest)
        #[arg(long)]
        to: String,

        /// Amount to send in integer satoshis
        #[arg(long)]
        amount: u64,

        /// Transaction feerate in sats/vB (defaults to 1 sat/vB)
        #[arg(long)]
        fee_rate: Option<u64>,
    },
}
