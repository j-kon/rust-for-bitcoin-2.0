use crate::compactsize::encode_varint;
use crate::transaction::Transaction;

pub fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    let mut result = Vec::new();

    // 1. Version (4 bytes LE)
    result.extend_from_slice(&trx.version.to_le_bytes());

    // 2. SegWit Marker and Flag (0x00 0x01)
    if trx.segwit {
        result.push(0x00); // marker
        result.push(0x01); // flag
    }

    // 3. Input Count & Inputs
    result.extend_from_slice(&encode_varint(trx.inputs.len()));
    for input in &trx.inputs {
        result.extend_from_slice(&input.prev_txid);
        result.extend_from_slice(&input.vout.to_le_bytes());
        result.extend_from_slice(&encode_varint(input.script_sig.len()));
        result.extend_from_slice(&input.script_sig);
        result.extend_from_slice(&input.sequence.to_le_bytes());
    }

    // 4. Output Count & Outputs
    result.extend_from_slice(&encode_varint(trx.outputs.len()));
    for output in &trx.outputs {
        result.extend_from_slice(&output.value.to_le_bytes());
        result.extend_from_slice(&encode_varint(output.script_pubkey.len()));
        result.extend_from_slice(&output.script_pubkey);
    }

    // 5. Witness Stacks (SegWit only)
    if trx.segwit {
        for input in &trx.inputs {
            result.extend_from_slice(&encode_varint(input.witness.len()));
            for item in &input.witness {
                result.extend_from_slice(&encode_varint(item.len()));
                result.extend_from_slice(item);
            }
        }
    }

    // 6. Locktime (4 bytes LE)
    result.extend_from_slice(&trx.locktime.to_le_bytes());

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TxInput, TxOutput};

    #[test]
    fn test_serialize_legacy_transaction() {
        let input = TxInput {
            prev_txid: vec![0xaa; 32],
            vout: 0,
            script_sig: vec![0x47, 0x30, 0x44],
            sequence: 0xffffffff,
            witness: vec![],
        };
        let output = TxOutput {
            value: 100_000,
            script_pubkey: vec![0x76, 0xa9, 0x14, 0x88, 0xac],
        };
        let trx = Transaction {
            version: 1,
            inputs: vec![input],
            outputs: vec![output],
            locktime: 0,
            segwit: false,
        };

        let serialized = serialize_transaction(&trx);
        assert_eq!(&serialized[0..4], &[1, 0, 0, 0]); // version 1 LE
        assert_eq!(serialized[4], 1); // 1 input
        assert_eq!(&serialized[5..37], &[0xaa; 32]); // prev txid
        assert_eq!(&serialized[37..41], &[0, 0, 0, 0]); // vout 0 LE
        assert_eq!(serialized[41], 3); // script_sig length
        assert_eq!(&serialized[42..45], &[0x47, 0x30, 0x44]); // script_sig
        assert_eq!(&serialized[45..49], &[0xff, 0xff, 0xff, 0xff]); // sequence
        assert_eq!(serialized[49], 1); // 1 output
        assert_eq!(&serialized[50..58], 100_000u64.to_le_bytes()); // satoshis
        assert_eq!(serialized[58], 5); // script_pubkey length
        assert_eq!(&serialized[59..64], &[0x76, 0xa9, 0x14, 0x88, 0xac]);
        assert_eq!(&serialized[64..68], &[0, 0, 0, 0]); // locktime
        assert_eq!(serialized.len(), 68);
    }

    #[test]
    fn test_serialize_segwit_marker_flag() {
        let input = TxInput {
            prev_txid: vec![0xbb; 32],
            vout: 2,
            script_sig: vec![],
            sequence: 0xffffffff,
            witness: vec![vec![0x01, 0x02]],
        };
        let output = TxOutput {
            value: 50_000,
            script_pubkey: vec![0x00, 0x14, 0x11],
        };
        let trx = Transaction {
            version: 2,
            inputs: vec![input],
            outputs: vec![output],
            locktime: 0,
            segwit: true,
        };

        let serialized = serialize_transaction(&trx);
        assert_eq!(&serialized[0..4], &[2, 0, 0, 0]);
        assert_eq!(&serialized[4..6], &[0x00, 0x01]); // SegWit marker + flag
    }
}
