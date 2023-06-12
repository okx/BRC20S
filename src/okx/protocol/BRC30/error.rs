use crate::okx::protocol;
use crate::InscriptionId;
use protocol::BRC30::num::Num;
use serde::{Deserialize, Serialize};

use crate::okx::datastore::BRC30::{BRC30DbReadAPI, BRC30DbReadWriteAPI};

#[derive(Debug, thiserror::Error)]
pub enum Error<L: BRC30DbReadAPI> {
  #[error("BRC30 error: {0}")]
  BRC30Error(BRC30Error),

  #[error("ledger error: {0}")]
  LedgerError(<L as BRC30DbReadAPI>::Error),

  #[error("others: {0}")]
  Others(anyhow::Error),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum JSONError {
  #[error("invalid content type")]
  InvalidContentType,

  #[error("unsupport content type")]
  UnSupportContentType,

  #[error("invalid json string")]
  InvalidJson,

  #[error("not BRC30 json")]
  NotBRC30Json,

  #[error("parse operation json error: {0}")]
  ParseOperationJsonError(String),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error, Deserialize, Serialize)]
pub enum BRC30Error {
  #[error("{op} overflow: original: {org}, other: {other}")]
  Overflow { op: String, org: Num, other: Num },

  #[error("invalid number: {0}")]
  InvalidNum(String),

  #[error("tick invalid supply {0}")]
  InvalidSupply(Num),

  #[error("tick: {0} has been existed")]
  DuplicateTick(String),

  #[error("tick: {0} not found")]
  TickNotFound(String),

  #[error("illegal tick length '{0}'")]
  InvalidTickLen(String),

  #[error("decimals {0} too large")]
  DecimalsTooLarge(u8),

  #[error("invalid integer {0}")]
  InvalidInteger(Num),

  #[error("tick: {0} has been minted")]
  TickMinted(String),

  #[error("tick: {0} mint limit out of range {0}")]
  MintLimitOutOfRange(String, Num),

  #[error("zero amount not allowed")]
  InvalidZeroAmount,

  #[error("amount overflow: {0}")]
  AmountOverflow(Num),

  #[error("insufficient balance: {0} {1}")]
  InsufficientBalance(Num, Num),

  #[error("amount exceed limit: {0}")]
  AmountExceedLimit(Num),

  #[error("transferable inscriptionId not found: {0}")]
  TransferableNotFound(InscriptionId),

  #[error("invalid inscribe to coinbase")]
  InscribeToCoinbase,

  #[error("transferable owner not match {0}")]
  TransferableOwnerNotMatch(InscriptionId),

  /// an InternalError is an error that happens exceed our expect
  /// and should not happen under normal circumstances
  #[error("internal error: {0}")]
  InternalError(String),
}

impl<L: BRC30DbReadAPI> From<BRC30Error> for Error<L> {
  fn from(e: BRC30Error) -> Self {
    Self::BRC30Error(e)
  }
}
