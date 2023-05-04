use serde::{de, Serialize, Deserialize, Deserializer, Serializer};
use crate::brc20::operation::TickType;

pub struct TickSerde;

impl TickSerde {
  pub fn serialize<S>(val: &TickType, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
  {
    let mut s = String::new();
    s.extend(val);
    s.serialize(serializer)
  }


  pub fn deserialize<'de, D>(deserializer: D) -> Result<TickType, D::Error>
    where
      D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    let chars = s.chars().collect::<Vec<_>>();

    let chars: TickType = chars.try_into().map_err(|_|de::Error::custom("invalid tick length"))?;
    if chars.iter().any(|c|!is_valid_tick_char(*c)) {
      return Err(de::Error::custom("invalid tick char"));
    }
    Ok(chars)
  }
}

/// a valid tick char include:
/// 1. alphabet include both upper case and lower case
/// 2. decimal digit 0 - 9
/// 3. punctuation
/// 4. emoji
fn is_valid_tick_char(c: char) -> bool {
  c.is_ascii_alphabetic() || c.is_ascii_digit() || c.is_ascii_punctuation() || unic_emoji_char::is_emoji(c)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, PartialEq, Serialize, Deserialize)]
  struct Test {
    #[serde(with="TickSerde")]
    tick: TickType,
  }

  #[test]
  fn test_tick_serialize() {
    let t = Test {tick: ['a', 'b', '1', 'o']};

    assert_eq!(serde_json::to_string(&t).unwrap(), String::from(r##"{"tick":"ab1o"}"##));
  }

  #[test]
  fn test_tick_deserialize() {
    let json_str = r##"{"tick":"ab1o"}"##;

    assert_eq!(serde_json::from_str::<Test>(json_str).unwrap(), Test {tick: ['a', 'b', '1', 'o']});
  }

  #[test]
  fn test_tick_deserialize_char_validation() {
    // valid chars
    assert_eq!(serde_json::from_str::<Test>(r##"{"tick":"ab1o"}"##).unwrap(), Test {tick: ['a', 'b', '1', 'o']});
    assert_eq!(serde_json::from_str::<Test>(r##"{"tick":"Ab1o"}"##).unwrap(), Test {tick: ['A', 'b', '1', 'o']});
    assert_eq!(serde_json::from_str::<Test>(r##"{"tick":";b1o"}"##).unwrap(), Test {tick: [';', 'b', '1', 'o']});
    assert_eq!(serde_json::from_str::<Test>(r##"{"tick":";b1A"}"##).unwrap(), Test {tick: [';', 'b', '1', 'A']});
    assert_eq!(serde_json::from_str::<Test>(r##"{"tick":";b1😀"}"##).unwrap(), Test {tick: [';', 'b', '1', '😀']});

    // invalid chars
    assert!(serde_json::from_str::<Test>(r##"{"tick":"ab1中"}"##).is_err());
    assert!(serde_json::from_str::<Test>(r##"{"tick":"ab 1"}"##).is_err());
  }
}
