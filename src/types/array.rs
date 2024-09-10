use crate::Signal;
use num_bigint::BigInt as CircomBigInt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteArray<const N: usize>(#[serde(with = "byte_array_format")] [u8; N]);

impl<const N: usize> ByteArray<N> {
    pub fn new(inner: [u8; N]) -> Self {
        ByteArray(inner)
    }
}

impl<const N: usize> AsRef<[u8]> for ByteArray<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl<const N: usize> AsRef<[u8; N]> for ByteArray<N> {
    fn as_ref(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize> Signal for ByteArray<N> {
    fn to_signal(&self) -> Vec<CircomBigInt> {
        self.0.iter().map(|x| CircomBigInt::from(*x)).collect()
    }
}

mod byte_array_format {
    use serde::{self, de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<const N: usize, S>(array: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted_array = hex::encode(array);
        serializer.serialize_str(&formatted_array)
    }

    pub fn deserialize<'de, const N: usize, D>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let vec =
            hex::decode(s).map_err(|e| Error::custom(format!("cannot decode hex: {:?}", e)))?;
        if vec.len() != N {
            return Err(Error::custom(format!(
                "Invalid hex length, expected {}, actual {}",
                N,
                vec.len()
            )));
        }

        Ok(vec.try_into().unwrap())
    }
}
