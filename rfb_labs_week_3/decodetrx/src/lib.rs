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

pub fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

pub fn decode_transaction(_transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::new())
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
}
