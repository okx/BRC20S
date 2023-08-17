use {
  super::{types::ScriptPubkey, *},
  crate::okx::datastore::brc20s,
  utoipa::ToSchema,
};

mod balance;
mod pool;
mod receipt;
mod reward;
mod ticker;
mod transferable;

pub(super) use {balance::*, pool::*, receipt::*, reward::*, ticker::*, transferable::*};

#[derive(Debug, thiserror::Error)]
pub enum BRC20SError {
  #[error("tid must be 10 hex length")]
  IncorrectTickIdFormat,
  #[error("pid must be 13 hex length")]
  IncorrectPidFormat,
  #[error("tid not found")]
  TickIdNotFound,
  #[error("balance not found")]
  BalanceNotFound,
  #[error("receipts not found")]
  ReceiptsNotFound,
  #[error("block receipts not found")]
  BlockReceiptsNotFound,
  #[error("pool info not found")]
  PoolInfoNotFound,
  #[error("stake info not found")]
  StakeInfoNotFound,
  #[error("user info not found")]
  UserInfoNotFound,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::Tick)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Tick {
  /// Id of the ticker.
  pub id: String,
  /// Name of the ticker.
  pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::Stake)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Stake {
  #[serde(rename = "type")]
  /// Type of the ticker. such as "brc20".
  pub type_field: String,
  /// Name of the ticker.
  pub tick: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::Earn)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Earn {
  /// Id of the ticker.
  pub id: String,
  /// Name of the ticker.
  pub name: String,
}
