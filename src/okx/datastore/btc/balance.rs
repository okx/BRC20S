use super::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Balance {
  pub balance: u64,
}

impl Balance {
  pub fn new() -> Self {
    Self {
      balance: 0u64,
    }
  }
}