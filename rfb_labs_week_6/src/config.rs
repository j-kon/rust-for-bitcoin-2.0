use bitcoin::Network;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("Invalid network '{0}': only 'regtest' is allowed for this assignment")]
    InvalidNetwork(String),

    #[error("Failed to parse network: {0}")]
    NetworkParseError(String),

    #[error("Invalid RPC URL: {0}")]
    InvalidRpcUrl(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub rpc_url: String,
    pub rpc_user: Option<String>,
    pub rpc_password: Option<String>,
    pub rpc_cookie: Option<PathBuf>,
    pub db_path: PathBuf,
    pub network: Network,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://127.0.0.1:18443".to_string(),
            rpc_user: Some("regtest_user".to_string()),
            rpc_password: Some("regtest_password".to_string()),
            rpc_cookie: None,
            db_path: PathBuf::from("./data/wallet.sqlite"),
            network: Network::Regtest,
        }
    }
}

impl AppConfig {
    /// Loads configuration from environment variables with safe defaults.
    pub fn from_env() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();

        let network_str =
            std::env::var("BITCOIN_NETWORK").unwrap_or_else(|_| "regtest".to_string());
        let network = Self::parse_and_validate_network(&network_str)?;

        let rpc_url = std::env::var("BITCOIN_RPC_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:18443".to_string());
        if rpc_url.is_empty() {
            return Err(ConfigError::InvalidRpcUrl(
                "RPC URL cannot be empty".to_string(),
            ));
        }

        let rpc_user = std::env::var("BITCOIN_RPC_USER")
            .ok()
            .filter(|s| !s.is_empty());
        let rpc_password = std::env::var("BITCOIN_RPC_PASSWORD")
            .ok()
            .filter(|s| !s.is_empty());
        let rpc_cookie = std::env::var("BITCOIN_RPC_COOKIE")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);

        let db_path_str =
            std::env::var("WALLET_DB_PATH").unwrap_or_else(|_| "./data/wallet.sqlite".to_string());
        let db_path = PathBuf::from(db_path_str);

        Ok(Self {
            rpc_url,
            rpc_user,
            rpc_password,
            rpc_cookie,
            db_path,
            network,
        })
    }

    /// Enforces that the network is strictly Regtest for safety.
    pub fn parse_and_validate_network(network_str: &str) -> Result<Network, ConfigError> {
        let trimmed = network_str.trim().to_ascii_lowercase();
        match trimmed.as_str() {
            "regtest" => Ok(Network::Regtest),
            "bitcoin" | "mainnet" => Err(ConfigError::InvalidNetwork(
                "Mainnet is strictly prohibited for security reasons".to_string(),
            )),
            "testnet" | "testnet4" | "signet" => Err(ConfigError::InvalidNetwork(format!(
                "Configured for '{trimmed}', but regtest is required for this lab",
            ))),
            other => Err(ConfigError::NetworkParseError(other.to_string())),
        }
    }

    /// Ensures the parent directory for the SQLite database exists.
    pub fn ensure_db_dir(&self) -> std::io::Result<()> {
        if let Some(parent) = self.db_path.parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.network, Network::Regtest);
        assert_eq!(config.db_path, PathBuf::from("./data/wallet.sqlite"));
        assert_eq!(config.rpc_url, "http://127.0.0.1:18443");
    }

    #[test]
    fn test_network_validation_regtest() {
        assert_eq!(
            AppConfig::parse_and_validate_network("regtest").unwrap(),
            Network::Regtest
        );
        assert_eq!(
            AppConfig::parse_and_validate_network("REGTEST").unwrap(),
            Network::Regtest
        );
    }

    #[test]
    fn test_network_validation_mainnet_rejected() {
        let err = AppConfig::parse_and_validate_network("mainnet").unwrap_err();
        assert!(matches!(err, ConfigError::InvalidNetwork(_)));
    }

    #[test]
    fn test_network_validation_testnet_rejected() {
        let err = AppConfig::parse_and_validate_network("testnet").unwrap_err();
        assert!(matches!(err, ConfigError::InvalidNetwork(_)));
    }
}
