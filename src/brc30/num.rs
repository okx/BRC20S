use crate::brc30::error::BRC30Error;
use crate::brc30::params::MAX_DECIMAL_WIDTH;
use bigdecimal::num_bigint::{BigInt, Sign, ToBigInt};
use bigdecimal::{BigDecimal, One, ToPrimitive};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct Num(BigDecimal);

impl Num {
  pub fn new(num: BigDecimal) -> Self {
    Self(num)
  }

  pub fn checked_add(&self, other: &Num) -> Result<Self, BRC30Error> {
    Ok(Self(self.0.clone() + &other.0))
  }

  pub fn checked_sub(&self, other: &Num) -> Result<Self, BRC30Error> {
    if self.0 < other.0 {
      return Err(BRC30Error::Overflow {
        op: String::from("checked_sub"),
        org: self.clone(),
        other: other.clone(),
      });
    }

    Ok(Self(self.0.clone() - &other.0))
  }

  pub fn checked_mul(&self, other: &Num) -> Result<Self, BRC30Error> {
    Ok(Self(self.0.clone() * &other.0))
  }

  pub fn checked_powu(&self, exp: u64) -> Result<Self, BRC30Error> {
    match exp {
      0 => Ok(Self(BigDecimal::one())),
      1 => Ok(Self(self.0.clone())),
      exp => {
        let mut result = self.0.clone();
        for _ in 1..exp {
          result = result * &self.0;
        }

        Ok(Self(result))
      }
    }
  }

  pub fn checked_to_u8(&self) -> Result<u8, BRC30Error> {
    if !self.0.is_integer() {
      return Err(BRC30Error::InvalidInteger(self.clone()));
    }
    Ok(self.0.clone().to_u8().ok_or(BRC30Error::Overflow {
      op: String::from("to_u8"),
      org: self.clone(),
      other: Self(BigDecimal::from(u8::MAX)),
    })?)
  }

  pub fn sign(&self) -> Sign {
    self.0.sign()
  }

  pub fn scale(&self) -> i64 {
    let (_, scale) = self.0.as_bigint_and_exponent();
    scale
  }

  pub fn checked_to_u128(&self) -> Result<u128, BRC30Error> {
    if !self.0.is_integer() {
      return Err(BRC30Error::InvalidInteger(self.clone()));
    }
    Ok(
      self
        .0
        .to_bigint()
        .ok_or(BRC30Error::InternalError(format!(
          "convert {} to bigint failed",
          self.0
        )))?
        .to_u128()
        .ok_or(BRC30Error::Overflow {
          op: String::from("to_u128"),
          org: self.clone(),
          other: Self(BigDecimal::from(BigInt::from(u128::MAX))), // TODO: change overflow error to others
        })?,
    )
  }
}

impl From<u64> for Num {
  fn from(n: u64) -> Self {
    Self(BigDecimal::from(n))
  }
}

impl From<u128> for Num {
  fn from(n: u128) -> Self {
    Self(BigDecimal::from(BigInt::from(n)))
  }
}

impl FromStr for Num {
  type Err = BRC30Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.starts_with(".") || s.ends_with(".") || s.find(&['e', 'E', '+', '-']).is_some() {
      return Err(BRC30Error::InvalidNum(s.to_string()));
    }
    let num = BigDecimal::from_str(s).map_err(|_| BRC30Error::InvalidNum(s.to_string()))?;

    let (_, scale) = num.as_bigint_and_exponent();
    if scale > MAX_DECIMAL_WIDTH as i64 {
      return Err(BRC30Error::InvalidNum(s.to_string()));
    }

    Ok(Self(num))
  }
}

impl Display for Num {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
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
      BigDecimal::from_str(&s).map_err(serde::de::Error::custom)?,
    ))
  }
}
