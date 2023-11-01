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
#[serde(rename_all = "camelCase")]
#[schema(as = brc20s::StakeValue)]
pub(crate) struct StakeValue {
  #[serde(rename = "type")]
  type_field: String,
  tick: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::Stake)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Stake {
  /// Stake BRC20 Ticker.
  #[schema(value_type = brc20s::StakeValue)]
  #[serde(untagged)]
  BRC20(StakeValue),
}

impl From<brc20s::PledgedTick> for Stake {
  fn from(pledged_tick: brc20s::PledgedTick) -> Self {
    match pledged_tick {
      brc20s::PledgedTick::BRC20Tick(_) => Self::BRC20(StakeValue {
        type_field: pledged_tick.to_type(),
        tick: pledged_tick.to_string(),
      }),
      _ => unreachable!(),
    }
  }
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

#[cfg(test)]
mod tests {
  use crate::okx::datastore::brc20::Tick;

  use super::*;

  #[test]
  fn test_stake() {
    let stake = Stake::from(brc20s::PledgedTick::BRC20Tick(
      Tick::from_str("ordi").unwrap(),
    ));
    assert_eq!(
      serde_json::to_string(&stake).unwrap(),
      r#"{"type":"BRC20","tick":"ordi"}"#
    );
  }
}
