use clap::Parser;
use crate::parser::{parse_input_str, parse_output_str, parse_witness_str};
use crate::serializer::serialize_transaction;
use crate::transaction::Transaction;
use crate::utils::bytes_to_hex;

#[derive(Parser, Debug)]
#[command(
    name = "serializeTrx",
    about = "Construct and serialize Bitcoin transactions from command-line arguments"
)]
pub struct Cli {
    /// Transaction version number
    #[arg(short = 'v', long, default_value = "2")]
    pub version: i32,

    /// SegWit transaction flag
    #[arg(short = 's', long)]
    pub segwit: bool,

    /// Transaction input specification: '<prev_txid>:<vout>:<script_sig_hex>:<sequence>'
    #[arg(short = 'i', long = "input", action = clap::ArgAction::Append)]
    pub inputs: Vec<String>,

    /// Transaction output specification: '<value_sats>:<script_pubkey_hex>'
    #[arg(short = 'o', long = "output", action = clap::ArgAction::Append)]
    pub outputs: Vec<String>,

    /// Witness item specification: '<input_index>:<item_hex>'
    #[arg(short = 'w', long = "witness", action = clap::ArgAction::Append)]
    pub witnesses: Vec<String>,

    /// Transaction locktime
    #[arg(short = 'l', long, default_value = "0")]
    pub locktime: u32,
}

pub fn build_transaction_from_cli(cli: Cli) -> Result<Transaction, String> {
    if cli.inputs.is_empty() {
        return Err("transaction must contain at least one input specification (--input)".to_string());
    }
    if cli.outputs.is_empty() {
        return Err("transaction must contain at least one output specification (--output)".to_string());
    }

    let mut inputs = Vec::with_capacity(cli.inputs.len());
    for (idx, input_str) in cli.inputs.iter().enumerate() {
        let input = parse_input_str(input_str)
            .map_err(|e| format!("invalid input #{idx}: {e}"))?;
        inputs.push(input);
    }

    let mut outputs = Vec::with_capacity(cli.outputs.len());
    for (idx, output_str) in cli.outputs.iter().enumerate() {
        let output = parse_output_str(output_str)
            .map_err(|e| format!("invalid output #{idx}: {e}"))?;
        outputs.push(output);
    }

    if !cli.segwit && !cli.witnesses.is_empty() {
        return Err(
            "witness data (--witness) cannot be supplied for a non-SegWit transaction"
                .to_string(),
        );
    }

    for (idx, witness_str) in cli.witnesses.iter().enumerate() {
        let (input_index, item_bytes) = parse_witness_str(witness_str)
            .map_err(|e| format!("invalid witness specification #{idx}: {e}"))?;
        if input_index >= inputs.len() {
            return Err(format!(
                "witness #{idx} references input index {input_index}, but transaction only has {} input(s)",
                inputs.len()
            ));
        }
        inputs[input_index].witness.push(item_bytes);
    }

    Ok(Transaction {
        version: cli.version,
        inputs,
        outputs,
        locktime: cli.locktime,
        segwit: cli.segwit,
    })
}

pub fn run_cli_args(cli: Cli) -> Result<(String, usize), String> {
    let trx = build_transaction_from_cli(cli)?;
    let serialized = serialize_transaction(&trx);
    let hex_str = bytes_to_hex(&serialized);
    let len = serialized.len();
    Ok((hex_str, len))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TxInput, TxOutput};
    use crate::utils::hex_to_bytes;

    #[test]
    fn test_starter_transaction_compatibility() {
        let hardcoded_input = TxInput {
            prev_txid: hex_to_bytes("8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821").unwrap(),
            vout: 1,
            script_sig: vec![],
            sequence: 0xffffffff,
            witness: vec![
                hex_to_bytes("3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301").unwrap(),
                hex_to_bytes("029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358").unwrap(),
            ],
        };
        let hardcoded_output_0 = TxOutput {
            value: 69886,
            script_pubkey: hex_to_bytes("0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b").unwrap(),
        };
        let hardcoded_output_1 = TxOutput {
            value: 29442,
            script_pubkey: hex_to_bytes("00149831122b93d21715c70db626ccc844d3c21f9687").unwrap(),
        };
        let hardcoded_trx = Transaction {
            version: 2,
            inputs: vec![hardcoded_input],
            outputs: vec![hardcoded_output_0, hardcoded_output_1],
            locktime: 0,
            segwit: true,
        };
        let expected_bytes = serialize_transaction(&hardcoded_trx);
        let expected_hex = bytes_to_hex(&expected_bytes);

        let cli = Cli {
            version: 2,
            segwit: true,
            inputs: vec!["21c80d2b05c1106360a36e435ad8e418e85d0eb708d9f2bf216476b37bd0b08f:1::ffffffff".to_string()],
            outputs: vec![
                "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b".to_string(),
                "29442:00149831122b93d21715c70db626ccc844d3c21f9687".to_string(),
            ],
            witnesses: vec![
                "0:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301".to_string(),
                "0:029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358".to_string(),
            ],
            locktime: 0,
        };

        let (cli_hex, cli_len) = run_cli_args(cli).unwrap();
        assert_eq!(cli_hex, expected_hex);
        assert_eq!(cli_len, expected_bytes.len());
        assert_eq!(cli_len, 223);
    }
}
