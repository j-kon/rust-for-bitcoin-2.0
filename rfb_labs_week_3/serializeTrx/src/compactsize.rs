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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_varint_boundaries() {
        // 0..=0xfc (1 byte)
        assert_eq!(encode_varint(0), vec![0x00]);
        assert_eq!(encode_varint(0xfc), vec![0xfc]);

        // 0xfd..=0xffff (0xfd + 2 bytes LE)
        assert_eq!(encode_varint(0xfd), vec![0xfd, 0xfd, 0x00]);
        assert_eq!(encode_varint(0xffff), vec![0xfd, 0xff, 0xff]);

        // 0x10000..=0xffff_ffff (0xfe + 4 bytes LE)
        assert_eq!(encode_varint(0x10000), vec![0xfe, 0x00, 0x00, 0x01, 0x00]);
        assert_eq!(encode_varint(0xffff_ffff), vec![0xfe, 0xff, 0xff, 0xff, 0xff]);

        // > 0xffff_ffff (0xff + 8 bytes LE)
        assert_eq!(
            encode_varint(0x1_0000_0000),
            vec![0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]
        );
    }
}
