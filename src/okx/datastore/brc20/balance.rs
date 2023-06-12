use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Balance {
  pub overall_balance: u128,
  pub transferable_balance: u128,
}

impl Balance {
  pub fn new() -> Self {
    Self {
      overall_balance: 0u128,
      transferable_balance: 0u128,
    }
  }
}
