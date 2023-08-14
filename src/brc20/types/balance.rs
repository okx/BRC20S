use {
  super::*,
  serde::{Deserialize, Serialize},
};
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Balance {
  pub tick: Tick,
  pub overall_balance: u128,
  pub transferable_balance: u128,
}

impl Balance {
  pub fn new(tick: &Tick) -> Self {
    Self {
      tick: tick.clone(),
      overall_balance: 0u128,
      transferable_balance: 0u128,
    }
  }
}
