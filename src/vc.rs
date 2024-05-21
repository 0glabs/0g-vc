use super::utils::*;
use ark_serialize::Read;
use chrono::NaiveDate;
use sha2::{Digest, Sha256};

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
        let input = encoded_vc.join();
        assert!(input.len() == 79);

        // 创建 SHA-256 散列器
        let mut hasher = Sha256::new();

        // 将数据输入散列器
        hasher.update(input.clone());

        // 计算散列值
        let result = hasher.finalize();

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
        let mut name = "name"
            .as_bytes()
            .to_vec()
            .into_iter()
            .chain(encode_fixed_length(&self.name, NAME_MAX_LEN).unwrap())
            .collect();
        let mut age: Vec<u8> = "age"
            .as_bytes()
            .to_vec()
            .into_iter()
            .chain(std::iter::once(self.age))
            .collect();
        let mut birth_date = "birth"
            .as_bytes()
            .to_vec()
            .into_iter()
            .chain(birth_date_bytes.iter().cloned())
            .collect();
        let mut edu_level: Vec<u8> = "edu"
            .as_bytes()
            .to_vec()
            .into_iter()
            .chain(std::iter::once(self.edu_level))
            .collect();
        let mut serial_no = "serial"
            .as_bytes()
            .to_vec()
            .into_iter()
            .chain(hex_string_to_bytes(&self.serial_no, SERIAL_MAX_LEN).unwrap())
            .collect();
        EncodedVC {
            name,
            age,
            birth_date,
            edu_level,
            serial_no,
        }
    }

    // pub fn prepare_for_hash(&self) -> Vec<u8> {
    //     // 将"20000304"格式的日期字符串转为Unix时间戳
    //     let birth_date_bytes = u64_to_u8_array(self.birth_date());
    //     [
    //         encode_fixed_length(&self.name, NAME_MAX_LEN)
    //             .unwrap()
    //             .as_slice(),
    //         [self.age].as_slice(),
    //         birth_date_bytes.as_slice(),
    //         [self.edu_level].as_slice(),
    //         hex_string_to_bytes(&self.serial_no, SERIAL_MAX_LEN)
    //             .unwrap()
    //             .as_slice(),
    //     ]
    //     .iter()
    //     .flat_map(|&arr| arr.iter())
    //     .cloned()
    //     .collect::<Vec<u8>>()
    // }
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
