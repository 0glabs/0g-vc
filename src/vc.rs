use core::num;
use std::result;

use super::utils::*;
use chrono::NaiveDate;
// use sha2::{Digest, Sha256};
use tiny_keccak::{Keccak, Hasher};

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
// name 最长为 NAME_MAX_LEN 字节
#[derive(Debug, Clone)]
pub struct VC {
    name: String,
    age: u8,
    birth_date: String,
    edu_level: u8,
    serial_no: String,
}

const NAME_MAX_LEN: usize = 16;
const SERIAL_MAX_LEN: usize = 32;

// add meta
#[derive(Debug, Clone)]
pub struct EncodedVC {
    name: Vec<u8>,
    age: Vec<u8>,
    birth_date: Vec<u8>,
    edu_level: Vec<u8>,
    serial_no: Vec<u8>,
}

impl VC {
    pub fn hash(&self) -> (EncodedVC, Vec<u8>) {
        // preapre for hash
        let encoded_vc = self.encode_for_hash();
        assert!(encoded_vc.name.len() == NAME_MAX_LEN + 4);
        assert!(encoded_vc.age.len() == 1 + 3);
        assert!(encoded_vc.birth_date.len() == 8 + 5);
        assert!(encoded_vc.edu_level.len() == 1 + 3);
        assert!(encoded_vc.serial_no.len() == SERIAL_MAX_LEN + 6);
        let mut input = encoded_vc.join();
        assert!(input.len() == 79);

        // 填充输入数据到 256 字节
        input.resize(256, 0x00);
        assert!(input.len() == 256);

        let mut hasher = Keccak::v256();
        hasher.update(&input.as_slice());
        let mut result = [0u8; 32];
        hasher.finalize(&mut result);

        // 将散列值转换为十六进制字符串表示形式
        (encoded_vc, result.iter().map(|byte| *byte).collect())
    }

    pub fn birth_date(&self) -> u64 {
        NaiveDate::parse_from_str(&self.birth_date, "%Y%m%d")
            .expect("Invalid birth date string")
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp() as u64
    }

    pub fn encode_for_hash(&self) -> EncodedVC {
        println!("birth_date: {}", self.birth_date());
        let birth_date_bytes = u64_to_u8_array(self.birth_date());
        let name = with_prefix(
            "name",
            encode_fixed_length(&self.name, NAME_MAX_LEN).unwrap(),
        );
        let age = with_prefix("age", std::iter::once(self.age));
        let birth_date = with_prefix("birth", birth_date_bytes.into_iter());
        let edu_level = with_prefix("edu", std::iter::once(self.edu_level));
        let serial_no = with_prefix(
            "serial",
            hex_string_to_bytes(&self.serial_no, SERIAL_MAX_LEN).unwrap(),
        );
        EncodedVC {
            name,
            age,
            birth_date,
            edu_level,
            serial_no,
        }
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

impl EncodedVC {
    // 定义一个 join 方法来将结构体的字段拼接为一个 Vec<u8>
    pub fn join(&self) -> Vec<u8> {
        let mut result = Vec::new();
        // 将每个字段逐个拼接到 result 中
        result.extend_from_slice(&self.name);
        result.extend_from_slice(&self.age);
        result.extend_from_slice(&self.birth_date);
        result.extend_from_slice(&self.edu_level);
        result.extend_from_slice(&self.serial_no);
        result
    }
}

use ark_bn254::Fr;
pub fn pub_input_without_witness(
    birth_date_threshold: String,
    target_leaf_hash: Vec<u8>
) -> (Fr, Fr) {
    use ark_ff::PrimeField;
    use num_bigint::{BigUint, BigInt, Sign};
    use num_traits::Signed;

    let birth_date_threshold = BigInt::from(NaiveDate::parse_from_str(birth_date_threshold.as_str(), "%Y%m%d")
        .expect("Invalid birth date string")
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp() as u64);
    
    let modulus = <Fr as PrimeField>::MODULUS;

    let birth_date_threshold = if birth_date_threshold.sign() == num_bigint::Sign::Minus {
        let birth_date_threshold_abs = birth_date_threshold.abs().to_biguint().unwrap();
        BigUint::from(modulus) - birth_date_threshold_abs
    } else {
        birth_date_threshold.to_biguint().unwrap()
    };

    let target_leaf_hash = BigUint::from_bytes_le(&target_leaf_hash);

    (Fr::from(birth_date_threshold), Fr::from(target_leaf_hash))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn vc_ecnode_work() {
        let vc_json = r#"{"name": "Alice", "age": 25, "birth_date": "19991231", "edu_level": 4, "serial_no": "1234567890"}"#;
        let vc: VC = serde_json::from_str(vc_json).unwrap();
        let encoded_vc = vc.encode_for_hash();
        println!("encoded_vc: {:?}", encoded_vc);
        println!("encoded_vc_len: {}", encoded_vc.join().len());
    }

    #[test]
    fn vc_birth_date_encode() {
        let vc_json = r#"{"name": "Alice", "age": 25, "birth_date": "19991231", "edu_level": 4, "serial_no": "1234567890"}"#;
        let vc: VC = serde_json::from_str(vc_json).unwrap();
        let encoded_birth_date = vc.birth_date();
        println!("encoded_birth_date: {:?}", encoded_birth_date);

        let vc_json = r#"{"name": "Alice", "age": 25, "birth_date": "20000101", "edu_level": 4, "serial_no": "1234567890"}"#;
        let vc: VC = serde_json::from_str(vc_json).unwrap();
        let encoded_birth_date = vc.birth_date();
        println!("encoded_birth_date: {:?}", encoded_birth_date);
    }
}
