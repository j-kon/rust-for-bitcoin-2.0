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

/// Executes transaction construction, signing, and RPC broadcasting.
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
