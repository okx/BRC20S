mod deploy;
mod mint;
mod stake;
mod transfer;
mod unstake;

use crate::okx::protocol::brc30::params::*;
use crate::okx::protocol::brc30::JSONError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use self::{deploy::Deploy, mint::Mint, stake::Stake, transfer::Transfer, unstake::UnStake};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(tag = "op")]
pub enum Operation {
  #[serde(rename = "deploy")]
  Deploy(Deploy),

  #[serde(rename = "stake")]
  Stake(Stake),

  #[serde(rename = "mint")]
  Mint(Mint),

  #[serde(rename = "unstake")]
  UnStake(UnStake),

  #[serde(rename = "transfer")]
  Transfer(Transfer),
}

pub fn deserialize_brc30(s: &str) -> Result<Operation, JSONError> {
  let value: Value = serde_json::from_str(s).map_err(|_| JSONError::InvalidJson)?;
  if value.get("p") != Some(&json!(PROTOCOL_LITERAL)) {
    return Err(JSONError::NotBRC30Json);
  }

  Ok(serde_json::from_value(value).map_err(|e| JSONError::ParseOperationJsonError(e.to_string()))?)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_deploy_deserialize() {
    let json_str = format!(
      r##"{{
  "p": "brc-30",
  "op": "deploy",
  "t": "type_earning",
  "pid": "pid",
  "stake": "stake",
  "earn": "earn",
  "erate": "earn_rate",
  "dmax": "distribution_max",
  "total": "total_supply",
  "only": "only",
  "dec": "decimals"
}}"##
    );

    let reuslt = deserialize_brc30(&json_str);

    assert!(!deserialize_brc30(&json_str).is_err());

    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Deploy(Deploy {
        pool_type: "type_earning".to_string(),
        pool_id: "pid".to_string(),
        stake: "stake".to_string(),
        earn: "earn".to_string(),
        earn_rate: "earn_rate".to_string(),
        distribution_max: "distribution_max".to_string(),
        total_supply: "total_supply".to_string(),
        only: "only".to_string(),
        decimals: Some("decimals".to_string()),
      })
    );
  }

  #[test]
  fn test_stake_deserialize() {
    let json_str = format!(
      r##"{{
  "p": "brc-30",
  "op": "stake",
  "pid": "pid",
  "amt": "amt"
}}"##
    );

    let reuslt = deserialize_brc30(&json_str);

    assert!(!deserialize_brc30(&json_str).is_err());

    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Stake(Stake {
        pool_id: "pid".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  #[test]
  fn test_mint_deserialize() {
    let json_str = format!(
      r##"{{
  "p": "brc-30",
  "op": "mint",
  "pid": "pid",
  "tick": "tick",
  "amt": "amt"
}}"##
    );

    let reuslt = deserialize_brc30(&json_str);

    assert!(!deserialize_brc30(&json_str).is_err());

    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Mint(Mint {
        tick: "tick".to_string(),
        pool_id: "pid".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  #[test]
  fn test_unstake_deserialize() {
    let json_str = format!(
      r##"{{
  "p": "brc-30",
  "op": "unstake",
  "pid": "pid",
  "amt": "amt"
}}"##
    );

    let reuslt = deserialize_brc30(&json_str);

    assert!(!deserialize_brc30(&json_str).is_err());

    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::UnStake(UnStake {
        pool_id: "pid".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  #[test]
  fn test_transfer_deserialize() {
    let json_str = format!(
      r##"{{
  "p": "brc-30",
  "op": "transfer",
  "pid": "pid",
  "tick": "tick",
  "amt": "amt"
}}"##
    );

    let reuslt = deserialize_brc30(&json_str);

    assert!(!deserialize_brc30(&json_str).is_err());

    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Transfer(Transfer {
        tick: "tick".to_string(),
        pool_id: "pid".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  //TODO test
}
