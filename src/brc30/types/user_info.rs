use super::*;
use crate::brc30::types::pool_info::Pid;
use crate::InscriptionId;
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct UserInfo {
  pub pid: Pid,
  pub staked: u128,              // 当前质押的数量
  pub reward: u128,              // 收益
  pub reward_debt: u128,         // 收益债务
  pub latest_updated_block: u64, // 最后一次更新池的块高
}
