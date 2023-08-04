pub mod deploy;
pub mod mint;
pub mod passiveunstake;
pub mod stake;
pub mod transfer;
pub mod unstake;

use super::error::JSONError;
use super::params::*;
use crate::{
  okx::datastore::{brc20s::OperationType, ord::Action},
  Inscription, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use self::{
  deploy::Deploy, mint::Mint, passiveunstake::PassiveUnStake, stake::Stake, transfer::Transfer,
  unstake::UnStake,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
  Deploy(Deploy),
  Mint(Mint),
  Stake(Stake),
  UnStake(UnStake),
  PassiveUnStake(PassiveUnStake),
  InscribeTransfer(Transfer),
  Transfer(Transfer),
}

impl Operation {
  pub fn op_type(&self) -> OperationType {
    match self {
      Operation::Deploy(_) => OperationType::Deploy,
      Operation::Mint(_) => OperationType::Mint,
      Operation::Stake(_) => OperationType::Stake,
      Operation::UnStake(_) => OperationType::UnStake,
      Operation::PassiveUnStake(_) => OperationType::PassiveUnStake,
      Operation::InscribeTransfer(_) => OperationType::InscribeTransfer,
      Operation::Transfer(_) => OperationType::Transfer,
    }
  }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(tag = "op")]
pub enum RawOperation {
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

pub(crate) fn deserialize_brc20s_operation(
  inscription: &Inscription,
  action: &Action,
) -> Result<Operation> {
  let content_body = std::str::from_utf8(inscription.body().ok_or(JSONError::InvalidJson)?)?;
  if content_body.len() < 40 {
    return Err(JSONError::NotBRC20SJson.into());
  }

  let content_type = inscription
    .content_type()
    .ok_or(JSONError::InvalidContentType)?;

  if content_type != "text/plain"
    && content_type != "text/plain;charset=utf-8"
    && content_type != "text/plain;charset=UTF-8"
    && content_type != "application/json"
    && !content_type.starts_with("text/plain;")
  {
    return Err(JSONError::UnSupportContentType.into());
  }

  let raw_operation = match deserialize_brc20s(content_body) {
    Ok(op) => op,
    Err(e) => {
      return Err(e.into());
    }
  };

  match action {
    Action::New { .. } => match raw_operation {
      RawOperation::Deploy(deploy) => Ok(Operation::Deploy(deploy)),
      RawOperation::Stake(stake) => Ok(Operation::Stake(stake)),
      RawOperation::UnStake(unstake) => Ok(Operation::UnStake(unstake)),
      RawOperation::Mint(mint) => Ok(Operation::Mint(mint)),
      RawOperation::Transfer(transfer) => Ok(Operation::InscribeTransfer(transfer)),
      RawOperation::PassiveUnStake(_) => Err(JSONError::NotBRC20SJson.into()),
    },
    Action::Transfer => match raw_operation {
      RawOperation::Transfer(transfer) => Ok(Operation::Transfer(transfer)),
      _ => Err(JSONError::NotBRC20SJson.into()),
    },
  }
}

pub fn deserialize_brc20s(s: &str) -> Result<RawOperation, JSONError> {
  let value: Value = serde_json::from_str(s).map_err(|_| JSONError::InvalidJson)?;
  if value.get("p") != Some(&json!(PROTOCOL_LITERAL)) {
    return Err(JSONError::NotBRC20SJson);
  }

  serde_json::from_value(value).map_err(|e| JSONError::ParseOperationJsonError(e.to_string()))
}

#[allow(unused)]
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_deploy_deserialize() {
    let json_str = r##"{"p":"brc20-s","op":"deploy","t":"pool","pid":"a3668daeaa#1f","stake":"btc","earn":"ordi","erate":"10","dmax":"12000000","dec":"18","total":"21000000","only":"1"}"##.to_string();

    let reuslt = deserialize_brc20s(&json_str);

    assert!(deserialize_brc20s(&json_str).is_ok());

    assert_eq!(
      deserialize_brc20s(&json_str).unwrap(),
      RawOperation::Deploy(Deploy {
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
    let json_str = r##"{
        "p": "brc20-s",
        "op": "deposit",
        "pid": "pid",
        "amt": "amt"
      }"##
      .to_string();

    let result = deserialize_brc20s(&json_str);

    assert!(deserialize_brc20s(&json_str).is_ok());

    assert_eq!(
      deserialize_brc20s(&json_str).unwrap(),
      RawOperation::Stake(Stake {
        pool_id: "pid".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  #[test]
  fn test_mint_deserialize() {
    let json_str = r##"{
        "p": "brc20-s",
        "op": "mint",
        "pid": "pid",
        "tick": "tick",
        "amt": "amt"
      }"##
      .to_string();

    let reuslt = deserialize_brc20s(&json_str);

    assert!(deserialize_brc20s(&json_str).is_ok());

    assert_eq!(
      deserialize_brc20s(&json_str).unwrap(),
      RawOperation::Mint(Mint {
        tick: "tick".to_string(),
        pool_id: "pid".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  #[test]
  fn test_unstake_deserialize() {
    let json_str = r##"{
        "p": "brc20-s",
        "op": "withdraw",
        "pid": "pid",
        "amt": "amt"
      }"##
      .to_string();

    let reuslt = deserialize_brc20s(&json_str);

    assert!(deserialize_brc20s(&json_str).is_ok());

    assert_eq!(
      deserialize_brc20s(&json_str).unwrap(),
      RawOperation::UnStake(UnStake {
        pool_id: "pid".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  #[test]
  fn test_transfer_deserialize() {
    let json_str = r##"{
        "p": "brc20-s",
        "op": "transfer",
        "tid": "tid",
        "tick": "tick",
        "amt": "amt"
      }"##
      .to_string();

    let reuslt = deserialize_brc20s(&json_str);

    assert!(deserialize_brc20s(&json_str).is_ok());

    assert_eq!(
      deserialize_brc20s(&json_str).unwrap(),
      RawOperation::Transfer(Transfer {
        tick: "tick".to_string(),
        tick_id: "tid".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  #[test]
  fn test_json_duplicate_field() {
    let json_str = r##"{
        "p": "brc20-s",
        "op": "deposit",
        "pid": "pid-1",
        "pid": "pid-2",
        "amt": "amt"
      }"##
      .to_string();
    assert_eq!(
      deserialize_brc20s(&json_str).unwrap(),
      RawOperation::Stake(Stake {
        pool_id: "pid-2".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  #[test]
  fn test_json_non_brc20s() {
    let json_str = r##"{
        "p": "brc-40",
        "op": "stake",
        "pid": "pid",
        "amt": "amt"
      }"##
      .to_string();
    assert_eq!(deserialize_brc20s(&json_str), Err(JSONError::NotBRC20SJson))
  }

  #[test]
  fn test_json_non_string() {
    let json_str = r##"{
        "p": "brc20-s",
        "op": "stake",
        "pid": "pid",
        "amt": "amt",
      }"##
      .to_string();
    assert_eq!(deserialize_brc20s(&json_str), Err(JSONError::InvalidJson))
  }

  #[test]
  fn test_deserialize_case_insensitive() {
    let json_str = r##"{
        "P": "brc20-s",
        "OP": "transfer",
        "Pid": "pid",
        "ticK": "tick",
        "amt": "amt"
      }"##
      .to_string();

    assert_eq!(deserialize_brc20s(&json_str), Err(JSONError::NotBRC20SJson));

    let json_str1 = r##"{
        "p": "brc20-s",
        "OP": "transfer",
        "Pid": "pid",
        "ticK": "tick",
        "amt": "amt"
      }"##
      .to_string();

    assert_eq!(
      deserialize_brc20s(&json_str1),
      Err(JSONError::ParseOperationJsonError(
        "missing field `op`".to_string()
      ))
    );
  }

  #[test]
  fn test_ignore_non_transfer_brc20s() {
    let content_type = "text/plain;charset=utf-8".as_bytes().to_vec();
    assert_eq!(
      deserialize_brc20s_operation(
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
      }),
    );

    assert_eq!(
      deserialize_brc20s_operation(
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
      Operation::Stake(Stake {
        pool_id: "pool_id".to_string(),
        amount: "12000".to_string()
      })
    );

    assert_eq!(
      deserialize_brc20s_operation(
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
      Operation::Mint(Mint {
        tick: "tick".to_string(),
        pool_id: "pool_id".to_string(),
        amount: "12000".to_string()
      })
    );

    assert_eq!(
      deserialize_brc20s_operation(
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
      Operation::UnStake(UnStake {
        pool_id: "pool_id".to_string(),
        amount: "12000".to_string()
      })
    );

    assert!(deserialize_brc20s_operation(
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

    assert!(deserialize_brc20s_operation(
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
      deserialize_brc20s_operation(
        &Inscription::new(
          Some(content_type),
          Some(
            r##"{"p":"brc20-s","op":"transfer","tid":"tick_id","tick":"abcd","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        &Action::Transfer,
      )
      .unwrap(),
      Operation::Transfer(Transfer {
        tick_id: "tick_id".to_string(),
        tick: "abcd".to_string(),
        amount: "12000".to_string()
      })
    );
  }
}
