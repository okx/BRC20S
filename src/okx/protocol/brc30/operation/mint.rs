use crate::okx::datastore::brc30::{BRC30Tick, Pid, TickId};
use crate::okx::protocol::brc30::util::{validate_amount, validate_pool_str};
use crate::okx::protocol::brc30::BRC30Error;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Mint {
  // Ticker: 4-6 letter identifier of the brc20-s
  #[serde(rename = "tick")]
  pub tick: String,

  // 10 letter identifier of the token id + "#" + 2 letter of pool number
  #[serde(rename = "pid")]
  pub pool_id: String,

  // Amount to mint: States the amount of the brc20-s to mint. Has to be less than "lim" above if stated
  #[serde(rename = "amt")]
  pub amount: String,
}

impl Mint {
  pub fn get_pool_id(&self) -> Result<Pid, BRC30Error> {
    Pid::from_str(self.pool_id.as_str())
  }

  pub fn get_tick_id(&self) -> Result<TickId, BRC30Error> {
    let tick_str = self.pool_id.as_str().split("#").next().unwrap_or("");
    TickId::from_str(tick_str)
  }

  pub fn validate_basic(&self) -> Result<(), BRC30Error> {
    if let Some(err) = validate_pool_str(self.pool_id.as_str()).err() {
      return Err(err);
    }

    if let Some(err) = BRC30Tick::from_str(self.tick.as_str()).err() {
      return Err(err);
    }

    //validate tick
    if let Some(err) = BRC30Tick::from_str(self.tick.as_str()).err() {
      return Err(err);
    }

    // validate amount
    validate_amount(self.amount.as_str())?;

    Ok(())
  }
}

#[allow(unused)]
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

  #[test]
  fn test_deserialize() {
    let json_str = format!(
      r##"{{
        "p": "brc20-s",
        "op": "mint",
        "pid": "tid",
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
        pool_id: "tid".to_string(),
        amount: "amt".to_string(),
      })
    );
  }

  #[test]
  fn test_loss_require_key() {
    let json_str = format!(
      r##"{{
        "p": "brc20-s",
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
        "p": "brc20-s",
        "op": "mint",
        "pid": "pid-2",
        "tick": "tick",
        "amt": "amt"
      }}"##
    );
    assert_eq!(
      deserialize_brc30(&json_str).unwrap(),
      Operation::Mint(Mint {
        tick: "tick".to_string(),
        pool_id: "pid-2".to_string(),
        amount: "amt".to_string(),
      })
    );
  }
}
