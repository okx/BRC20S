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

  //TODO test
}
