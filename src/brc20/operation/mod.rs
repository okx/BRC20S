mod deploy;
mod mint;
mod tick;
mod transfer;

use crate::brc20::error::JSONError;
use crate::brc20::params::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use self::{deploy::Deploy, mint::Mint, transfer::Transfer};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "op")]
pub enum Operation {
  #[serde(rename = "deploy")]
  Deploy(Deploy),
  #[serde(rename = "mint")]
  Mint(Mint),
  #[serde(rename = "transfer")]
  Transfer(Transfer),
}

pub fn deserialize_brc20(s: &str) -> Result<Operation, JSONError> {
  let value: Value = serde_json::from_str(s).map_err(|_| JSONError::InvalidJson)?;
  if value.get("p") != Some(&json!(PROTOCOL_LITERAL)) {
    return Err(JSONError::NotBRC20Json);
  }

  Ok(serde_json::from_value(value).map_err(|e| JSONError::ParseOperationJsonError(e.to_string()))?)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_deploy_deserialize() {
    let max_supply = "21000000".to_string();
    let mint_limit = "1000".to_string();

    let json_str = format!(
      r##"{{
  "p": "brc-20",
  "op": "deploy",
  "tick": "ordi",
  "max": "{max_supply}",
  "lim": "{mint_limit}"
}}"##
    );

    assert_eq!(
      deserialize_brc20(&json_str).unwrap(),
      Operation::Deploy(Deploy {
        tick: "ordi".to_string(),
        max_supply,
        mint_limit: Some(mint_limit),
        decimals: None
      })
    );
  }

  #[test]
  fn test_mint_deserialize() {
    let amount = "1000".to_string();

    let json_str = format!(
      r##"{{
  "p": "brc-20",
  "op": "mint",
  "tick": "ordi",
  "amt": "{amount}"
}}"##
    );

    assert_eq!(
      deserialize_brc20(&json_str).unwrap(),
      Operation::Mint(Mint {
        tick: "ordi".to_string(),
        amount,
      })
    );
  }

  #[test]
  fn test_transfer_deserialize() {
    let amount = "100".to_string();

    let json_str = format!(
      r##"{{
  "p": "brc-20",
  "op": "transfer",
  "tick": "ordi",
  "amt": "{amount}"
}}"##
    );

    assert_eq!(
      deserialize_brc20(&json_str).unwrap(),
      Operation::Transfer(Transfer {
        tick: "ordi".to_string(),
        amount,
      })
    );
  }
  #[test]
  fn test_json_duplicate_field() {
    let json_str = r##"{"p":"brc-20","op":"mint","tick":"smol","amt":"333","amt":"33"}"##;
    assert_eq!(
      deserialize_brc20(json_str).unwrap(),
      Operation::Mint(Mint {
        tick: Tick::from("smol"),
        amount: Num::from_str("33").unwrap()
      })
    )
  }
}
