use rfb_labs_week_6::config::{AppConfig, ConfigError};
use rfb_labs_week_6::error::AppError;
use rfb_labs_week_6::wallet::AppWallet;
use tempfile::NamedTempFile;

#[test]
fn test_config_network_enforcement() {
    assert!(matches!(
        AppConfig::parse_and_validate_network("mainnet").unwrap_err(),
        ConfigError::InvalidNetwork(_)
    ));
    assert!(matches!(
        AppConfig::parse_and_validate_network("bitcoin").unwrap_err(),
        ConfigError::InvalidNetwork(_)
    ));
    assert!(matches!(
        AppConfig::parse_and_validate_network("testnet").unwrap_err(),
        ConfigError::InvalidNetwork(_)
    ));
    assert_eq!(
        AppConfig::parse_and_validate_network("regtest").unwrap(),
        bitcoin::Network::Regtest
    );
}

#[test]
fn test_transaction_input_validation() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_path_buf();
    std::fs::remove_file(&db_path).unwrap();

    let mut config = AppConfig::default();
    config.db_path = db_path;

    AppWallet::init(&config).expect("init succeeds");
    let mut wallet = AppWallet::open(&config).expect("open succeeds");

    // 1. Zero amount
    let regtest_dest = wallet.new_external_address().unwrap().address.to_string();
    let zero_err = wallet.build_and_sign_transaction(&regtest_dest, 0, None);
    assert!(matches!(zero_err.err(), Some(AppError::ZeroAmount)));

    // 2. Malformed address
    let bad_addr_err = wallet.build_and_sign_transaction("not_a_valid_bitcoin_address", 1000, None);
    assert!(matches!(bad_addr_err.err(), Some(AppError::InvalidAddress { .. })));

    // 3. Mainnet address rejection on regtest
    let mainnet_addr = "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq";
    let network_err = wallet.build_and_sign_transaction(mainnet_addr, 1000, None);
    assert!(matches!(network_err.err(), Some(AppError::AddressNetworkMismatch { .. })));

    // 4. Testnet address rejection on regtest (tb1...)
    let testnet_addr = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
    let testnet_err = wallet.build_and_sign_transaction(testnet_addr, 1000, None);
    assert!(matches!(testnet_err.err(), Some(AppError::AddressNetworkMismatch { .. })));

    // 5. Insufficient funds
    let funds_err = wallet.build_and_sign_transaction(&regtest_dest, 50_000, Some(1));
    assert!(matches!(
        funds_err.err(),
        Some(AppError::InsufficientFunds { needed: _, available: 0 })
    ));
}
