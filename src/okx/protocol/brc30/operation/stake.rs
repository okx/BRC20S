use crate::okx::datastore::brc30::{Pid, TickId};
use crate::okx::protocol::brc30::params::BIGDECIMAL_TEN;
use crate::okx::protocol::brc30::util::{validate_amount, validate_pool_str};
use crate::okx::protocol::brc30::{BRC30Error, Num};
use bigdecimal::num_bigint::Sign;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Stake {
  // 10 letter identifier of the pool id + "#" + 2 letter of pool number
  #[serde(rename = "pid")]
  pub pool_id: String,

  // Amount to deposit: States the amount of the brc-20 to deposit.
  #[serde(rename = "amt")]
  pub amount: String,
}

impl Stake {
  pub fn new(pool_id: &str, amount: &str) -> Self {
    Self {
      pool_id: pool_id.to_string(),
      amount: amount.to_string(),
    }
  }

  pub fn get_pool_id(&self) -> Pid {
    Pid::from_str(self.pool_id.as_str()).unwrap()
  }

  pub fn get_amount(&self, decimal: u8) -> Result<Option<Num>, BRC30Error> {
    let base = BIGDECIMAL_TEN.checked_powu(decimal as u64)?;
    let mut amt = Num::from_str(self.amount.as_str()).unwrap();

    if amt.scale() > decimal as i64 {
      return Err(BRC30Error::InvalidNum(amt.to_string()));
    }

    amt = amt.checked_mul(&base)?;
    if amt.sign() == Sign::NoSign {
      return Err(BRC30Error::InvalidZeroAmount);
    }

    Ok(Some(amt))
  }

  pub fn validate_basic(&self) -> Result<(), BRC30Error> {
    if let Some(err) = validate_pool_str(self.pool_id.as_str()).err() {
      return Err(err);
    }

    // validate amount
    validate_amount(self.amount.as_str())?;

    Ok(())
  }
  pub fn get_tick_id(&self) -> TickId {
    let tick_str = self.pool_id.as_str().split("#").next().unwrap();
    TickId::from_str(tick_str).unwrap()
  }
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
        "p": "brc20-s",
        "op": "deposit",
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
        "p": "brc20-s",
        "op": "deposit",
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
        "op": "deposit",
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
