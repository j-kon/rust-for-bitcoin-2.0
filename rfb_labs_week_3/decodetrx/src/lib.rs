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
}
