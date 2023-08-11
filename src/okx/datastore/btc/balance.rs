use super::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Balance {
  pub overall_balance: u64,
}

impl Balance {
  pub fn new() -> Self {
    Self {
      overall_balance: 0u64,
    }
  }
}