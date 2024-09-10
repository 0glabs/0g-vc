use crate::utils::{date_to_timestamp, encode_fixed_length};
use chrono::NaiveDate;
use keccak_hash::{keccak, H256};

use serde::{Deserialize, Serialize};

use super::{birthdate_format, serial_no_format};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VC {
    name: String,
    age: u8,
    #[serde(with = "birthdate_format")]
    birth_date: NaiveDate,
    edu_level: u8,
    #[serde(with = "serial_no_format")]
    serial_no: Vec<u8>,
}

const NAME_MAX_LEN: usize = 16;
const SERIAL_MAX_LEN: usize = 32;
pub const VC_LEN: usize = 79;

impl VC {
    pub fn new(
        name: String,
        age: u8,
        birth_date: NaiveDate,
        edu_level: u8,
        serial_no: Vec<u8>,
    ) -> Self {
        Self {
            name,
            age,
            birth_date,
            edu_level,
            serial_no,
        }
    }

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn hash(&self) -> H256 {
        let encoded_vc = self.encode();
        keccak(&encoded_vc)
    }

    pub fn plaintext(&self) -> [u8; VC_LEN + 32] {
        let mut answer = [0u8; VC_LEN + 32];
        let encoded_vc = self.encode();
        answer[0..VC_LEN].copy_from_slice(&encoded_vc);
        let digest = keccak(&encoded_vc);
        answer[VC_LEN..VC_LEN + 32].copy_from_slice(&digest[..]);
        answer
    }

    pub fn file_hash(&self) -> H256 {
        let mut file_data = self.hash().0.to_vec();
        file_data.resize(256, 0);
        keccak(&file_data)
    }

    pub fn encode(&self) -> [u8; VC_LEN] {
        let name_padding = encode_fixed_length(&self.name, NAME_MAX_LEN).unwrap();
        let name = with_prefix("name", name_padding);

        let age = with_prefix("age", [self.age]);

        let birth_date_bytes = date_to_timestamp(&self.birth_date).to_le_bytes();
        let birth_date = with_prefix("birth", birth_date_bytes);

        let edu_level = with_prefix("edu", [self.edu_level]);

        let mut serial_no_padding = self.serial_no.clone();
        serial_no_padding.resize(SERIAL_MAX_LEN, 0);
        let serial_no = with_prefix("serial", serial_no_padding);

        assert!(name.len() == NAME_MAX_LEN + 4);
        assert!(age.len() == 1 + 3);
        assert!(birth_date.len() == 8 + 5);
        assert!(edu_level.len() == 1 + 3);
        assert!(serial_no.len() == SERIAL_MAX_LEN + 6);

        let encoded: Vec<u8> = [name, age, birth_date, edu_level, serial_no]
            .into_iter()
            .flatten()
            .collect();

        encoded.try_into().unwrap()
    }
}

fn with_prefix(prefix: &'static str, iter: impl IntoIterator<Item = u8>) -> Vec<u8> {
    prefix
        .as_bytes()
        .iter()
        .cloned()
        .chain(iter.into_iter())
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn vc_encode_work() {
        let vc_json = r#"{"name": "Alice", "age": 25, "birth_date": "19991231", "edu_level": 4, "serial_no": "1234567890"}"#;
        let vc = VC::from_json(vc_json).unwrap();
        let encoded_vc = vc.encode();
        println!("encoded_vc: {:?}", encoded_vc);
    }
}
