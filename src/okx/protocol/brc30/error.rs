use crate::okx::protocol;
use crate::InscriptionId;
use protocol::brc30::num::Num;
use serde::{Deserialize, Serialize};

use crate::okx::datastore::brc30::BRC30DataStoreReadOnly;

#[derive(Debug, thiserror::Error)]
pub enum Error<L: BRC30DataStoreReadOnly> {
  #[error("brc30 error: {0}")]
  BRC30Error(BRC30Error),

  #[error("ledger error: {0}")]
  LedgerError(<L as BRC30DataStoreReadOnly>::Error),

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

  #[error("not brc30 json")]
  NotBRC30Json,

  #[error("parse operation json error: {0}")]
  ParseOperationJsonError(String),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error, Deserialize, Serialize)]
pub enum BRC30Error {
  #[error("{op} overflow: original: {org}, other: {other}")]
  Overflow { op: String, org: Num, other: Num },

  #[error("in divsion the dived is zero")]
  DivedZero,

  #[error("invalid number: {0}")]
  InvalidNum(String),

  #[error("invalid erate: {0}")]
  InvalidErate(String),

  #[error("tick invalid supply {0}")]
  InvalidSupply(Num),

  #[error("tick: {0} has been existed")]
  DuplicateTick(String),

  #[error("tick: {0} not found")]
  TickNotFound(String),

  #[error("stake: {0} not found")]
  StakeNotFound(String),

  #[error("illegal tick length '{0}'")]
  InvalidTickLen(String),

  #[error("illegal tick id '{0}'")]
  InvalidTickId(String),

  #[error("the prefix:{0} of pool id must be hash(tick_info) which is:{1}")]
  InvalidPoolTickId(String, String),

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

  #[error("dmax:{0} must be less than totoalsupply:{1}")]
  ExceedDmax(String, String),

  #[error("amount exceed limit: {0}")]
  AmountExceedLimit(Num),

  #[error("transferable inscriptionId not found: {0}")]
  TransferableNotFound(InscriptionId),

  #[error("invalid inscribe to coinbase")]
  InscribeToCoinbase,

  #[error("from {0} must equal to to {1}")]
  FromToNotEqual(String, String),

  #[error("pool {0}  only be deployed by {0},but got {2}")]
  DeployerNotEqual(String, String, String),

  #[error("transferable owner not match {0}")]
  TransferableOwnerNotMatch(InscriptionId),

  /// an InternalError is an error that happens exceed our expect
  /// and should not happen under normal circumstances
  #[error("internal error: {0}")]
  InternalError(String),

  #[error("insufficient supply error: {0}")]
  InsufficientTickSupply(String),

  #[error("tick {0} is already exist")]
  TickAlreadyExist(String),

  #[error("tick name {0} is not match")]
  TickNameNotMatch(String),

  #[error("pool {0} is already exist")]
  PoolAlreadyExist(String),

  #[error("pool {0} is not exist")]
  PoolNotExist(String),

  #[error("unknown pool type")]
  UnknownPoolType,

  #[error("illegal pool id '{0}' error: {1}")]
  InvalidPoolId(String, String),

  #[error("illegal hex str error: {0}")]
  InvalidHexStr(String),

  #[error("{0} can not empty")]
  EmptyParams(String),

  #[error("stake {0} has already exist in pool {1}")]
  StakeAlreadyExist(String, String),

  #[error("unknown stake type")]
  UnknownStakeType,

  #[error("no stake: pid:{0}")]
  NoStaked(String),

  #[error("user has staked:{0} > user can staked:{1}")]
  InValidStakeInfo(u128, u128),

  #[error("staked:{0} can not equal to earn:{1}")]
  StakeEqualEarn(String, String),
}

impl<L: BRC30DataStoreReadOnly> From<BRC30Error> for Error<L> {
  fn from(e: BRC30Error) -> Self {
    Self::BRC30Error(e)
  }
}
