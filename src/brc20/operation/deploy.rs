use super::TickType;
use crate::brc20::custom_serde::{TickSerde, U32StringSerde};
use crate::brc20::num::Num;
use crate::brc20::params::*;
use crate::brc20::Error;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Deploy {
  #[serde(rename = "tick", with = "TickSerde")]
  pub tick: TickType,
  #[serde(rename = "max")]
  pub max_supply: Num,
  #[serde(rename = "lim")]
  pub mint_limit: Option<Num>,
  #[serde(rename = "dec", default = "default_decimals", with = "U32StringSerde")]
  pub decimals: u32,
}

impl Deploy {
  pub(super) fn check(&self) -> Result<(), Error> {
    if self.max_supply > *MAXIMUM_SUPPLY.deref() {
      return Err(Error::InvalidMaxSupply(self.max_supply.clone()));
    }
    if self.decimals > MAX_DECIMAL_WIDTH {
      return Err(Error::InvalidDecimals(self.decimals));
    }
    Ok(())
  }

  pub(super) fn reset_decimals(&mut self) {
    self.max_supply.rescale(self.decimals);
  }
}
