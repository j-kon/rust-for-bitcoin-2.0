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
