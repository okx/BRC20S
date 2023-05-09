use crate::brc20::LedgerRead;
use crate::{brc20::num::Num, InscriptionId};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error<L: LedgerRead> {
  #[error("brc20 error: {0}")]
  BRC20Error(BRC20Error),

  #[error("ledger error: {0}")]
  LedgerError(<L as LedgerRead>::Error),
}

#[derive(Debug, PartialEq, thiserror::Error)]
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
  #[error("tick has been existed: {0}")]
  DuplicateTick(String),

  #[error("tick not found: {0}")]
  TickNotFound(String),

  #[error("invaild mint limit")]
  InvalidMintLimit,

  #[error("tick has been mined out: {0}")]
  TickMintOut(String),

  #[error("balance overflow")]
  BalanceOverflow,

  #[error("invalid brc20 number: {0}")]
  InvalidNum(String),

  #[error("{op} overflow: original: {org}, other: {other}")]
  Overflow { op: String, org: Num, other: Num },

  #[error("invalid decimals {0}")]
  InvalidDecimals(u8),

  #[error("inscribe transfer overflow {0} range: (0, supply]")]
  InscribeTransferOverflow(Num),

  #[error("invalid max supply: {0}")]
  InvalidMaxSupply(Num),

  #[error("invalid tick length: {0}")]
  InvalidTickLen(usize),

  #[error("invalid tick char: {0}")]
  InvalidTickChar(String),

  #[error("insufficient balance")]
  InsufficientBalance,

  #[error("mint amout exceed limit: {0}")]
  MintAmountExceedLimit(String),

  #[error("transferable inscription not found: {0}")]
  TransferableNotFound(InscriptionId),

  #[error("invalid inscribe inscription to coinbase")]
  InscribeToCoinbase,

  #[error("transferable owner not match {0}")]
  TransferableOwnerNotMatch(InscriptionId),
}

impl<L: LedgerRead> From<BRC20Error> for Error<L> {
  fn from(e: BRC20Error) -> Self {
    Self::BRC20Error(e)
  }
}
