use serde::{Deserialize, Serialize};
use crate::okx::datastore::BRC30::BRC30DataStoreReadOnly;
use crate::okx::protocol::BRC30::Error;

#[derive(Debug, Clone, PartialEq, thiserror::Error, Deserialize, Serialize)]
pub enum RewardError {
  #[error("invalid number: {0}")]
  InvalidNum(String),

  #[error("no stake: pid:{0}")]
  NoStaked(String),

  #[error("calculate overflow")]
  Overflow(),
}

impl<L: BRC30DataStoreReadOnly> From<RewardError> for Error<L> {
  fn from(e: RewardError) -> Self {
    Self::RewardError(e)
  }
}
