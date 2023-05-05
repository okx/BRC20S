use crate::brc20::error::BRC20Error;
use crate::brc20::params::TICK_BYTE_COUNT;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tick(String);

impl Tick {
  pub(super) fn check(&self) -> Result<(), BRC20Error> {
    if self.0.as_bytes().len() != TICK_BYTE_COUNT {
      return Err(BRC20Error::InvalidTickLen(self.0.as_bytes().len()));
    }
    if self.0.chars().any(|c| !is_valid_tick_char(c)) {
      return Err(BRC20Error::InvalidTickChar(self.0.clone()));
    }

    Ok(())
  }
}

impl<T: ToString> From<T> for Tick {
  fn from(s: T) -> Self {
    Self(s.to_string())
  }
}

impl Deref for Tick {
  type Target = String;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Tick {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

/// a valid tick char include:
/// 1. alphabet include both upper case and lower case
/// 2. decimal digit 0 - 9
/// 3. punctuation
/// 4. emoji
fn is_valid_tick_char(c: char) -> bool {
  c.is_ascii_alphabetic()
    || c.is_ascii_digit()
    || c.is_ascii_punctuation()
    || unic_emoji_char::is_emoji(c)
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde::de::Unexpected::Str;

  #[test]
  fn test_tick_serialize() {
    assert_eq!(
      serde_json::to_string(&Tick(String::from("ab1o"))).unwrap(),
      String::from(r##""ab1o""##)
    );
  }

  #[test]
  fn test_tick_deserialize() {
    assert_eq!(
      serde_json::from_str::<Tick>(r##""ab1o""##).unwrap(),
      Tick(String::from("ab1o"))
    );
  }

  #[test]
  fn test_tick_deserialize_char_validation() {
    // valid chars
    let tick = serde_json::from_str::<Tick>(r##""ab1o""##).unwrap();
    assert_eq!(tick, Tick(String::from("ab1o")));
    assert!(tick.check().is_ok());

    let tick = serde_json::from_str::<Tick>(r##""Ab1o""##).unwrap();
    assert_eq!(tick, Tick(String::from("Ab1o")));
    assert!(tick.check().is_ok());

    let tick = serde_json::from_str::<Tick>(r##"";b1o""##).unwrap();
    assert_eq!(tick, Tick(String::from(";b1o")));
    assert!(tick.check().is_ok());

    let tick = serde_json::from_str::<Tick>(r##"";b1A""##).unwrap();
    assert_eq!(tick, Tick(String::from(";b1A")));
    assert!(tick.check().is_ok());

    let tick = serde_json::from_str::<Tick>(r##""😀""##).unwrap();
    assert_eq!(tick, Tick(String::from("😀")));
    assert!(tick.check().is_ok());

    // invalid chars
    let tick = serde_json::from_str::<Tick>(r##""b中""##).unwrap();
    assert_eq!(tick, Tick(String::from("b中")));
    assert_eq!(
      tick.check().unwrap_err(),
      BRC20Error::InvalidTickChar(String::from("b中"))
    );

    let tick = serde_json::from_str::<Tick>(r##""ab 1""##).unwrap();
    assert_eq!(tick, Tick(String::from("ab 1")));
    assert_eq!(
      tick.check().unwrap_err(),
      BRC20Error::InvalidTickChar(String::from("ab 1"))
    );

    let tick = serde_json::from_str::<Tick>(r##""abcde""##).unwrap();
    assert_eq!(tick, Tick(String::from("abcde")));
    assert_eq!(tick.check().unwrap_err(), BRC20Error::InvalidTickLen(5));
  }
}
