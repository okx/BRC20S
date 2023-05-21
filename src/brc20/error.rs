use crate::brc20::LedgerRead;
use crate::{brc20::num::Num, InscriptionId};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error<L: LedgerRead> {
  #[error("brc20 error: {0}")]
  BRC20Error(BRC20Error),

  #[error("ledger error: {0}")]
  LedgerError(<L as LedgerRead>::Error),

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

  #[error("not brc20 json")]
  NotBRC20Json,

  #[error("parse operation json error: {0}")]
  ParseOperationJsonError(String),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error, Deserialize, Serialize)]
pub enum BRC20Error {
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
}

impl<L: LedgerRead> From<BRC20Error> for Error<L> {
  fn from(e: BRC20Error) -> Self {
    Self::BRC20Error(e)
  }
}
