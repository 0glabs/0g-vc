mod input;
mod vc;

pub use input::{Input, PublicInput};
pub use vc::VC;

mod serial_no_format {
    use serde::{self, de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(serial_no: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(serial_no))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_serial_no = String::deserialize(deserializer)?;
        hex::decode(hex_serial_no).map_err(Error::custom)
    }
}

mod birthdate_format {
    use chrono::NaiveDate;
    use serde::{self, de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted_date = date.format("%Y%m%d").to_string();
        serializer.serialize_str(&formatted_date)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, "%Y%m%d").map_err(Error::custom)
    }
}
