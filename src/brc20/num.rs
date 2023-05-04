use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use once_cell::sync::Lazy;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use super::Error;

/*
#[derive(PartialEq, Debug)]
pub struct Num(u128);

impl FromStr for Num {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut s_iter = s.split(".");
    let integer_part = s_iter.next().ok_or(Error::InvalidNum(s.to_string()))?.trim();
    let decimal_part = s_iter.next().unwrap_or("").trim().trim_end_matches("0");
    if s_iter.next().is_some() {
      // there's more than one '.' in s
      return Err(Error::InvalidNum(s.to_string()));
    }

    let integer_part = u64::from_str_radix(integer_part, 10).map_err(|_|Error::InvalidNum(s.to_string()))?;
    let decimal_part = if decimal_part.len() == 0 {0} else {
      u64::from_str_radix(decimal_part, 10).map_err(|_|Error::InvalidNum(s.to_string()))?
    };

    let x = (integer_part as u128) << 64;
    Ok(Self( x +  decimal_part as u128))
  }
}
 */

const MAX_DECIMAL_WIDTH : u32 = 18;
static MAXIMUM_SUPPLY : Lazy<Decimal> = Lazy::new(||Decimal::from_str_radix("FFFFFFFFFFFFFFFF", 16).unwrap());

#[derive(PartialEq, Debug, Clone)]
pub struct Num(Decimal);

impl Num {
  pub fn checked_add(&self, other: Num) -> Result<Self, Error> {
    Ok(Self(self.0.clone().checked_add(other.0).ok_or(Error::Overflow{op: "checked_add", org: self.clone(), other})?))
  }

  pub fn checked_sub(&self, other: Num) -> Result<Self, Error> {
    Ok(Self(self.0.clone().checked_sub(other.0).ok_or(Error::Overflow {op:"checked_sub", org: self.clone(), other})?))
  }
}

impl FromStr for Num {
  type Err = Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let num = Decimal::from_str_radix(s, 10).map_err(|_|Error::InvalidNum(s.to_string()))?;

    if num.is_sign_negative() {
      return Err(Error::InvalidNum(s.to_string()));
    }
    if num.scale() > MAX_DECIMAL_WIDTH {
      return Err(Error::InvalidNum(s.to_string()));
    }

    Ok(Self(num))
  }
}

impl Display for Num {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}

impl Deref for Num {
  type Target = Decimal;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Num {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl Serialize for Num {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let s = self.to_string();
    serializer.serialize_str(&s)
  }
}

impl<'de> Deserialize<'de> for Num {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
    let s = String::deserialize(deserializer)?;
    Ok(Self(Decimal::from_str(&s).map_err(serde::de::Error::custom)?))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_num_from_str() {
    assert_eq!(Num(Decimal::new(11, 1)), Num::from_str("1.1").unwrap());
    assert_eq!(Num(Decimal::new(11, 1)), Num::from_str("1.1000").unwrap());
    assert_eq!(Num(Decimal::new(101, 2)), Num::from_str("1.01").unwrap());

    // can not be negative
    assert!(Num::from_str("-1.1").is_err());

    // number of decimal fractional can not exceed 18
    assert_eq!(Num(Decimal::new(1_0000000000_00000001, 18)), Num::from_str("1.000000000000000001").unwrap());
    assert!(Num::from_str("1.0000000000000000001").is_err());
  }

  #[test]
  fn test_num_serialize() {
    let num = Num::from_str("1.01").unwrap();
    let json = serde_json::to_string(&num).unwrap();
    assert_eq!(json.as_str(), "\"1.01\"");
  }

  #[test]
  fn test_num_deserialize() {
    let num = serde_json::from_str::<Num>("\"1.11\"").unwrap();
    assert_eq!(Num::from_str("1.11").unwrap(), num);
  }

  #[test]
  fn test_num_checked_add() {
    assert_eq!(Num::from_str("2"), Num::from_str("1").unwrap().checked_add(Num::from_str("1").unwrap()));
    assert_eq!(Num::from_str("2.1"), Num::from_str("1").unwrap().checked_add(Num::from_str("1.1").unwrap()));
    assert_eq!(Num::from_str("2.1"), Num::from_str("1.1").unwrap().checked_add(Num::from_str("1").unwrap()));
    assert_eq!(Num::from_str("2.222"), Num::from_str("1.101").unwrap().checked_add(Num::from_str("1.121").unwrap()));

    assert!(Num(Decimal::MAX).checked_add(Num::from_str("1").unwrap()).is_err());
  }

  #[test]
  fn test_num_checked_sub() {
    assert_eq!(Num::from_str("2"), Num::from_str("3").unwrap().checked_sub(Num::from_str("1").unwrap()));
    assert_eq!(Num::from_str("2.1"), Num::from_str("3").unwrap().checked_sub(Num::from_str("0.9").unwrap()));
    assert_eq!(Num::from_str("2.1"), Num::from_str("3.1").unwrap().checked_sub(Num::from_str("1").unwrap()));
    assert_eq!(Num::from_str("2.222"), Num::from_str("3.303").unwrap().checked_sub(Num::from_str("1.081").unwrap()));

    assert!(Num(Decimal::MIN).checked_sub(Num::from_str("1").unwrap()).is_err());
  }
}

