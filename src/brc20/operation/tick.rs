use std::ops::{Deref, DerefMut};
use crate::brc20::error::BRC20Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tick(String);

impl Tick {
  pub(super) fn check(&self) -> Result<(), BRC20Error> {
    if self.0.chars().any(|c| !is_valid_tick_char(c)) {
      Err(BRC20Error::InvalidTick(self.0.clone()))
    } else {
      Ok(())
    }
  }
}

impl<T:ToString> From<T> for Tick {
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

  #[test]
  fn test_tick_serialize() {
    assert_eq!(
      serde_json::to_string(&Tick(String::from("ab1o"))).unwrap(),
      String::from(r##"{"ab1o"}"##)
    );
  }

  #[test]
  fn test_tick_deserialize() {
    assert_eq!(
      serde_json::from_str::<Tick>(r##"{"ab1o"}"##).unwrap(),
      Tick(String::from("ab1o"))
    );
  }

  #[test]
  fn test_tick_deserialize_char_validation() {
    // valid chars
    assert_eq!(
      serde_json::from_str::<Tick>(r##"{"ab1o"}"##).unwrap(),
      Tick(String::from("ab1o"))
    );
    assert_eq!(
      serde_json::from_str::<Tick>(r##"{"Ab1o"}"##).unwrap(),
      Tick(String::from("Ab1o"))
    );
    assert_eq!(
      serde_json::from_str::<Tick>(r##"{";b1o"}"##).unwrap(),
      Tick(String::from(";b1o"))
    );
    assert_eq!(
      serde_json::from_str::<Tick>(r##"{";b1A"}"##).unwrap(),
      Tick(String::from(";b1A"))
    );
    assert_eq!(
      serde_json::from_str::<Tick>(r##"{";b1😀"}"##).unwrap(),
      Tick(String::from(r##"{";b1😀"}"##))
    );

    // invalid chars
    assert!(serde_json::from_str::<Tick>(r##"{"ab1中"}"##).is_err());
    assert!(serde_json::from_str::<Tick>(r##"{"ab 1"}"##).is_err());
  }
}
