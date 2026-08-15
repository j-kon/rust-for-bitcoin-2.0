use serde::{Serialize, Serializer};

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Transaction {
    pub transaction_id: Txid,
    pub version: u32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub lock_time: u32,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Input {
    pub txid: Txid,
    pub output_index: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Output {
    #[serde(serialize_with = "as_btc")]
    pub amount: Amount,
    pub script_pubkey: Vec<u8>,
}

fn as_btc<S: Serializer, T: BitcoinValue>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_f64(t.to_btc())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount(pub u64);

impl Amount {
    pub fn from_sat(satoshi: u64) -> Amount {
        Amount(satoshi)
    }

    pub fn satoshis(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Txid(pub [u8; 32]);

impl Txid {
    pub fn from_bytes(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Serialize for Txid {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let hex_str = hex::encode(self.0);
        s.serialize_str(&hex_str)
    }
}

pub trait BitcoinValue {
    fn to_btc(&self) -> f64;
}

impl BitcoinValue for Amount {
    fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_to_btc() {
        assert_eq!(Amount::from_sat(0).to_btc(), 0.0);
        assert_eq!(Amount::from_sat(100_000_000).to_btc(), 1.0);
        assert_eq!(Amount::from_sat(123_456_789).to_btc(), 1.23456789);
        assert_eq!(Amount::from_sat(50).to_btc(), 0.00000050);
    }

    #[test]
    fn test_txid_serialization() {
        let mut bytes = [0u8; 32];
        for (i, item) in bytes.iter_mut().enumerate() {
            *item = i as u8;
        }
        let txid = Txid::from_bytes(bytes);
        let json = serde_json::to_string(&txid).unwrap();
        assert_eq!(
            json,
            "\"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\""
        );
    }

    #[test]
    fn test_output_amount_as_btc_serialization() {
        let output = Output {
            amount: Amount::from_sat(250_000_000),
            script_pubkey: vec![0x76, 0xa9, 0x14],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"amount\":2.5"));
    }
}
