use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Deploy {
  #[serde(rename = "t")]
  pub type_earning: String,

  // 10 letter identifier of the pool id + "#" + 2 letter of pool number
  #[serde(rename = "pid")]
  pub pool_id: String,

  // Ticker: 4 letter identifier of the brc-20,"btc" for special
  #[serde(rename = "stake")]
  pub stake: String,

  // Ticker: 4-6 letter identifier of the brc-30,"btc" for special
  #[serde(rename = "earn")]
  pub earn: String,

  // Distribution rate every seconds
  #[serde(rename = "erate")]
  pub earn_rate: String,

  // Distribution max amounts
  #[serde(rename = "dmax")]
  pub distribution_max: String,

  // Total supply
  #[serde(rename = "total")]
  pub total_supply: String,

  // Assets only deposit this pool，must be yes
  #[serde(rename = "only")]
  pub only: String,

  // The decimal precision of earn token, default: 18
  #[serde(rename = "dec")]
  pub decimals: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::super::*;
  use super::*;

  #[test]
  fn test_serialize() {
    let obj = Deploy {
      type_earning: "abcd".to_string(),
      pool_id: "12000".to_string(),
      stake: "12".to_string(),
      earn: "12".to_string(),
      earn_rate: "12".to_string(),
      distribution_max: "12".to_string(),
      total_supply: "12".to_string(),
      only: "12".to_string(),
      decimals: Some("11".to_string()),
    };

    assert_eq!(
      serde_json::to_string(&obj).unwrap(),
      format!(
        r##"{{"t":"{}","pid":"{}","stake":"{}","earn":"{}","erate":"{}","dmax":"{}","total":"{}","only":"{}","dec":"{}"}}"##,
        obj.type_earning,
        obj.pool_id,
        obj.stake,
        obj.earn,
        obj.earn_rate,
        obj.distribution_max,
        obj.total_supply,
        obj.only,
        obj.decimals.unwrap()
      )
    )
  }

  //TODO test
}
