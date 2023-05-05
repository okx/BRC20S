use crate::brc20::num::Num;
use crate::brc20::Ledger;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error<L: Ledger> {
  #[error("json parse error: {0}")]
  JSONError(JSONError),

  #[error("brc20 error: {0}")]
  BRC20Error(BRC20Error),

  #[error("ledger error: {0}")]
  LedgerError(<L as Ledger>::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum JSONError {
  #[error("invalid json string")]
  InvalidJson,

  #[error("not brc20 json")]
  NotBRC20Json,

  #[error("parse operation json error: {0}")]
  ParseOperationJsonError(String),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error, Deserialize, Serialize)]
pub enum BRC20Error {
  #[error("invalid brc20 number: {0}")]
  InvalidNum(String),

  #[error("{op} overflow: original: {org}, other: {other}")]
  Overflow { op: String, org: Num, other: Num },

  #[error("invalid decimals {0}")]
  InvalidDecimals(u32),

  #[error("invalid max supply: {0}")]
  InvalidMaxSupply(Num),

  #[error("invalid tick length: {0}")]
  InvalidTickLen(usize),

  #[error("invalid tick char: {0}")]
  InvalidTickChar(String),
}

impl<L: Ledger> From<JSONError> for Error<L> {
  fn from(e: JSONError) -> Self {
    Self::JSONError(e)
  }
}

impl<L: Ledger> From<BRC20Error> for Error<L> {
  fn from(e: BRC20Error) -> Self {
    Self::BRC20Error(e)
  }
}
