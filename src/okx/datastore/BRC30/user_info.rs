use super::*;
use crate::okx::datastore::BRC30::pool_info::Pid;

use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct UserInfo {
  pub pid: Pid,
  pub staked: u128,
  pub reward: u128,
  pub reward_debt: u128,
  pub latest_updated_block: u64,
}

impl UserInfo {
  pub fn default(pid: &Pid) -> Self {
    Self {
      pid:pid.clone(),
      staked:0,
      reward:0,
      reward_debt:0,
      latest_updated_block:0,
    }
  }
}
