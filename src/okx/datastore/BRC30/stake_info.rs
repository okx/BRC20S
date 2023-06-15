use super::*;
use crate::okx::datastore::BRC30::PledgedTick;
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct StakeInfo {
  pub stake: PledgedTick,
  pub max_share: u128,
  pub total_only: u128,
  pub pids: Vec<Pid>,
}

impl StakeInfo {
  pub fn new(
    pool_ids: &Vec<Pid>,
    stake: &PledgedTick,
    max_share:u128,
    total_only:u128,
  ) -> Self {
    Self {
      stake:stake.clone(),
      pids:pool_ids.clone(),
      max_share:max_share,
      total_only:total_only,
    }
  }
}
