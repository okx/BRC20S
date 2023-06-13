use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Mint {
  // Ticker: 4-6 letter identifier of the brc-30
  #[serde(rename = "tick")]
  pub tick: String,

  // 10 letter identifier of the pool id，pool number not included
  #[serde(rename = "pid")]
  pub pool_id: String,

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
      pool_id: "pid".to_string(),
      amount: "amt".to_string(),
    };

    assert_eq!(
      serde_json::to_string(&obj).unwrap(),
      format!(
        r##"{{"tick":"{}","pid":"{}","amt":"{}"}}"##,
        obj.tick, obj.pool_id, obj.amount
      )
    )
  }

  //TODO test
}
