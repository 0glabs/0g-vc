use chrono::NaiveDate;
use keccak_hash::H256;
use tiny_keccak::Hasher;
use tiny_keccak::Keccak;

pub fn date_to_timestamp(date: &NaiveDate) -> u64 {
    date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as u64
}

pub fn encode_fixed_length(input: &str, length: usize) -> Result<Vec<u8>, &'static str> {
    let utf8_bytes = input.as_bytes();

    if utf8_bytes.len() > length {
        return Err("Input string exceeds specified length");
    }

    let mut result = vec![0u8; length];
    result[..utf8_bytes.len()].copy_from_slice(utf8_bytes);

    Ok(result)
}

pub fn keccak_tuple(x: H256, y: H256) -> H256 {
    let mut keccak256 = Keccak::v256();
    keccak256.update(x.as_ref());
    keccak256.update(y.as_ref());
    let mut result = H256::default();
    keccak256.finalize(&mut result.0);
    result
}
