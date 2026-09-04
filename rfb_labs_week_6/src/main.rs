use clap::Parser;
use rfb_labs_week_6::cli::{Cli, Commands};
use rfb_labs_week_6::commands;
use rfb_labs_week_6::config::AppConfig;

fn main() {
    let cli = Cli::parse();

    let mut config = match AppConfig::from_env() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Configuration Error: {err}");
            std::process::exit(1);
        }
    };

    // Apply CLI argument overrides
    if let Some(db_path) = cli.db_path {
        config.db_path = db_path;
    }
    if let Some(rpc_url) = cli.rpc_url {
        config.rpc_url = rpc_url;
    }
    if let Some(rpc_user) = cli.rpc_user {
        config.rpc_user = Some(rpc_user);
    }
    if let Some(rpc_password) = cli.rpc_password {
        config.rpc_password = Some(rpc_password);
    }

    let result = match cli.command {
        Commands::Init => commands::handle_init(&config),
        Commands::Info => commands::handle_info(&config),
        Commands::NewAddress => commands::handle_new_address(&config),
        Commands::NewChangeAddress => commands::handle_new_change_address(&config),
        Commands::NodeInfo => commands::handle_node_info(&config),
        Commands::Sync => commands::handle_sync(&config),
        Commands::Balance => commands::handle_balance(&config),
        Commands::Utxos => commands::handle_utxos(&config),
        Commands::Send {
            to,
            amount,
            fee_rate,
        } => commands::handle_send(&config, &to, amount, fee_rate),
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
