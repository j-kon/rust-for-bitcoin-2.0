use std::io::{Error, ErrorKind};

pub mod transaction;

pub fn read_bytes<'a>(slice: &mut &'a [u8], len: usize) -> Result<&'a [u8], Error> {
    if slice.len() < len {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            format!("needed {len} bytes, but only {} available", slice.len()),
        ));
    }
    let (head, tail) = slice.split_at(len);
    *slice = tail;
    Ok(head)
}

pub fn read_u8(bytes: &mut &[u8]) -> Result<u8, Error> {
    let buf = read_bytes(bytes, 1)?;
    Ok(buf[0])
}

pub fn read_u16_le(bytes: &mut &[u8]) -> Result<u16, Error> {
    let buf = read_bytes(bytes, 2)?;
    Ok(u16::from_le_bytes([buf[0], buf[1]]))
}

pub fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let buf = read_bytes(bytes_slice, 4)?;
    Ok(u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]))
}

pub fn read_u64(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let buf = read_bytes(transaction_bytes, 8)?;
    Ok(u64::from_le_bytes([
        buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
    ]))
}

pub fn read_amount(transaction_bytes: &mut &[u8]) -> Result<transaction::Amount, Error> {
    let sat = read_u64(transaction_bytes)?;
    Ok(transaction::Amount::from_sat(sat))
}

pub fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let n = read_u8(transaction_bytes)?;
    match n {
        0x00..=0xfc => Ok(n as u64),
        0xfd => {
            let val = read_u16_le(transaction_bytes)?;
            Ok(val as u64)
        }
        0xfe => {
            let val = read_u32(transaction_bytes)?;
            Ok(val as u64)
        }
        0xff => {
            let val = read_u64(transaction_bytes)?;
            Ok(val)
        }
    }
}

pub fn read_txid(transaction_bytes: &mut &[u8]) -> Result<transaction::Txid, Error> {
    let raw = read_bytes(transaction_bytes, 32)?;
    let mut reversed = [0u8; 32];
    for i in 0..32 {
        reversed[i] = raw[31 - i];
    }
    Ok(transaction::Txid::from_bytes(reversed))
}

pub fn read_script_bytes(transaction_bytes: &mut &[u8]) -> Result<Vec<u8>, Error> {
    let len = read_compact_size(transaction_bytes)?;
    let len_usize = usize::try_from(len).map_err(|_| {
        Error::new(ErrorKind::InvalidData, "script length exceeds memory limits")
    })?;
    let bytes = read_bytes(transaction_bytes, len_usize)?;
    Ok(bytes.to_vec())
}

pub fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let bytes = read_script_bytes(transaction_bytes)?;
    Ok(hex::encode(bytes))
}

pub fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

pub fn hash_raw_transaction(raw_transaction_bytes: &[u8]) -> Result<transaction::Txid, Error> {
    use sha2::{Digest, Sha256};
    let first_hash = Sha256::digest(raw_transaction_bytes);
    let second_hash = Sha256::digest(first_hash);
    let mut reversed = [0u8; 32];
    for i in 0..32 {
        reversed[i] = second_hash[31 - i];
    }
    Ok(transaction::Txid::from_bytes(reversed))
}

pub fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<transaction::Txid, Error> {
    hash_raw_transaction(row_transaction_bytes)
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    use transaction::{Input, Output, Transaction};

    let trimmed = transaction_hex.trim();
    if trimmed.is_empty() {
        return Err("empty transaction hex input".into());
    }
    let raw_bytes = hex::decode(trimmed)?;
    let mut cursor = raw_bytes.as_slice();

    // 1. Version (4 bytes LE)
    let version = read_u32(&mut cursor)?;

    // 2. Check SegWit Marker and Flag (0x00 0x01)
    let is_segwit = cursor.len() >= 2 && cursor[0] == 0x00 && cursor[1] == 0x01;
    if is_segwit {
        read_bytes(&mut cursor, 2)?;
    }

    let inputs_start_offset = raw_bytes.len() - cursor.len();

    // 3. Inputs
    let in_count = read_compact_size(&mut cursor)?;
    let mut inputs = Vec::with_capacity(in_count as usize);
    for _ in 0..in_count {
        let prev_txid = read_txid(&mut cursor)?;
        let vout = read_u32(&mut cursor)?;
        let script_sig = read_script_bytes(&mut cursor)?;
        let sequence = read_u32(&mut cursor)?;
        inputs.push(Input {
            txid: prev_txid,
            output_index: vout,
            script_sig,
            sequence,
        });
    }

    // 4. Outputs
    let out_count = read_compact_size(&mut cursor)?;
    let mut outputs = Vec::with_capacity(out_count as usize);
    for _ in 0..out_count {
        let amount = read_amount(&mut cursor)?;
        let script_pubkey = read_script_bytes(&mut cursor)?;
        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    let outputs_end_offset = raw_bytes.len() - cursor.len();

    // 5. Witness Data (SegWit only)
    if is_segwit {
        for _ in 0..in_count {
            let item_count = read_compact_size(&mut cursor)?;
            for _ in 0..item_count {
                let _item_bytes = read_script_bytes(&mut cursor)?;
            }
        }
    }

    // 6. Locktime (4 bytes LE)
    let locktime_offset = raw_bytes.len() - cursor.len();
    let lock_time = read_u32(&mut cursor)?;

    if !cursor.is_empty() {
        return Err(format!(
            "unexpected trailing {} byte(s) after transaction locktime",
            cursor.len()
        )
        .into());
    }

    // 7. TXID Calculation (Double-SHA256 of non-witness serialization)
    let txid = if is_segwit {
        let mut legacy_payload =
            Vec::with_capacity(4 + (outputs_end_offset - inputs_start_offset) + 4);
        legacy_payload.extend_from_slice(&raw_bytes[0..4]);
        legacy_payload.extend_from_slice(&raw_bytes[inputs_start_offset..outputs_end_offset]);
        legacy_payload.extend_from_slice(&raw_bytes[locktime_offset..]);
        hash_raw_transaction(&legacy_payload)?
    } else {
        hash_raw_transaction(&raw_bytes)?
    };

    let tx_struct = Transaction {
        transaction_id: txid,
        version,
        inputs,
        outputs,
        lock_time,
    };

    let json = serde_json::to_string_pretty(&tx_struct)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u32_le() {
        let bytes = [0x02, 0x00, 0x00, 0x00];
        let mut slice = &bytes[..];
        assert_eq!(read_u32(&mut slice).unwrap(), 2);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_read_u64_le() {
        let bytes = [0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut slice = &bytes[..];
        assert_eq!(read_u64(&mut slice).unwrap(), 100);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_compact_size_variants() {
        // Direct u8
        let bytes_u8 = [0x2a];
        let mut slice = &bytes_u8[..];
        assert_eq!(read_compact_size(&mut slice).unwrap(), 42);

        // 0xfd -> u16 LE
        let bytes_u16 = [0xfd, 0x00, 0x02];
        let mut slice = &bytes_u16[..];
        assert_eq!(read_compact_size(&mut slice).unwrap(), 512);

        // 0xfe -> u32 LE
        let bytes_u32 = [0xfe, 0x00, 0x00, 0x01, 0x00];
        let mut slice = &bytes_u32[..];
        assert_eq!(read_compact_size(&mut slice).unwrap(), 65536);

        // 0xff -> u64 LE
        let bytes_u64 = [0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
        let mut slice = &bytes_u64[..];
        assert_eq!(read_compact_size(&mut slice).unwrap(), 4294967296);
    }

    #[test]
    fn test_compact_size_truncated() {
        let bytes_truncated = [0xfd, 0x01];
        let mut slice = &bytes_truncated[..];
        assert!(read_compact_size(&mut slice).is_err());
    }

    #[test]
    fn test_decode_segwit_transaction_sample() {
        let segwit_hex = "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000";
        let json_str = decode_transaction(segwit_hex.to_string()).unwrap();
        assert!(json_str.contains("\"version\": 2"));
        assert!(json_str.contains("\"lock_time\": 0"));
        assert!(json_str.contains("\"transaction_id\""));
    }

    #[test]
    fn test_segwit_txid_differs_from_wtxid() {
        let segwit_hex = "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000";
        let raw_bytes = hex::decode(segwit_hex).unwrap();

        // Full raw hash = wTXID
        let wtxid = hash_raw_transaction(&raw_bytes).unwrap();

        // Decoded TXID (Legacy payload hash)
        let json_str = decode_transaction(segwit_hex.to_string()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let txid_hex = parsed["transaction_id"].as_str().unwrap();

        let wtxid_hex = hex::encode(wtxid.as_bytes());
        assert_ne!(txid_hex, wtxid_hex, "SegWit TXID must NOT equal wTXID!");
    }
}
