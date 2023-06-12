use super::*;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

pub const TICK_BYTE_COUNT: usize = 4;
#[derive(Debug, Clone)]
pub struct Tick([u8; TICK_BYTE_COUNT]);

impl FromStr for Tick {
  type Err = BRC20Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let bytes = s.as_bytes();

    if bytes.len() != TICK_BYTE_COUNT {
      return Err(BRC20Error::InvalidTickLen(s.to_string()));
    }
    Ok(Self(bytes.try_into().unwrap()))
  }
}

impl PartialEq for Tick {
  fn eq(&self, other: &Self) -> bool {
    self.to_lowercase().0 == other.to_lowercase().0
  }

  fn ne(&self, other: &Self) -> bool {
    !self.eq(other)
  }
}

impl Tick {
  pub fn as_str(&self) -> &str {
    // NOTE: Tick comes from &str by from_str,
    // so it could be calling unwrap when convert to str
    std::str::from_utf8(self.0.as_slice()).unwrap()
  }

  pub fn to_lowercase(&self) -> Tick {
    Self::from_str(self.as_str().to_lowercase().as_str()).unwrap()
  }

  pub fn as_bytes(&self) -> &[u8] {
    self.0.as_slice()
  }

  pub fn hex(&self) -> String {
    hex::encode(&self.0)
  }

  pub fn min_hex() -> String {
    Self([0u8; TICK_BYTE_COUNT]).hex()
  }

  pub fn max_hex() -> String {
    Self([0xffu8; TICK_BYTE_COUNT]).hex()
  }
}

impl Serialize for Tick {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    self.as_str().serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for Tick {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Self::from_str(&String::deserialize(deserializer)?)
      .map_err(|e| de::Error::custom(format!("deserialize tick error: {}", e)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tick_compare_ignore_case() {
    assert_eq!(Tick::from_str("aBc1"), Tick::from_str("AbC1"));

    assert_ne!(Tick::from_str("aBc1"), Tick::from_str("aBc2"));
  }

  #[test]
  fn test_tick_serialize() {
    let obj = Tick::from_str("Ab1;").unwrap();
    assert_eq!(serde_json::to_string(&obj).unwrap(), r##""Ab1;""##);
  }

  #[test]
  fn test_tick_deserialize() {
    assert_eq!(
      serde_json::from_str::<Tick>(r##""Ab1;""##).unwrap(),
      Tick::from_str("Ab1;").unwrap()
    );
  }
}
