use crate::InscriptionId;
use serde::{Deserialize, Serialize};

use crate::okx::datastore::brc20s::DataStoreReadOnly;

#[derive(Debug, thiserror::Error)]
pub enum Error<L: DataStoreReadOnly> {
  #[error("brc20s error: {0}")]
  BRC20SError(BRC20SError),

  #[error("ledger error: {0}")]
  LedgerError(<L as DataStoreReadOnly>::Error),

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

  #[error("not brc20s json")]
  NotBRC20SJson,

  #[error("parse operation json error: {0}")]
  ParseOperationJsonError(String),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error, Deserialize, Serialize)]
pub enum BRC20SError {
  #[error("{op} overflow: original: {org}, other: {other}")]
  Overflow {
    op: String,
    org: String,
    other: String,
  },

  #[error("in divsion the dived is zero")]
  DivedZero,

  #[error("invalid number: {0}")]
  InvalidNum(String),

  #[error("invalid erate: {0}")]
  InvalidErate(String),

  #[error("tick invalid supply {0}")]
  InvalidSupply(String),

  #[error("tick: {0} has been existed")]
  DuplicateTick(String),

  #[error("tick: {0} not found")]
  TickNotFound(String),

  #[error("stake: {0} not found")]
  StakeNotFound(String),

  #[error("tick: {0} has no permission staked")]
  StakeNoPermission(String),

  #[error("share pool can not deploy")]
  ShareNoPermission(),

  #[error("illegal tick length '{0}'")]
  InvalidTickLen(String),

  #[error("illegal tick id '{0}'")]
  InvalidTickId(String),

  #[error("the prefix:{0} of pool id must be hash(tick_info) which is:{1}")]
  InvalidPoolTickId(String, String),

  #[error("decimals {0} too large")]
  DecimalsTooLarge(u8),

  #[error("invalid integer {0}")]
  InvalidInteger(String),

  #[error("tick: {0} has been minted")]
  TickMinted(String),

  #[error("tick: {0} mint limit out of range {0}")]
  MintLimitOutOfRange(String, String),

  #[error("zero amount not allowed")]
  InvalidZeroAmount,

  #[error("amount overflow: {0}")]
  AmountOverflow(String),

  #[error("insufficient balance: {0} {1}")]
  InsufficientBalance(String, String),

  #[error("dmax:{0} must be less than totoalsupply:{1}")]
  ExceedDmax(String, String),

  #[error("amount exceed limit: {0}")]
  AmountExceedLimit(String),

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

impl<L: DataStoreReadOnly> From<BRC20SError> for Error<L> {
  fn from(e: BRC20SError) -> Self {
    Self::BRC20SError(e)
  }
}
