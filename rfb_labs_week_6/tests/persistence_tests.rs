use rfb_labs_week_6::config::AppConfig;
use rfb_labs_week_6::error::AppError;
use rfb_labs_week_6::wallet::AppWallet;
use tempfile::NamedTempFile;

#[test]
fn test_wallet_persistence_lifecycle() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_path_buf();
    std::fs::remove_file(&db_path).unwrap();

    let mut config = AppConfig::default();
    config.db_path = db_path.clone();

    // 1. Initial wallet initialization
    let init_res = AppWallet::init(&config).expect("initialization should succeed");
    assert_eq!(init_res.network, bitcoin::Network::Regtest);
    assert!(init_res.external_descriptor_public.contains("wpkh"));
    assert!(init_res.internal_descriptor_public.contains("wpkh"));

    // 2. Open wallet and reveal addresses
    let (ext_addr0, ext_addr1, change_addr0) = {
        let mut wallet = AppWallet::open(&config).expect("wallet must open");
        let ext0 = wallet.new_external_address().expect("reveal external 0");
        let ext1 = wallet.new_external_address().expect("reveal external 1");
        let ch0 = wallet.new_internal_address().expect("reveal change 0");

        assert_eq!(ext0.index, 0);
        assert_eq!(ext1.index, 1);
        assert_eq!(ch0.index, 0);

        (ext0.address, ext1.address, ch0.address)
    };

    // 3. Reopen wallet from disk (simulating application restart)
    {
        let mut reloaded = AppWallet::open(&config).expect("reloaded wallet must open");

        // Next address must continue the sequence
        let ext2 = reloaded.new_external_address().expect("reveal external 2");
        let ch1 = reloaded.new_internal_address().expect("reveal change 1");

        assert_eq!(ext2.index, 2);
        assert_eq!(ch1.index, 1);

        assert_ne!(ext2.address, ext_addr0);
        assert_ne!(ext2.address, ext_addr1);
        assert_ne!(ch1.address, change_addr0);

        // Check summary reflection
        let summary = reloaded.get_summary();
        assert_eq!(summary.next_external_index, 3);
        assert_eq!(summary.next_internal_index, 2);
    }
}

#[test]
fn test_wallet_refuses_reinitialization() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_path_buf();
    std::fs::remove_file(&db_path).unwrap();

    let mut config = AppConfig::default();
    config.db_path = db_path;

    AppWallet::init(&config).expect("first init must succeed");

    let second_init = AppWallet::init(&config);
    assert!(matches!(
        second_init.unwrap_err(),
        AppError::WalletAlreadyInitialized(_)
    ));
}

#[test]
fn test_uninitialized_wallet_error() {
    let mut config = AppConfig::default();
    config.db_path = std::path::PathBuf::from("./data/non_existent_wallet_test_12345.sqlite");

    let open_res = AppWallet::open(&config);
    assert!(matches!(open_res.err(), Some(AppError::WalletNotInitialized)));
}
