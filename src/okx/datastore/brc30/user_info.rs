use super::*;
use crate::okx::datastore::brc30::pool_info::Pid;

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
      pid: pid.clone(),
      staked: 0,
      reward: 0,
      reward_debt: 0,
      latest_updated_block: 0,
    }
  }
}

impl std::fmt::Display for UserInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "UserInfo {{ pid: {}, staked: {}, reward: {},reward_debt: {},latest_updated_block: {}}}",
      self.pid.as_str(),
      self.staked,
      self.reward,
      self.reward_debt,
      self.latest_updated_block,
    )
  }
}
