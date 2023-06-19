mod deploy;
mod mint;
mod stake;
mod transfer;
mod unstake;
mod passiveunstake;

use super::error::JSONError;
use super::params::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use self::{deploy::Deploy, mint::Mint, stake::Stake, transfer::Transfer, unstake::UnStake,passiveunstake::PassiveUnStake};

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

  #[serde(rename = "passive_unstake")]
  PassiveUnStake(PassiveUnStake),

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
      r##"{{"p":"brc-30","op":"deploy","t":"pool","pid":"a3668daeaa#1f","stake":"btc","earn":"ordi","erate":"10","dmax":"12000000","dec":"18","total":"21000000","only":"1"}}"##
    );

    let reuslt = deserialize_brc30(&json_str);

    assert!(!deserialize_brc30(&json_str).is_err());

    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Deploy(Deploy {
        pool_type: "pool".to_string(),
        pool_id: "a3668daeaa#1f".to_string(),
        stake: "btc".to_string(),
        earn: "ordi".to_string(),
        earn_rate: "10".to_string(),
        distribution_max: "12000000".to_string(),
        decimals: Some("18".to_string()),
        total_supply: Some("21000000".to_string()),
        only: Some("1".to_string()),
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
        "tid": "tid",
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
        tick_id: "tid".to_string(),
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
        "tid": "tid",
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
        tick_id: "tid".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  #[test]
  fn test_json_duplicate_field() {
    let json_str = format!(
      r##"{{
        "p": "brc-30",
        "op": "stake",
        "pid": "pid-1",
        "pid": "pid-2",
        "amt": "amt"
      }}"##
    );
    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Stake(Stake {
        pool_id: "pid-2".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  #[test]
  fn test_json_non_brc30() {
    let json_str = format!(
      r##"{{
        "p": "brc-40",
        "op": "stake",
        "pid": "pid",
        "amt": "amt"
      }}"##
    );
    assert_eq!(deserialize_brc30(&json_str), Err(JSONError::NotBRC30Json))
  }

  #[test]
  fn test_json_non_string() {
    let json_str = format!(
      r##"{{
        "p": "brc-30",
        "op": "stake",
        "pid": "pid",
        "amt": "amt",
      }}"##
    );
    assert_eq!(deserialize_brc30(&json_str), Err(JSONError::InvalidJson))
  }

  #[test]
  fn test_deserialize_case_insensitive() {
    let json_str = format!(
      r##"{{
        "P": "brc-30",
        "OP": "transfer",
        "Pid": "pid",
        "ticK": "tick",
        "amt": "amt"
      }}"##
    );

    assert_eq!(deserialize_brc30(&json_str), Err(JSONError::NotBRC30Json));

    let json_str1 = format!(
      r##"{{
        "p": "brc-30",
        "OP": "transfer",
        "Pid": "pid",
        "ticK": "tick",
        "amt": "amt"
      }}"##
    );

    assert_eq!(
      deserialize_brc30(&json_str1),
      Err(JSONError::ParseOperationJsonError(
        "missing field `op`".to_string()
      ))
    );
  }
}
