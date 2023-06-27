use crate::okx::datastore::brc20::Tick;
use crate::okx::datastore::brc30::{PledgedTick, TickId};
use crate::okx::protocol::brc30::params::{NATIVE_TOKEN, TICK_BYTE_COUNT, TICK_ID_STR_COUNT};
use crate::okx::protocol::brc30::{BRC30Error, Num};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct PassiveUnStake {
  // 10 letter identifier of the pool id + "#" + 2 letter of pool number
  #[serde(rename = "stake")]
  pub stake: String,

  // Amount to withdraw: States the amount of the brc-20 to withdraw.
  #[serde(rename = "amt")]
  pub amount: String,
}

impl PassiveUnStake {
  pub fn new(stake: &str, amount: &str) -> Self {
    Self {
      stake: stake.to_string(),
      amount: amount.to_string(),
    }
  }

  pub fn get_stake_tick(&self) -> PledgedTick {
    let stake = self.stake.as_str();
    match stake {
      NATIVE_TOKEN => PledgedTick::Native,
      _ => match self.stake.len() {
        TICK_BYTE_COUNT => PledgedTick::BRC20Tick(Tick::from_str(stake).unwrap()),
        TICK_ID_STR_COUNT => PledgedTick::BRC30Tick(TickId::from_str(stake).unwrap()),
        _ => PledgedTick::Unknown,
      },
    }
  }
  pub fn validate_basics(&self) -> Result<(), BRC30Error> {
    if self.get_stake_tick() == PledgedTick::Unknown {
      return Err(BRC30Error::UnknownStakeType);
    }

    if let Some(iserr) = Num::from_str(self.amount.as_str()).err() {
      return Err(BRC30Error::InvalidNum(
        self.amount.clone() + iserr.to_string().as_str(),
      ));
    }
    Ok(())
  }
}
