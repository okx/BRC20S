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
