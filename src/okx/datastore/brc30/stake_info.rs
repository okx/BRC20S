use super::*;
use crate::okx::datastore::brc30::PledgedTick;
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct StakeInfo {
  pub stake: PledgedTick,
  pub pool_stakes: Vec<(Pid, bool, u128)>,
}

impl StakeInfo {
  pub fn new(pool_stakes: &Vec<(Pid, bool, u128)>, stake: &PledgedTick) -> Self {
    Self {
      stake: stake.clone(),
      pool_stakes: pool_stakes.clone(),
    }
  }
}
