use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

pub struct U32StringSerde;

impl U32StringSerde {
  pub fn serialize<S>(val: &u32, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    String::serialize(&val.to_string(), serializer)
  }

  pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
  where
    D: Deserializer<'de>,
  {
    u32::from_str(&String::deserialize(deserializer)?)
      .map_err(|e| de::Error::custom(format!("u32 from string error: {}", e)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, PartialEq, Serialize, Deserialize)]
  struct Test {
    #[serde(with = "U32StringSerde")]
    v: u32,
  }

  #[test]
  fn test_u32_serialize_string() {
    let obj = Test { v: 33 };
    let json_str = serde_json::to_string(&obj).unwrap();

    assert_eq!(json_str, r##"{"v":"33"}"##)
  }

  #[test]
  fn test_u32_deserialize_string() {
    let json_str = r##"{"v":"44"}"##;
    let obj = serde_json::from_str::<Test>(json_str).unwrap();

    assert_eq!(obj, Test { v: 44 });
  }
}
