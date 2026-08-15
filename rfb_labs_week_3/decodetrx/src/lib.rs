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
}
