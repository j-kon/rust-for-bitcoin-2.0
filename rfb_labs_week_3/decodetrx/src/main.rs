use clap::{Arg, Command};
use decodetrx::decode_transaction;
use std::process::exit;

fn main() {
    let matches = Command::new("decodetrx")
        .version("1.0")
        .about("Bitcoin Transaction Decoder")
        .arg(
            Arg::new("transaction_hex")
                .help("Raw Bitcoin transaction hex string")
                .required(true)
                .index(1),
        )
        .get_matches();

    let hex_input = matches
        .get_one::<String>("transaction_hex")
        .expect("transaction_hex is required");

    match decode_transaction(hex_input.clone()) {
        Ok(json_output) => {
            println!("{json_output}");
        }
        Err(err) => {
            eprintln!("Error decoding transaction: {err}");
            exit(1);
        }
    }
}
