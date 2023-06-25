use super::Num;
use crate::okx::datastore::brc20::{BRC20DataStoreReadOnly, BRC20Error};

#[derive(Debug, thiserror::Error)]
pub enum Error<L: BRC20DataStoreReadOnly> {
  #[error("brc20 error: {0}")]
  BRC20Error(BRC20Error),

  #[error("ledger error: {0}")]
  LedgerError(<L>::Error),

  #[error("brc20 num error: {0}")]
  NumError(NumError),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum JSONError {
  #[error("invalid content type")]
  InvalidContentType,

  #[error("unsupport content type")]
  UnSupportContentType,

  #[error("invalid json string")]
  InvalidJson,

  #[error("not brc20 json")]
  NotBRC20Json,

  #[error("parse operation json error: {0}")]
  ParseOperationJsonError(String),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum NumError {
  #[error("{op} overflow: original: {org}, other: {other}")]
  Overflow { op: String, org: Num, other: Num },

  #[error("invalid integer {0}")]
  InvalidInteger(Num),

  #[error("internal error: {0}")]
  InternalError(String),

  #[error("invalid number: {0}")]
  InvalidNum(String),
}

impl<L: BRC20DataStoreReadOnly> From<BRC20Error> for Error<L> {
  fn from(e: BRC20Error) -> Self {
    Self::BRC20Error(e)
  }
}
impl<L: BRC20DataStoreReadOnly> From<NumError> for Error<L> {
  fn from(e: NumError) -> Self {
    Self::NumError(e)
  }
}
