use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, thiserror::Error, Deserialize, Serialize)]
pub enum RewardError {
  #[error("invalid number: {0}")]
  InvalidNum(String),

  #[error("no stake: pid:{0}")]
  NoStaked(String),

  #[error("calculate overflow")]
  Overflow(),
}
