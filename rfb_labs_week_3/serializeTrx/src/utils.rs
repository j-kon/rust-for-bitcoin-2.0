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
