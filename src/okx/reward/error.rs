use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, thiserror::Error, Deserialize, Serialize)]
pub enum RewardError {
  #[error("invalid number: {0}")]
  InvalidNum(String),
}
