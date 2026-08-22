use crate::transaction::{TxInput, TxOutput};
use crate::utils::{hex_to_bytes, txid_to_bytes};

pub fn parse_u32_int_or_hex(val: &str, field_name: &str) -> Result<u32, String> {
    let v = val.trim();
    if v.is_empty() {
        return Err(format!("{field_name} cannot be empty"));
    }
    if let Some(hex_part) = v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
        u32::from_str_radix(hex_part, 16)
            .map_err(|e| format!("{field_name} must be a valid u32 integer or hex: {e}"))
    } else if v.len() == 8
        && v.chars()
            .any(|c| c.is_ascii_hexdigit() && !c.is_ascii_digit())
    {
        u32::from_str_radix(v, 16)
            .map_err(|e| format!("{field_name} must be a valid u32 integer or hex: {e}"))
    } else {
        v.parse::<u32>()
            .map_err(|e| format!("{field_name} must be a valid u32 integer: {e}"))
    }
}

pub fn parse_u64_sats(val: &str, field_name: &str) -> Result<u64, String> {
    let v = val.trim();
    if v.is_empty() {
        return Err(format!("{field_name} cannot be empty"));
    }
    v.parse::<u64>()
        .map_err(|e| format!("{field_name} must be a valid u64 satoshi amount: {e}"))
}

pub fn parse_input_str(s: &str) -> Result<TxInput, String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 4 {
        return Err(format!(
            "malformed input specification '{s}': expected format '<prev_txid>:<vout>:<script_sig_hex>:<sequence>'"
        ));
    }

    let prev_txid = txid_to_bytes(parts[0])?;
    let vout = parse_u32_int_or_hex(parts[1], "input vout")?;
    let script_sig = hex_to_bytes(parts[2])?;
    let sequence = parse_u32_int_or_hex(parts[3], "input sequence")?;

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig,
        sequence,
        witness: Vec::new(),
    })
}

pub fn parse_output_str(s: &str) -> Result<TxOutput, String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "malformed output specification '{s}': expected format '<value_sats>:<script_pubkey_hex>'"
        ));
    }

    let value = parse_u64_sats(parts[0], "output value")?;
    let script_pubkey = hex_to_bytes(parts[1])?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}

pub fn parse_witness_str(s: &str) -> Result<(usize, Vec<u8>), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "malformed witness specification '{s}': expected format '<input_index>:<item_hex>'"
        ));
    }

    let input_index = parts[0]
        .trim()
        .parse::<usize>()
        .map_err(|e| format!("invalid witness input index '{}': {e}", parts[0]))?;
    let item_bytes = hex::decode(parts[1]).map_err(|e| {
        format!(
            "invalid witness item hexadecimal string '{}': {e}",
            parts[1]
        )
    })?;

    Ok((input_index, item_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input_str_valid() {
        let input_str =
            "21c80d2b05c1106360a36e435ad8e418e85d0eb708d9f2bf216476b37bd0b08f:1::ffffffff";
        let input = parse_input_str(input_str).unwrap();
        assert_eq!(input.vout, 1);
        assert_eq!(input.sequence, 0xffffffff);
        assert!(input.script_sig.is_empty());
        assert_eq!(input.prev_txid.len(), 32);
    }

    #[test]
    fn test_parse_input_str_malformed() {
        assert!(parse_input_str("invalid:format").is_err());
        assert!(parse_input_str("21c8:not_an_int::ffffffff").is_err());
    }

    #[test]
    fn test_parse_output_str_valid() {
        let output_str = "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b";
        let output = parse_output_str(output_str).unwrap();
        assert_eq!(output.value, 69886);
        assert_eq!(output.script_pubkey.len(), 22);
    }

    #[test]
    fn test_parse_output_str_malformed() {
        assert!(parse_output_str("invalid_format").is_err());
        assert!(parse_output_str("not_a_number:0014a6").is_err());
    }

    #[test]
    fn test_parse_witness_str_valid() {
        let witness_str = "0:3045022100f8";
        let (idx, bytes) = parse_witness_str(witness_str).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(bytes, vec![0x30, 0x45, 0x02, 0x21, 0x00, 0xf8]);
    }

    #[test]
    fn test_parse_witness_str_malformed() {
        assert!(parse_witness_str("invalid_witness").is_err());
        assert!(parse_witness_str("not_a_number:0014").is_err());
    }
}
