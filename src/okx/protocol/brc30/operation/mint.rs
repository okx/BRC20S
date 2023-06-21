use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Mint {
  // Ticker: 4-6 letter identifier of the brc-30
  #[serde(rename = "tick")]
  pub tick: String,

  // 10 letter identifier of the pool id，pool number not included
  #[serde(rename = "tid")]
  pub tick_id: String,

  // Amount to mint: States the amount of the brc-30 to mint. Has to be less than "lim" above if stated
  #[serde(rename = "amt")]
  pub amount: String,
}

#[cfg(test)]
mod tests {
  use super::super::*;
  use super::*;

  #[test]
  fn test_serialize() {
    let obj = Mint {
      tick: "tick".to_string(),
      tick_id: "tid".to_string(),
      amount: "amt".to_string(),
    };

    assert_eq!(
      serde_json::to_string(&obj).unwrap(),
      format!(
        r##"{{"tick":"{}","tid":"{}","amt":"{}"}}"##,
        obj.tick, obj.tick_id, obj.amount
      )
    )
  }

  #[test]
  fn test_deserialize() {
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
  fn test_loss_require_key() {
    let json_str = format!(
      r##"{{
        "p": "brc-30",
        "op": "mint",
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
        "op": "mint",
        "tid": "pid-1",
        "tid": "pid-2",
        "tick": "tick",
        "amt": "amt"
      }}"##
    );
    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Mint(Mint {
        tick: "tick".to_string(),
        tick_id: "tid-2".to_string(),
        amount: "amt".to_string(),
      })
    );
  }
}
