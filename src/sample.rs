//! Generate example input for benchmark

use chrono::NaiveDate;
use keccak_hash::H256;

use rand::SeedableRng;
use rand_xorshift::XorShiftRng;

use crate::types::{ByteArray, VcProveInput, VcVerifyInput, VC};

pub struct Sample;

impl Sample {
    pub fn vc() -> VC {
        let vc_json = r#"{"name": "Alice", "age": 25, "birth_date": "20000101", "edu_level": 4, "serial_no": "1234567890"}"#;
        VC::from_json(vc_json).unwrap()
    }

    pub fn threshold() -> NaiveDate {
        NaiveDate::parse_from_str("20000304", "%Y%m%d").unwrap()
    }

    pub fn merkle_path(depth: usize) -> Vec<H256> {
        let mut rng = XorShiftRng::seed_from_u64(22);
        let mut random_hash = || {
            let mut x = H256::random_using(&mut rng);
            let (lo, hi) = x.0.split_at_mut(16);
            lo.copy_from_slice(hi);
            x
        };
        (0..depth).map(|_| random_hash()).collect()
    }

    pub fn input() -> VcProveInput {
        let key = ByteArray::new(*b"verysecretkey123");
        let iv = ByteArray::new(*b"uniqueiv12345678");
        VcProveInput::new(
            Self::vc(),
            key,
            iv,
            Self::threshold(),
            Self::merkle_path(3),
            0,
        )
    }

    pub fn public_input() -> VcVerifyInput {
        VcVerifyInput::new(Self::threshold(), Self::input().merkle_root())
    }
}
