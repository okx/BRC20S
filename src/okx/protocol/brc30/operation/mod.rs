pub mod deploy;
pub mod mint;
pub mod passiveunstake;
pub mod stake;
pub mod transfer;
pub mod unstake;

use super::error::JSONError;
use super::params::*;
use crate::{
  okx::datastore::{brc30::BRC30OperationType, ord::Action},
  Inscription, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use self::{
  deploy::Deploy, mint::Mint, passiveunstake::PassiveUnStake, stake::Stake, transfer::Transfer,
  unstake::UnStake,
};

#[derive(Debug, Clone, PartialEq)]
pub enum BRC30Operation {
  Deploy(Deploy),
  Mint(Mint),
  Stake(Stake),
  UnStake(UnStake),
  PassiveUnStake(PassiveUnStake),
  InscribeTransfer(Transfer),
  Transfer,
}

impl BRC30Operation {
  pub fn op_type(&self) -> BRC30OperationType {
    match self {
      BRC30Operation::Deploy(_) => BRC30OperationType::Deploy,
      BRC30Operation::Mint(_) => BRC30OperationType::Mint,
      BRC30Operation::Stake(_) => BRC30OperationType::Stake,
      BRC30Operation::UnStake(_) => BRC30OperationType::UnStake,
      BRC30Operation::PassiveUnStake(_) => BRC30OperationType::PassiveUnStake,
      BRC30Operation::InscribeTransfer(_) => BRC30OperationType::InscribeTransfer,
      BRC30Operation::Transfer => BRC30OperationType::Transfer,
    }
  }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(tag = "op")]
pub enum Operation {
  #[serde(rename = "deploy")]
  Deploy(Deploy),

  #[serde(rename = "deposit")]
  Stake(Stake),

  #[serde(rename = "mint")]
  Mint(Mint),

  #[serde(rename = "withdraw")]
  UnStake(UnStake),

  #[serde(rename = "passive_withdraw")]
  PassiveUnStake(PassiveUnStake),

  #[serde(rename = "transfer")]
  Transfer(Transfer),
}

pub fn deserialize_brc30_operation(
  inscription: &Inscription,
  action: &Action,
) -> Result<BRC30Operation> {
  let content_body = std::str::from_utf8(inscription.body().ok_or(JSONError::InvalidJson)?)?;
  if content_body.len() < 40 {
    return Err(JSONError::NotBRC30Json.into());
  }

  let content_type = inscription
    .content_type()
    .ok_or(JSONError::InvalidContentType)?;

  if content_type != "text/plain"
    && content_type != "text/plain;charset=utf-8"
    && content_type != "text/plain;charset=UTF-8"
    && content_type != "application/json"
  {
    if !content_type.starts_with("text/plain;") {
      return Err(JSONError::UnSupportContentType.into());
    }
  }

  let raw_operation = match deserialize_brc30(content_body) {
    Ok(op) => op,
    Err(e) => {
      return Err(e.into());
    }
  };

  match action {
    Action::New { .. } => match raw_operation {
      Operation::Deploy(deploy) => Ok(BRC30Operation::Deploy(deploy)),
      Operation::Stake(stake) => Ok(BRC30Operation::Stake(stake)),
      Operation::UnStake(unstake) => Ok(BRC30Operation::UnStake(unstake)),
      Operation::Mint(mint) => Ok(BRC30Operation::Mint(mint)),
      Operation::Transfer(transfer) => Ok(BRC30Operation::InscribeTransfer(transfer)),
      Operation::PassiveUnStake(_) => Err(JSONError::NotBRC30Json.into()),
    },
    Action::Transfer => match raw_operation {
      Operation::Transfer(_) => Ok(BRC30Operation::Transfer),
      _ => Err(JSONError::NotBRC30Json.into()),
    },
  }
}

pub fn deserialize_brc30(s: &str) -> Result<Operation, JSONError> {
  let value: Value = serde_json::from_str(s).map_err(|_| JSONError::InvalidJson)?;
  if value.get("p") != Some(&json!(PROTOCOL_LITERAL)) {
    return Err(JSONError::NotBRC30Json);
  }

  Ok(serde_json::from_value(value).map_err(|e| JSONError::ParseOperationJsonError(e.to_string()))?)
}

#[allow(unused)]
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_deploy_deserialize() {
    let json_str = format!(
      r##"{{"p":"brc20-s","op":"deploy","t":"pool","pid":"a3668daeaa#1f","stake":"btc","earn":"ordi","erate":"10","dmax":"12000000","dec":"18","total":"21000000","only":"1"}}"##
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
        "p": "brc20-s",
        "op": "deposit",
        "pid": "pid",
        "amt": "amt"
      }}"##
    );

    let result = deserialize_brc30(&json_str);

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
        "p": "brc20-s",
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
        "p": "brc20-s",
        "op": "withdraw",
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
        "p": "brc20-s",
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
        "p": "brc20-s",
        "op": "deposit",
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
        "p": "brc20-s",
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
        "P": "brc20-s",
        "OP": "transfer",
        "Pid": "pid",
        "ticK": "tick",
        "amt": "amt"
      }}"##
    );

    assert_eq!(deserialize_brc30(&json_str), Err(JSONError::NotBRC30Json));

    let json_str1 = format!(
      r##"{{
        "p": "brc20-s",
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

  #[test]
  fn test_ignore_non_transfer_brc30() {
    let content_type = "text/plain;charset=utf-8".as_bytes().to_vec();
    assert_eq!(
      deserialize_brc30_operation(
        &Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc20-s","op":"deploy","t":"pool","pid":"a3668daeaa#1f","stake":"btc","earn":"ordi","erate":"10","dmax":"12000000","dec":"18","total":"21000000","only":"1"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        &Action::New{cursed:false,unbound:false},
      )
      .unwrap(),
      BRC30Operation::Deploy(Deploy {
        pool_type: "pool".to_string(),
        pool_id: "a3668daeaa#1f".to_string(),
        stake: "btc".to_string(),
        earn: "ordi".to_string(),
        earn_rate: "10".to_string(),
        distribution_max: "12000000".to_string(),
        decimals: Some("18".to_string()),
        total_supply: Some("21000000".to_string()),
        only: Some("1".to_string()),
      }),
    );

    assert_eq!(
      deserialize_brc30_operation(
        &Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc20-s","op":"deposit","pid":"pool_id","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        &Action::New {
          cursed: false,
          unbound: false
        },
      )
      .unwrap(),
      BRC30Operation::Stake(Stake {
        pool_id: "pool_id".to_string(),
        amount: "12000".to_string()
      })
    );

    assert_eq!(
      deserialize_brc30_operation(
        &Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc20-s","op":"mint","tick":"tick","pid":"pool_id","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        &Action::New {
          cursed: false,
          unbound: false
        },
      )
      .unwrap(),
      BRC30Operation::Mint(Mint {
        tick: "tick".to_string(),
        pool_id: "pool_id".to_string(),
        amount: "12000".to_string()
      })
    );

    assert_eq!(
      deserialize_brc30_operation(
        &Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc20-s","op":"withdraw","pid":"pool_id","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        &Action::New {
          cursed: false,
          unbound: false
        },
      )
      .unwrap(),
      BRC30Operation::UnStake(UnStake {
        pool_id: "pool_id".to_string(),
        amount: "12000".to_string()
      })
    );

    assert!(deserialize_brc30_operation(
      &Inscription::new(
        Some(content_type.clone()),
        Some(
          r##"{"p":"brc-20","op":"deploy","tick":"abcd","max":"12000","lim":"12","dec":"11"}"##
            .as_bytes()
            .to_vec(),
        ),
      ),
      &Action::Transfer,
    )
    .is_err());

    assert!(deserialize_brc30_operation(
      &Inscription::new(
        Some(content_type.clone()),
        Some(
          r##"{"p":"brc20-s","op":"mint","tick":"abcd","amt":"12000"}"##
            .as_bytes()
            .to_vec(),
        ),
      ),
      &Action::Transfer,
    )
    .is_err());

    assert_eq!(
      deserialize_brc30_operation(
        &Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc20-s","op":"transfer","tid":"tick_id","tick":"abcd","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        &Action::Transfer,
      )
      .unwrap(),
      BRC30Operation::Transfer
    );
  }
}
