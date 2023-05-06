use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Balance {
  pub overall_balance: u128,
  pub transferable_balance: u128,
}

impl Balance {
  pub fn new() -> Self {
    Self {
      overall_balance: 0 as u128,
      transferable_balance: 0 as u128,
    }
  }
}
