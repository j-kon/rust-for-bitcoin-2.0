pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    let s = hex_str.trim();
    if s.is_empty() {
        return Ok(Vec::new());
    }
    if s.len() % 2 != 0 {
        return Err("hexadecimal string must have an even length".to_string());
    }
    hex::decode(s).map_err(|e| format!("invalid hexadecimal string '{s}': {e}"))
}

pub fn txid_to_bytes(txid_str: &str) -> Result<Vec<u8>, String> {
    let bytes = hex_to_bytes(txid_str)?;
    if bytes.len() != 32 {
        return Err(format!(
            "invalid previous transaction id length: expected 32 bytes (64 hex characters), got {} bytes",
            bytes.len()
        ));
    }
    let mut reversed = bytes;
    reversed.reverse();
    Ok(reversed)
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_bytes_valid() {
        assert_eq!(hex_to_bytes("0014a6").unwrap(), vec![0x00, 0x14, 0xa6]);
        assert_eq!(hex_to_bytes("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn test_hex_to_bytes_odd_length() {
        assert!(hex_to_bytes("0014a").is_err());
    }

    #[test]
    fn test_hex_to_bytes_invalid_chars() {
        assert!(hex_to_bytes("0014zz").is_err());
    }

    #[test]
    fn test_txid_to_bytes_reversal() {
        let display_txid = "21c80d2b05c1106360a36e435ad8e418e85d0eb708d9f2bf216476b37bd0b08f";
        let wire_bytes = txid_to_bytes(display_txid).unwrap();
        let expected_wire_hex = "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821";
        assert_eq!(bytes_to_hex(&wire_bytes), expected_wire_hex);
    }

    #[test]
    fn test_txid_to_bytes_invalid_length() {
        assert!(txid_to_bytes("0014a6").is_err());
    }
}
