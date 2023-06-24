use super::*;
use crate::okx::datastore::brc30::PledgedTick;
use crate::okx::protocol::brc30::{BRC30Error, Num};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct StakeInfo {
  pub stake: PledgedTick,
  pub pool_stakes: Vec<(Pid, bool, u128)>,
  pub max_share: u128,
  pub total_only: u128,
}

impl StakeInfo {
  pub fn new(
    pool_stakes: &Vec<(Pid, bool, u128)>,
    stake: &PledgedTick,
    max_share: u128,
    total_only: u128,
  ) -> Self {
    Self {
      stake: stake.clone(),
      pool_stakes: pool_stakes.clone(),
      max_share,
      total_only,
    }
  }

  pub fn calculate_max_share(&self) -> Result<Num, BRC30Error> {
    let mut staked_max_share = Num::from(0_u128);
    for (_, only, pool_stake) in self.pool_stakes.clone() {
      let current_pool_stake = Num::from(pool_stake);
      if !only && current_pool_stake.gt(&staked_max_share) {
        staked_max_share = current_pool_stake;
      }
    }
    Ok(staked_max_share)
  }
}
