use crate::brc20::error::BRC20Error;
use crate::brc20::params::MAX_DECIMAL_WIDTH;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Num(Decimal);

impl Num {
  pub fn new(num: Decimal) -> Self {
    Self(num)
  }

  pub fn from_str_radix(str: &str, radix: u32) -> Result<Self, BRC20Error> {
    Ok(Self(
      Decimal::from_str_radix(str, radix).map_err(|e| BRC20Error::InvalidNum(e.to_string()))?,
    ))
  }

  pub fn checked_add(&self, other: Num) -> Result<Self, BRC20Error> {
    Ok(Self(self.0.clone().checked_add(other.0).ok_or(
      BRC20Error::Overflow {
        op: String::from("checked_add"),
        org: self.clone(),
        other,
      },
    )?))
  }

  pub fn checked_sub(&self, other: Num) -> Result<Self, BRC20Error> {
    Ok(Self(self.0.clone().checked_sub(other.0).ok_or(
      BRC20Error::Overflow {
        op: String::from("checked_sub"),
        org: self.clone(),
        other,
      },
    )?))
  }

  pub fn checked_mul(&self, other: Num) -> Result<Self, BRC20Error> {
    Ok(Self(self.0.clone().checked_mul(other.0).ok_or(
      BRC20Error::Overflow {
        op: String::from("checked_mul"),
        org: self.clone(),
        other,
      },
    )?))
  }

  pub fn checked_powu(&self, exp: u64) -> Result<Self, BRC20Error> {
    Ok(Self(self.0.clone().checked_powu(exp).ok_or(
      BRC20Error::Overflow {
        op: String::from("checked_powu"),
        org: self.clone(),
        other: Num(Decimal::from_u64(exp).unwrap()),
      },
    )?))
  }

  pub fn checked_to_u8(&self) -> Result<u8, BRC20Error> {
    Ok(self.0.clone().to_u8().ok_or(BRC20Error::Overflow {
      op: String::from("to_u8"),
      org: self.clone(),
      other: Num(Decimal::from_u8(u8::MAX).unwrap()),
    })?)
  }

  pub fn checked_to_u128(&self) -> Result<u128, BRC20Error> {
    Ok(self.0.clone().to_u128().ok_or(BRC20Error::Overflow {
      op: String::from("to_u128"),
      org: self.clone(),
      other: Num(Decimal::from_u128(u128::MAX).unwrap()),
    })?)
  }

  pub fn rescale(&mut self, scale: u32) {
    self.0.rescale(scale)
  }
}

impl From<Decimal> for Num {
  fn from(num: Decimal) -> Self {
    Num(num)
  }
}

impl From<u64> for Num {
  fn from(n: u64) -> Self {
    Num(Decimal::from_u64(n).unwrap())
  }
}

impl From<u128> for Num {
  fn from(n: u128) -> Self {
    Num(Decimal::from_u128(n).unwrap())
  }
}

impl FromStr for Num {
  type Err = BRC20Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let num = Decimal::from_str_radix(s, 10).map_err(|_| BRC20Error::InvalidNum(s.to_string()))?;

    if num.is_sign_negative() {
      return Err(BRC20Error::InvalidNum(s.to_string()));
    }
    if num.scale() > MAX_DECIMAL_WIDTH as u32 {
      return Err(BRC20Error::InvalidNum(s.to_string()));
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
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let s = self.to_string();
    serializer.serialize_str(&s)
  }
}

impl<'de> Deserialize<'de> for Num {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Ok(Self(
      Decimal::from_str(&s).map_err(serde::de::Error::custom)?,
    ))
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
    assert_eq!(
      Num(Decimal::new(1_0000000000_00000001, 18)),
      Num::from_str("1.000000000000000001").unwrap()
    );
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
    assert_eq!(
      Num::from_str("2"),
      Num::from_str("1")
        .unwrap()
        .checked_add(Num::from_str("1").unwrap())
    );
    assert_eq!(
      Num::from_str("2.1"),
      Num::from_str("1")
        .unwrap()
        .checked_add(Num::from_str("1.1").unwrap())
    );
    assert_eq!(
      Num::from_str("2.1"),
      Num::from_str("1.1")
        .unwrap()
        .checked_add(Num::from_str("1").unwrap())
    );
    assert_eq!(
      Num::from_str("2.222"),
      Num::from_str("1.101")
        .unwrap()
        .checked_add(Num::from_str("1.121").unwrap())
    );

    assert!(Num(Decimal::MAX)
      .checked_add(Num::from_str("1").unwrap())
      .is_err());
  }

  #[test]
  fn test_num_checked_sub() {
    assert_eq!(
      Num::from_str("2"),
      Num::from_str("3")
        .unwrap()
        .checked_sub(Num::from_str("1").unwrap())
    );
    assert_eq!(
      Num::from_str("2.1"),
      Num::from_str("3")
        .unwrap()
        .checked_sub(Num::from_str("0.9").unwrap())
    );
    assert_eq!(
      Num::from_str("2.1"),
      Num::from_str("3.1")
        .unwrap()
        .checked_sub(Num::from_str("1").unwrap())
    );
    assert_eq!(
      Num::from_str("2.222"),
      Num::from_str("3.303")
        .unwrap()
        .checked_sub(Num::from_str("1.081").unwrap())
    );

    assert!(Num(Decimal::MIN)
      .checked_sub(Num::from_str("1").unwrap())
      .is_err());
  }

  #[test]
  fn test_rescale() {
    let mut num = Num::from_str("1.123").unwrap();
    num.rescale(5);
    assert_eq!(num.to_string(), "1.12300");

    let mut num = Num::from_str("1.123").unwrap();
    num.rescale(2);
    assert_eq!(num.to_string(), "1.12");

    let mut num = Num::from_str("1.125").unwrap();
    num.rescale(2);
    assert_eq!(num.to_string(), "1.13");
  }
}
