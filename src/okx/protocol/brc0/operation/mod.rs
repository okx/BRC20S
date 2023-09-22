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

#[allow(unused)]
#[cfg(test)]
mod tests {
  use std::f32::consts::E;

  use super::deserialize_brc0;

  #[test]
  fn test_deserialize_brc0() {
    let str = r##"{"p":"brc-zero","op":"evm","d":{"gas":"0x2faf080","gasPrice":"0x1","input":"0x608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80632e64cec11461003b5780636057361d14610059575b600080fd5b610043610075565b60405161005091906100a1565b60405180910390f35b610073600480360381019061006e91906100ed565b61007e565b005b60008054905090565b8060008190555050565b6000819050919050565b61009b81610088565b82525050565b60006020820190506100b66000830184610092565b92915050565b600080fd5b6100ca81610088565b81146100d557600080fd5b50565b6000813590506100e7816100c1565b92915050565b600060208284031215610103576101026100bc565b5b6000610111848285016100d8565b9150509291505056fea2646970667358221220322c78243e61b783558509c9cc22cb8493dde6925aa5e89a08cdf6e22f279ef164736f6c63430008120033","nonce":"0x1","to":null,"value":"0x0","v":"0xa9","r":"0x491363cd27b37a89e8241996cc54e96c3fb024c3f8ab7dce738c8441440afeb","s":"0x71fd6579c3d76760e393f7981a8818d6736130fa81d32653530d566b7964e1e3"}}"##;
    match deserialize_brc0(str) {
      Ok(r) => {
        println!("result{:?}", r)
      }
      Err(e) => {
        print!("err:{}", e)
      }
    }
  }
}
