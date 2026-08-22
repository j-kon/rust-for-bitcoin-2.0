pub fn encode_varint(value: usize) -> Vec<u8> {
    match value {
        0..=0xfc => vec![value as u8],
        0xfd..=0xffff => {
            let mut result = Vec::with_capacity(3);
            result.push(0xfd);
            result.extend_from_slice(&(value as u16).to_le_bytes());
            result
        }
        0x10000..=0xffff_ffff => {
            let mut result = Vec::with_capacity(5);
            result.push(0xfe);
            result.extend_from_slice(&(value as u32).to_le_bytes());
            result
        }
        _ => {
            let mut result = Vec::with_capacity(9);
            result.push(0xff);
            result.extend_from_slice(&(value as u64).to_le_bytes());
            result
        }
    }
}
