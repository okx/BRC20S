mod deploy;
mod mint;
mod transfer;

use super::{params::*, *};
use crate::{okx::datastore::ord::Action, Inscription};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use self::{deploy::Deploy, mint::Mint, transfer::Transfer};

#[derive(Debug, Clone, PartialEq)]
pub enum BRC20Operation {
  Deploy(BRC20Deploy),
  Mint(BRC20Mint),
  InscribeTransfer(BRC20Transfer),
  Transfer,
}

impl BRC20Operation {
  pub fn op_type(&self) -> BRC20OperationType {
    match self {
      BRC20Operation::Deploy(_) => BRC20OperationType::Deploy,
      BRC20Operation::Mint(_) => BRC20OperationType::Mint,
      BRC20Operation::InscribeTransfer(_) => BRC20OperationType::InscribeTransfer,
      BRC20Operation::Transfer => BRC20OperationType::Transfer,
    }
  }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(tag = "op")]
enum RawOperation {
  #[serde(rename = "deploy")]
  Deploy(Deploy),
  #[serde(rename = "mint")]
  Mint(Mint),
  #[serde(rename = "transfer")]
  Transfer(Transfer),
}

pub fn deserialize_brc20_operation(
  inscription: &Inscription,
  action: &Action,
) -> Result<BRC20Operation> {
  let content_body = std::str::from_utf8(inscription.body().ok_or(JSONError::InvalidJson)?)?;
  if content_body.len() < 40 {
    return Err(JSONError::NotBRC20Json.into());
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
  let raw_operation = deserialize_brc20(content_body)?;

  match action {
    Action::New => match raw_operation {
      RawOperation::Deploy(deploy) => Ok(BRC20Operation::Deploy(deploy)),
      RawOperation::Mint(mint) => Ok(BRC20Operation::Mint(mint)),
      RawOperation::Transfer(transfer) => Ok(BRC20Operation::InscribeTransfer(transfer)),
    },
    Action::Transfer => match raw_operation {
      RawOperation::Transfer(_) => Ok(BRC20Operation::Transfer),
      _ => Err(JSONError::NotBRC20Json.into()),
    },
  }
}

fn deserialize_brc20(s: &str) -> Result<RawOperation, JSONError> {
  let value: Value = serde_json::from_str(s).map_err(|_| JSONError::InvalidJson)?;
  if value.get("p") != Some(&json!(PROTOCOL_LITERAL)) {
    return Err(JSONError::NotBRC20Json);
  }

  Ok(serde_json::from_value(value).map_err(|e| JSONError::ParseOperationJsonError(e.to_string()))?)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::ORD::Action;

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
      RawOperation::Deploy(Deploy {
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
      RawOperation::Mint(Mint {
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
      RawOperation::Transfer(Transfer {
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
      RawOperation::Mint(Mint {
        tick: String::from("smol"),
        amount: String::from("33"),
      })
    )
  }

  #[test]
  fn test_json_non_string() {
    let json_str = r##"{"p":"brc-20","op":"mint","tick":"smol","amt":33}"##;
    assert!(deserialize_brc20(json_str).is_err())
  }

  #[test]
  fn test_deserialize_case_insensitive() {
    let max_supply = "21000000".to_string();
    let mint_limit = "1000".to_string();

    let json_str = format!(
      r##"{{
  "P": "brc-20",
  "Op": "deploy",
  "Tick": "ordi",
  "mAx": "{max_supply}",
  "Lim": "{mint_limit}"
}}"##
    );

    assert_eq!(deserialize_brc20(&json_str), Err(JSONError::NotBRC20Json));
  }
  #[test]
  fn test_ignore_non_transfer_brc20() {
    let content_type = "text/plain;charset=utf-8".as_bytes().to_vec();
    assert_eq!(
      deserialize_brc20_operation(
        &Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-20","op":"deploy","tick":"abcd","max":"12000","lim":"12","dec":"11"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        &Action::New,
      )
      .unwrap(),
      BRC20Operation::Deploy(BRC20Deploy {
        tick: "abcd".to_string(),
        max_supply: "12000".to_string(),
        mint_limit: Some("12".to_string()),
        decimals: Some("11".to_string()),
      }),
    );

    assert_eq!(
      deserialize_brc20_operation(
        &Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-20","op":"mint","tick":"abcd","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        &Action::New,
      )
      .unwrap(),
      BRC20Operation::Mint(Mint {
        tick: "abcd".to_string(),
        amount: "12000".to_string()
      })
    );

    assert_eq!(
      deserialize_brc20_operation(
        &Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-20","op":"transfer","tick":"abcd","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        &Action::New,
      )
      .unwrap(),
      BRC20Operation::InscribeTransfer(Transfer {
        tick: "abcd".to_string(),
        amount: "12000".to_string()
      })
    );

    assert!(deserialize_brc20_operation(
      &Inscription::new(
        Some(content_type.clone()),
        Some(
          r##"{"p":"brc-20","op":"deploy","tick":"abcd","max":"12000","lim":"12","dec":"11"}"##
            .as_bytes()
            .to_vec(),
        ),
      ),
      &Action::Transfer
    )
    .is_err());

    assert!(deserialize_brc20_operation(
      &Inscription::new(
        Some(content_type.clone()),
        Some(
          r##"{"p":"brc-20","op":"mint","tick":"abcd","amt":"12000"}"##
            .as_bytes()
            .to_vec(),
        ),
      ),
      &Action::Transfer
    )
    .is_err());

    assert_eq!(
      deserialize_brc20_operation(
        &Inscription::new(
          Some(content_type.clone()),
          Some(
            r##"{"p":"brc-20","op":"transfer","tick":"abcd","amt":"12000"}"##
              .as_bytes()
              .to_vec(),
          ),
        ),
        &Action::Transfer
      )
      .unwrap(),
      BRC20Operation::Transfer
    );
  }
}
