use crate::okx::datastore::brc20::{BRC20DataStoreReadOnly, BRC20Error};

#[derive(Debug, thiserror::Error)]
pub enum Error<L: BRC20DataStoreReadOnly> {
  #[error("brc20 error: {0}")]
  BRC20Error(BRC20Error),

  #[error("ledger error: {0}")]
  LedgerError(<L>::Error),
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

impl<L: BRC20DataStoreReadOnly> From<BRC20Error> for Error<L> {
  fn from(e: BRC20Error) -> Self {
    Self::BRC20Error(e)
  }
}
