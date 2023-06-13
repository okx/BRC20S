use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Transfer {
  // 10 letter identifier of the pool id，pool number not included
  #[serde(rename = "pid")]
  pub pool_id: String,

  // Ticker: 4-6 letter identifier of the brc-30
  #[serde(rename = "tick")]
  pub tick: String,

  // Amount to transfer: States the amount of the brc-30 to transfer.
  #[serde(rename = "amt")]
  pub amount: String,
}

#[cfg(test)]
mod tests {
  use super::super::*;
  use super::*;

  #[test]
  fn test_serialize() {
    let obj = Transfer {
      pool_id: "pid".to_string(),
      tick: "tick".to_string(),
      amount: "amt".to_string(),
    };

    assert_eq!(
      serde_json::to_string(&obj).unwrap(),
      format!(
        r##"{{"pid":"{}","tick":"{}","amt":"{}"}}"##,
        obj.pool_id, obj.tick, obj.amount
      )
    )
  }

  #[test]
  fn test_deserialize() {
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

  #[test]
  fn test_loss_require_key() {
    let json_str = format!(
      r##"{{
        "p": "brc-30",
        "op": "transfer",
        "tick": "tick",
        "amt": "amt"
      }}"##
    );

    let reuslt = deserialize_brc30(&json_str);

    assert_eq!(
      deserialize_brc30(&json_str).unwrap_err(),
      JSONError::ParseOperationJsonError("missing field `pid`".to_string())
    );
  }

  #[test]
  fn test_duplicate_key() {
    let json_str = format!(
      r##"{{
        "p": "brc-30",
        "op": "transfer",
        "pid": "pid",
        "tick": "tick-1",
        "tick": "tick-2",
        "amt": "amt"
      }}"##
    );
    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Transfer(Transfer {
        tick: "tick-2".to_string(),
        pool_id: "pid".to_string(),
        amount: "amt".to_string(),
      })
    );
  }
}
