use crate::config::AppConfig;
use crate::error::AppError;
use crate::node::NodeClient;
use crate::wallet::AppWallet;

pub struct SendSummary {
    pub txid: bitcoin::Txid,
    pub recipient: String,
    pub amount_sats: u64,
    pub fee_sats: u64,
}

pub fn handle_init(config: &AppConfig) -> Result<(), AppError> {
    let result = AppWallet::init(config)?;
    println!("=== Wallet Initialized Successfully ===");
    println!("Database Location:           {}", result.db_path);
    println!("Network:                     {}", result.network);
    println!("External Public Descriptor:  {}", result.external_descriptor_public);
    println!("Internal Public Descriptor:  {}", result.internal_descriptor_public);
    println!();
    println!("Run `cargo run -- new-address` to get your first receiving address.");
    Ok(())
}

pub fn handle_info(config: &AppConfig) -> Result<(), AppError> {
    let wallet = AppWallet::open(config)?;
    let summary = wallet.get_summary();
    println!("=== Wallet Information ===");
    println!("Database:                    {}", summary.db_path);
    println!("Network:                     {}", summary.network);
    println!("Next External Address Index: {}", summary.next_external_index);
    println!("Next Internal Address Index: {}", summary.next_internal_index);
    println!("Local Chain Tip Height:      {}", summary.tip_height);
    println!("Local Chain Tip Hash:        {}", summary.tip_hash);
    println!("External Public Descriptor:  {}", summary.external_descriptor_public);
    println!("Internal Public Descriptor:  {}", summary.internal_descriptor_public);
    Ok(())
}

pub fn handle_new_address(config: &AppConfig) -> Result<(), AppError> {
    let mut wallet = AppWallet::open(config)?;
    let addr = wallet.new_external_address()?;
    println!("=== New External Receiving Address ===");
    println!("Address:   {}", addr.address);
    println!("Keychain:  {:?}", addr.keychain);
    println!("Index:     {}", addr.index);
    println!("Network:   {}", addr.network);
    Ok(())
}

pub fn handle_new_change_address(config: &AppConfig) -> Result<(), AppError> {
    let mut wallet = AppWallet::open(config)?;
    let addr = wallet.new_internal_address()?;
    println!("=== New Internal Change Address ===");
    println!("Address:   {}", addr.address);
    println!("Keychain:  {:?}", addr.keychain);
    println!("Index:     {}", addr.index);
    println!("Network:   {}", addr.network);
    Ok(())
}

pub fn handle_node_info(config: &AppConfig) -> Result<(), AppError> {
    let node = NodeClient::new(config)?;
    let summary = node.get_summary(config.network)?;
    println!("=== Bitcoin Core Node Info ===");
    println!("RPC URL:                 {}", summary.url);
    println!("Chain:                   {}", summary.chain);
    println!("Blocks:                  {}", summary.blocks);
    println!("Headers:                 {}", summary.headers);
    println!("Best Block Hash:         {}", summary.best_block_hash);
    println!("Difficulty:              {:.6}", summary.difficulty);
    println!("Verification Progress:   {:.4}%", summary.verification_progress * 100.0);
    println!("Initial Block Download:  {}", summary.initial_block_download);
    println!("Node Version / Client:   {}", summary.subversion);
    Ok(())
}

pub fn handle_sync(config: &AppConfig) -> Result<(), AppError> {
    let mut wallet = AppWallet::open(config)?;
    let node = NodeClient::new(config)?;
    println!("Connecting to Bitcoin Core RPC at {}...", config.rpc_url);
    let result = node.sync(&mut wallet)?;
    println!("=== Sync Complete ===");
    println!("Blocks Scanned & Applied: {}", result.blocks_applied);
    println!("Mempool Txs Applied:      {}", result.mempool_txs_applied);
    println!("Wallet Tip Height:        {} -> {}", result.start_height, result.end_height);
    println!("Best Block Hash:          {}", result.best_block_hash);
    Ok(())
}

pub fn handle_balance(config: &AppConfig) -> Result<(), AppError> {
    let wallet = AppWallet::open(config)?;
    let balance = wallet.get_balance();
    println!("=== Wallet Balance ===");
    println!("Confirmed:         {:>12} sats", balance.confirmed_sats);
    println!("Trusted Pending:   {:>12} sats", balance.trusted_pending_sats);
    println!("Untrusted Pending: {:>12} sats", balance.untrusted_pending_sats);
    println!("Immature:          {:>12} sats", balance.immature_sats);
    println!("--------------------------------------");
    println!("Total Balance:     {:>12} sats", balance.total_sats);
    Ok(())
}

pub fn handle_utxos(config: &AppConfig) -> Result<(), AppError> {
    let wallet = AppWallet::open(config)?;
    let utxos = wallet.get_utxos();
    println!("=== Wallet UTXOs (Unspent Outputs) ===");
    if utxos.is_empty() {
        println!("No UTXOs currently tracked by the wallet.");
        println!("Ensure your wallet has received funds and you have run `cargo run -- sync`.");
        return Ok(());
    }

    println!("{:<68} {:>12} {:<10} {:<6} {:<15}", "Outpoint (txid:vout)", "Amount (sat)", "Keychain", "Index", "Status");
    println!("{}", "-".repeat(115));
    for utxo in utxos {
        let status = if utxo.is_confirmed {
            format!("Block {}", utxo.confirmation_height.unwrap_or(0))
        } else {
            "Unconfirmed".to_string()
        };
        println!(
            "{:<68} {:>12} {:<10?} {:<6} {:<15}",
            utxo.outpoint,
            utxo.amount_sats,
            utxo.keychain,
            utxo.derivation_index,
            status
        );
    }
    Ok(())
}

pub fn execute_send(
    config: &AppConfig,
    to: &str,
    amount_sats: u64,
    fee_rate: Option<u64>,
) -> Result<SendSummary, AppError> {
    let mut wallet = AppWallet::open(config)?;
    let node = NodeClient::new(config)?;

    // Verify node connectivity before constructing/signing
    node.check_connection(wallet.wallet.network())?;

    let build_res = wallet.build_and_sign_transaction(to, amount_sats, fee_rate)?;
    let broadcast_txid = node.broadcast_transaction(&build_res.tx)?;

    Ok(SendSummary {
        txid: broadcast_txid,
        recipient: to.to_string(),
        amount_sats,
        fee_sats: build_res.fee_sats,
    })
}

pub fn handle_send(
    config: &AppConfig,
    to: &str,
    amount_sats: u64,
    fee_rate: Option<u64>,
) -> Result<(), AppError> {
    println!("Constructing and signing transaction to {to} for {amount_sats} sats...");
    let summary = execute_send(config, to, amount_sats, fee_rate)?;
    println!("=== Transaction Broadcast Successful ===");
    println!("Transaction ID (txid): {}", summary.txid);
    println!("Recipient:             {}", summary.recipient);
    println!("Amount:                {} sats", summary.amount_sats);
    println!("Network Fee:           {} sats", summary.fee_sats);
    println!("Status:                Broadcast to Bitcoin Core node mempool");
    Ok(())
}
