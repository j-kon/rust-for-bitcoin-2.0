use crate::config::AppConfig;
use crate::error::AppError;
use bdk_bitcoind_rpc::bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoin::{Network, Transaction};
use bitcoincore_rpc::bitcoincore_rpc_json::GetBlockchainInfoResult;

pub struct NodeClient {
    client: Client,
    url: String,
}

#[derive(Debug, Clone)]
pub struct NodeSummary {
    pub url: String,
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    pub best_block_hash: String,
    pub difficulty: f64,
    pub verification_progress: f64,
    pub initial_block_download: bool,
    pub subversion: String,
}

impl NodeClient {
    /// Creates a new Bitcoin Core RPC client from configuration.
    pub fn new(config: &AppConfig) -> Result<Self, AppError> {
        let auth = if let Some(cookie) = &config.rpc_cookie {
            Auth::CookieFile(cookie.clone())
        } else if let (Some(user), Some(pass)) = (&config.rpc_user, &config.rpc_password) {
            Auth::UserPass(user.clone(), pass.clone())
        } else {
            Auth::None
        };

        let client = Client::new(&config.rpc_url, auth).map_err(|err| AppError::NodeOffline {
            url: config.rpc_url.clone(),
            details: err.to_string(),
        })?;

        Ok(Self {
            client,
            url: config.rpc_url.clone(),
        })
    }

    /// Access the underlying bitcoincore-rpc Client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Validates connectivity and verifies that the connected node is on the expected network.
    pub fn check_connection(&self, expected_network: Network) -> Result<GetBlockchainInfoResult, AppError> {
        let info = self.client.get_blockchain_info().map_err(|err| {
            let err_str = err.to_string();
            if err_str.contains("Unauthorized") || err_str.contains("401") {
                AppError::NodeAuthFailed
            } else {
                AppError::NodeOffline {
                    url: self.url.clone(),
                    details: err_str,
                }
            }
        })?;

        if info.chain != expected_network {
            return Err(AppError::NetworkMismatch {
                node_network: info.chain.to_string(),
                expected_network: expected_network.to_string(),
            });
        }

        Ok(info)
    }

    /// Retrieves detailed status summary of the connected Bitcoin Core node.
    pub fn get_summary(&self, expected_network: Network) -> Result<NodeSummary, AppError> {
        let chain_info = self.check_connection(expected_network)?;
        let network_info = self.client.get_network_info().map_err(AppError::Rpc)?;

        Ok(NodeSummary {
            url: self.url.clone(),
            chain: chain_info.chain.to_string(),
            blocks: chain_info.blocks,
            headers: chain_info.headers,
            best_block_hash: chain_info.best_block_hash.to_string(),
            difficulty: chain_info.difficulty,
            verification_progress: chain_info.verification_progress,
            initial_block_download: chain_info.initial_block_download,
            subversion: network_info.subversion,
        })
    }

    /// Broadcasts a signed raw Bitcoin transaction.
    pub fn broadcast_transaction(&self, tx: &Transaction) -> Result<bitcoin::Txid, AppError> {
        self.client.send_raw_transaction(tx).map_err(|err| {
            let err_str = err.to_string();
            AppError::BroadcastRejected(err_str)
        })
    }
}
