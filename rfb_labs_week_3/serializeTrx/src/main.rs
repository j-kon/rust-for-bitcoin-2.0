#![allow(non_snake_case)]

use clap::Parser;
use serializeTrx::cli::{run_cli_args, Cli};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run_cli_args(cli) {
        Ok((hex_str, size)) => {
            println!("Serialized transaction:");
            println!("{hex_str}");
            println!("\nTransaction size: {size} bytes");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}
