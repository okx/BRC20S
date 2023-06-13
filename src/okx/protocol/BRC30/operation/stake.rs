use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Stake {
  // 10 letter identifier of the pool id + "#" + 2 letter of pool number
  #[serde(rename = "pid")]
  pub pool_id: String,

  // Amount to deposit: States the amount of the brc-20 to deposit.
  #[serde(rename = "amt")]
  pub amount: String,
}

#[cfg(test)]
mod tests {
  use super::super::*;
  use super::*;

  #[test]
  fn test_serialize() {
    let obj = Stake {
      pool_id: "pid".to_string(),
      amount: "amt".to_string(),
    };

    assert_eq!(
      serde_json::to_string(&obj).unwrap(),
      format!(r##"{{"pid":"{}","amt":"{}"}}"##, obj.pool_id, obj.amount)
    )
  }

  #[test]
  fn test_deserialize() {
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
  fn test_loss_require_key() {
    let json_str = format!(
      r##"{{
        "p": "brc-30",
        "op": "stake",
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
}
