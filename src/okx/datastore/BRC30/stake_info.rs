use super::*;
use crate::okx::datastore::BRC30::PledgedTick;
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct StakeInfo {
  pub stake: PledgedTick,
  pub total_share: u128,
  pub total_only: u128,
  pub pids: Vec<Pid>,
}
