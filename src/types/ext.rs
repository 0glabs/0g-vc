use std::ops::Deref;

use super::birthdate_format;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionSignal {
    Date(#[serde(with = "birthdate_format")] NaiveDate),
    Number(u128),
}

pub const NUM_EXTENSIONS: usize = 16;
#[derive(Serialize, Deserialize, Debug)]
pub struct Extensions(Vec<ExtensionSignal>);

impl TryFrom<Vec<ExtensionSignal>> for Extensions {
    type Error = &'static str;

    fn try_from(value: Vec<ExtensionSignal>) -> Result<Self, Self::Error> {
        if value.len() > NUM_EXTENSIONS {
            return Err("Too many extension signals");
        }
        Ok(Extensions(value))
    }
}

impl Deref for Extensions {
    type Target = [ExtensionSignal];

    fn deref(&self) -> &Self::Target {
        &self.0[..]
    }
}

#[test]
fn test() {
    let date_field = ExtensionSignal::Date(NaiveDate::from_ymd_opt(2005, 6, 7).unwrap());
    let number_field = ExtensionSignal::Number(123);

    let serialized_date = serde_json::to_string(&date_field).unwrap();
    let serialized_number = serde_json::to_string(&number_field).unwrap();

    println!("Serialized Date: {}", serialized_date);
    println!("Serialized Number: {}", serialized_number);

    let deserialized_date: ExtensionSignal = serde_json::from_str(&serialized_date).unwrap();
    let deserialized_number: ExtensionSignal = serde_json::from_str(&serialized_number).unwrap();

    assert_eq!(date_field, deserialized_date);
    assert_eq!(number_field, deserialized_number);

    println!("Deserialized Date: {:?}", deserialized_date);
    println!("Deserialized Number: {:?}", deserialized_number);
}
// mod ext_format {
//     use std::collections::HashMap;

//     use serde::{self, de::Error, Deserialize, Deserializer, Serializer, ser::SerializeMap};
//     use super::ExtensionField;

//     pub fn serialize<S>(ext: &ExtensionField, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut map: HashMap<String, String> = HashMap::new();
//         match ext {
//             ExtensionField::Date(date) => {
//                 let formatted_date = date.format("%Y%m%d").to_string();
//                 map.insert("date".into(), formatted_date);

//             }
//             ExtensionField::Number(num) => {
//                 map.serialize_entry("number", num)?;
//             },
//         }
//         map.end()
//     }

//     pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
//     where
//         D: Deserializer<'de>,
//     {

//         let hex_serial_no = String::deserialize(deserializer)?;
//         hex::decode(hex_serial_no).map_err(Error::custom)
//     }
// }
