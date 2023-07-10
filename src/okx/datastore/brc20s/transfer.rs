use super::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TransferInfo {
  pub tick_id: TickId,
  pub tick_name: Tick,
  pub amt: u128,
}
