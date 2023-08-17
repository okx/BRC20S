mod evm;

use super::{params::*, *};
use crate::{okx::datastore::ord::Action, Inscription};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use self::evm::Evm;

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
  Evm(Evm),
}

impl Operation {
  pub fn op_type(&self) -> OperationType {
    match self {
      Operation::Evm(_) => OperationType::Evm,
    }
  }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(tag = "op")]
enum RawOperation {
  #[serde(rename = "evm")]
  Evm(Evm),
}

pub(crate) fn deserialize_brc0_operation(
  inscription: &Inscription,
  action: &Action,
) -> Result<Operation> {
  let content_body = std::str::from_utf8(inscription.body().ok_or(JSONError::InvalidJson)?)?;
  if content_body.len() < 40 {
    return Err(JSONError::NotBRC0Json.into());
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
  let raw_operation = match deserialize_brc0(content_body) {
    Ok(op) => op,
    Err(e) => {
      return Err(e.into());
    }
  };

  match action {
    Action::New { .. } => match raw_operation {
      RawOperation::Evm(evm) => Ok(Operation::Evm(evm)),
    },
    Action::Transfer => match raw_operation {
      _ => Err(JSONError::NotBRC0Json.into()),
    },
  }
}

fn deserialize_brc0(s: &str) -> Result<RawOperation, JSONError> {
  let value: Value = serde_json::from_str(s).map_err(|_| JSONError::InvalidJson)?;
  if value.get("p") != Some(&json!(PROTOCOL_LITERAL)) {
    return Err(JSONError::NotBRC0Json);
  }

  serde_json::from_value(value).map_err(|e| JSONError::ParseOperationJsonError(e.to_string()))
}
