use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Deploy {
  // Type:Type of earning(pool,fixed)
  // pool: share earning with all pool deposits.
  // fixed: earn solo,and have a fixed rate.
  #[serde(rename = "t")]
  pub pool_type: String,

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
      pool_type: "abcd".to_string(),
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
        obj.pool_type,
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

  #[test]
  fn test_deserialize() {
    let json_str = format!(
      r##"{{
        "p": "brc-30",
        "op": "deploy",
        "t": "type_earning",
        "pid": "pid",
        "stake": "stake",
        "earn": "earn",
        "erate": "earn_rate",
        "dmax": "distribution_max",
        "total": "total_supply",
        "only": "only",
        "dec": "decimals"
      }}"##
    );

    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Deploy(Deploy {
        pool_type: "type_earning".to_string(),
        pool_id: "pid".to_string(),
        stake: "stake".to_string(),
        earn: "earn".to_string(),
        earn_rate: "earn_rate".to_string(),
        distribution_max: "distribution_max".to_string(),
        total_supply: "total_supply".to_string(),
        only: "only".to_string(),
        decimals: Some("decimals".to_string()),
      })
    );
  }

  #[test]
  fn test_loss_require_key() {
    let json_str = format!(
      r##"{{
        "p": "brc-30",
        "op": "deploy",
        "t": "type_earning",
        "pid": "pid",
        "earn": "earn",
        "erate": "earn_rate",
        "dmax": "distribution_max",
        "total": "total_supply",
        "only": "only",
        "dec": "decimals"
      }}"##
    );

    assert_eq!(
      deserialize_brc30(&json_str).unwrap_err(),
      JSONError::ParseOperationJsonError("missing field `stake`".to_string())
    );
  }

  #[test]
  fn test_loss_option_key() {
    let json_str = format!(
      r##"{{
        "p": "brc-30",
        "op": "deploy",
        "t": "type_earning",
        "pid": "pid",
        "stake": "stake",
        "earn": "earn",
        "erate": "earn_rate",
        "dmax": "distribution_max",
        "total": "total_supply",
        "only": "only"
      }}"##
    );

    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Deploy(Deploy {
        pool_type: "type_earning".to_string(),
        pool_id: "pid".to_string(),
        stake: "stake".to_string(),
        earn: "earn".to_string(),
        earn_rate: "earn_rate".to_string(),
        distribution_max: "distribution_max".to_string(),
        total_supply: "total_supply".to_string(),
        only: "only".to_string(),
        decimals: None,
      })
    );
  }

  #[test]
  fn test_duplicate_key() {
    let json_str = format!(
      r##"{{
        "p": "brc-30",
        "op": "deploy",
        "t": "type_earning",
        "pid": "pid",
        "stake": "stake-1",
        "stake": "stake-2",
        "earn": "earn",
        "erate": "earn_rate",
        "dmax": "distribution_max",
        "total": "total_supply",
        "only": "only",
        "dec": "decimals"
      }}"##
    );
    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Deploy(Deploy {
        pool_type: "type_earning".to_string(),
        pool_id: "pid".to_string(),
        stake: "stake-2".to_string(),
        earn: "earn".to_string(),
        earn_rate: "earn_rate".to_string(),
        distribution_max: "distribution_max".to_string(),
        total_supply: "total_supply".to_string(),
        only: "only".to_string(),
        decimals: Some("decimals".to_string()),
      })
    )
  }
}
