use std::fmt::format;
use std::str::FromStr;
use bitcoin::hashes::{Hash, sha256};
use bitcoin::hashes::hex::ToHex;
use bitcoin::util::base58::from;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use crate::okx::datastore::BRC20::Tick;
use crate::okx::datastore::BRC30::{BRC30Tick, Pid, PledgedTick, PoolType, TickId};
use crate::okx::protocol::BRC30::params::{FIXED_TYPE, MAX_DECIMAL_WIDTH, NATIVE_TOKEN, PID_BYTE_COUNT, POOL_TYPE,TICK_BYTE_COUNT,TICK_ID_STR_COUNT};
use crate::okx::protocol::BRC30::{BRC30Error, Error, Num, Stake};
use crate::okx::protocol::BRC30::util::validate_pool_str;

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
  pub total_supply: Option<String>,

  // The decimal precision of earn token, default: 18
  #[serde(rename = "dec")]
  pub decimals: Option<String>,

  // Assets only deposit this pool，must be yes
  #[serde(rename = "only")]
  pub only: Option<String>,


}

impl Deploy {
  pub fn new(
    pool_type: String,
    pool_id: String,
    stake: String,
    earn: String,
    earn_rate: String,
    distribution_max: String,
    total_supply: Option<String>,
    only: Option<String>,
    decimals: Option<String>,
  ) -> Self {
    Self {
      pool_type,
      pool_id,
      stake,
      earn,
      earn_rate,
      distribution_max,
      total_supply,
      only,
      decimals,
    }
  }

  pub fn get_pool_type(&self) -> PoolType {
    match self.pool_type.as_str() {
      POOL_TYPE => PoolType::Pool,
      FIXED_TYPE => PoolType::Fixed,
      _ => PoolType::Unknown,
    }
  }

  pub fn get_pool_id(&self) -> Pid {
    Pid::from_str(self.pool_id.as_str()).unwrap()
  }

  pub fn get_stake_id(&self) -> PledgedTick {
    let stake = self.stake.as_str();
    match stake {
      NATIVE_TOKEN => PledgedTick::NATIVE,
      _ => match self.stake.len() {
        TICK_BYTE_COUNT => PledgedTick::BRC20Tick( Tick::from_str(stake).unwrap() ),
        TICK_ID_STR_COUNT => PledgedTick::BRC30Tick( TickId::from_str(stake).unwrap()),
        _ => PledgedTick::UNKNOWN,
      }
    }
  }

  pub fn get_earn_id(&self) -> BRC30Tick {
    return BRC30Tick::from_str(self.earn.as_str()).unwrap()
  }

  pub fn get_earn_rate(&self) -> u128 {
    from_str(self.earn_rate.as_str()).unwrap()
  }

  pub fn get_distribution_max(&self) -> u128 {
    from_str(self.distribution_max.as_str()).unwrap()
  }

  pub fn get_only(&self) -> bool {
    self.only == Some("1".to_string())
  }

  pub fn get_tick_id(&self) -> TickId {
    let tick_str = self.pool_id.as_str().split("#").next().unwrap();
    TickId::from_str(tick_str).unwrap()
  }
  pub fn validate_basic(&self) -> Result<(), BRC30Error> {
    if self.get_pool_type() == PoolType::Unknown {
      return Err(BRC30Error::UnknownPoolType);
    }
    let iserr= validate_pool_str(self.pool_id.as_str()).err();
    if None != iserr  {
      return Err(iserr.unwrap());
    }

    if self.stake.is_empty() {
      return Err(BRC30Error::EmptyParams(self.stake.clone()));
    }

    if self.earn.is_empty() {
      return Err(BRC30Error::EmptyParams(self.earn.clone()));
    }

    if let Some(iserr) = BRC30Tick::from_str(self.earn.as_str()).err() {
        return Err(iserr);
    }

    if let Some(iserr) = Num::from_str(self.earn_rate.as_str()).err()  {
      return Err(BRC30Error::InvalidNum(self.earn_rate.clone()+iserr.to_string().as_str()));
    }
    if let Some(iserr) = Num::from_str(self.distribution_max.as_str()).err()  {
      return Err(BRC30Error::InvalidNum(self.distribution_max.clone()+iserr.to_string().as_str()));
    }

    if let Some(supply) = self.total_supply.as_ref() {
      if let Some(iserr) = Num::from_str(supply.as_str()).err()  {
        return Err(BRC30Error::InvalidNum(supply.to_string() + iserr.to_string().as_str()));
      }
    }

    if let Some(dec) = self.decimals.as_ref() {
      if let Some(iserr) = Num::from_str(dec.as_str()).err()  {
        return Err(BRC30Error::InvalidNum(dec.to_string() + iserr.to_string().as_str()));
      }
    }


    Ok(())
  }
}



#[cfg(test)]
mod tests {
  use super::super::*;
  use super::*;

  #[test]
  fn test_serialize() {
    let obj = Deploy {
      pool_type: "abcd".to_string(),
      pool_id: "a3668daeaa#1f".to_string(),
      stake: "12".to_string(),
      earn: "12".to_string(),
      earn_rate: "12".to_string(),
      distribution_max: "12".to_string(),
      total_supply: Some("12".to_string()),
      decimals: Some("11".to_string()),
      only: Some("1".to_string()),
    };

    assert_eq!(
      serde_json::to_string(&obj).unwrap(),
      format!(
        r##"{{"t":"{}","pid":"{}","stake":"{}","earn":"{}","erate":"{}","dmax":"{}","total":"{}","dec":"{}","only":"{}"}}"##,
        obj.pool_type,
        obj.pool_id,
        obj.stake,
        obj.earn,
        obj.earn_rate,
        obj.distribution_max,
        obj.total_supply.unwrap(),
        obj.decimals.unwrap(),
        obj.only.unwrap(),
      )
    );
  }

  #[test]
  fn test_deserialize() {
    assert_eq!(
      deserialize_brc30(
        r##"{"p":"brc-30","op":"deploy","t":"pool","pid":"a3668daeaa#1f","stake":"btc","earn":"ordi","erate":"10","dmax":"12000000","dec":"18","total":"21000000","only":"1"}"##
      )
        .unwrap(),
      Operation::Deploy(Deploy {
        pool_type: "pool".to_string(),
        pool_id: "a3668daeaa#1f".to_string(),
        stake: "btc".to_string(),
        earn: "ordi".to_string(),
        earn_rate: "10".to_string(),
        distribution_max: "12000000".to_string(),
        decimals: Some("18".to_string()),
        total_supply: Some("21000000".to_string()),
        only: Some("1".to_string()),
      })
    );
  }

  #[test]
  fn test_loss_require_key() {
    assert_eq!(
      deserialize_brc30(r##"{"p":"brc-30","op":"deploy","t":"pool","pid":"a3668daeaa#1f","earn":"ordi","erate":"10","dmax":"12000000","dec":"18","total":"21000000","only":"1"}"##)
        .unwrap_err(),
      JSONError::ParseOperationJsonError("missing field `stake`".to_string())
    );
  }

  #[test]
  fn test_loss_option_key() {
    // loss only
    assert_eq!(
      deserialize_brc30(r##"{"p":"brc-30","op":"deploy","t":"pool","pid":"a3668daeaa#1f","stake":"btc","earn":"ordi","erate":"10","dmax":"12000000","dec":"18","total":"21000000"}"##)
        .unwrap(),
      Operation::Deploy(Deploy {
        pool_type: "pool".to_string(),
        pool_id: "a3668daeaa#1f".to_string(),
        stake: "btc".to_string(),
        earn: "ordi".to_string(),
        earn_rate: "10".to_string(),
        distribution_max: "12000000".to_string(),
        decimals: Some("18".to_string()),
        total_supply: Some("21000000".to_string()),
        only: None,
      })
    );

    // loss dec
    assert_eq!(
      deserialize_brc30(r##"{"p":"brc-30","op":"deploy","t":"pool","pid":"a3668daeaa#1f","stake":"btc","earn":"ordi","erate":"10","dmax":"12000000","dec":"18","only":"1"}"##)
        .unwrap(),
      Operation::Deploy(Deploy {
        pool_type: "pool".to_string(),
        pool_id: "a3668daeaa#1f".to_string(),
        stake: "btc".to_string(),
        earn: "ordi".to_string(),
        earn_rate: "10".to_string(),
        distribution_max: "12000000".to_string(),
        decimals: Some("18".to_string()),
        total_supply: None,
        only: Some("1".to_string()),
      })
    );

    // loss all option
    assert_eq!(
      deserialize_brc30(r##"{"p":"brc-30","op":"deploy","t":"pool","pid":"a3668daeaa#1f","stake":"btc","earn":"ordi","erate":"10","dmax":"12000000","total":"21000000","only":"1"}"##).unwrap(),
      Operation::Deploy(Deploy {
        pool_type: "pool".to_string(),
        pool_id: "a3668daeaa#1f".to_string(),
        stake: "btc".to_string(),
        earn: "ordi".to_string(),
        earn_rate: "10".to_string(),
        distribution_max: "12000000".to_string(),
        decimals: None,
        total_supply: Some("21000000".to_string()),
        only: Some("1".to_string()),
      })
    );
  }

  #[test]
  fn test_duplicate_key() {
    let json_str = r##"{"p":"brc-30","op":"deploy","t":"pool","pid":"a3668daeaa#1f","stake":"btc","earn":"ordi","erate":"10","dmax":"12000000","dec":"18","dec":"20","total":"21000000","only":"1"}"##;
    assert_eq!(
      deserialize_brc30(json_str).unwrap(),
      Operation::Deploy(Deploy {
        pool_type: "pool".to_string(),
        pool_id: "a3668daeaa#1f".to_string(),
        stake: "btc".to_string(),
        earn: "ordi".to_string(),
        earn_rate: "10".to_string(),
        distribution_max: "12000000".to_string(),
        decimals: Some("20".to_string()),
        total_supply: Some("21000000".to_string()),
        only: Some("1".to_string()),
      })
    );
  }

  #[test]
  fn test_validate_basics() {
    let deploy = Deploy {
      pool_type: "pool".to_string(),
      pool_id: "a3668daeaa#1f".to_string(),
      stake: "btc".to_string(),
      earn: "ordi".to_string(),
      earn_rate: "10".to_string(),
      distribution_max: "12000000".to_string(),
      decimals: Some("18".to_string()),
      total_supply: Some("21000000".to_string()),
      only: Some("1".to_string()),
    };
    assert_eq!((),deploy.validate_basic().map_err(|e| { println!("{}", e);e }).unwrap());


    let deploy = Deploy {
      pool_type: "pool".to_string(),
      pool_id: "a668daeaa#1f".to_string(),
      stake: "btc".to_string(),
      earn: "ordi".to_string(),
      earn_rate: "10".to_string(),
      distribution_max: "12000000".to_string(),
      decimals: Some("18".to_string()),
      total_supply: Some("21000000".to_string()),
      only: Some("1".to_string()),
    };
    assert_eq!(true,deploy.validate_basic().is_err());


    let deploy = Deploy {
      pool_type: "pool".to_string(),
      pool_id: "a3668daeaa#1".to_string(),
      stake: "btc".to_string(),
      earn: "ordi".to_string(),
      earn_rate: "10".to_string(),
      distribution_max: "12000000".to_string(),
      decimals: Some("18".to_string()),
      total_supply: Some("21000000".to_string()),
      only: Some("1".to_string()),
    };
    assert_eq!(true,deploy.validate_basic().is_err());

    let deploy = Deploy {
      pool_type: "pool".to_string(),
      pool_id: "a3668dae#a#1f".to_string(),
      stake: "btc".to_string(),
      earn: "ordi".to_string(),
      earn_rate: "10".to_string(),
      distribution_max: "12000000".to_string(),
      decimals: Some("18".to_string()),
      total_supply: Some("21000000".to_string()),
      only: Some("1".to_string()),
    };
    assert_eq!(true,deploy.validate_basic().is_err());

    let deploy = Deploy {
      pool_type: "pool".to_string(),
      pool_id: "a3&68daeaa#1f".to_string(),
      stake: "btc".to_string(),
      earn: "ordi".to_string(),
      earn_rate: "10".to_string(),
      distribution_max: "12000000".to_string(),
      decimals: Some("18".to_string()),
      total_supply: Some("21000000".to_string()),
      only: Some("1".to_string()),
    };
    assert_eq!(true,deploy.validate_basic().is_err());

    let deploy = Deploy {
      pool_type: "pool".to_string(),
      pool_id: "a3668daeaa#&f".to_string(),
      stake: "btc".to_string(),
      earn: "ordi".to_string(),
      earn_rate: "10".to_string(),
      distribution_max: "12000000".to_string(),
      decimals: Some("18".to_string()),
      total_supply: Some("21000000".to_string()),
      only: Some("1".to_string()),
    };
    assert_eq!(true,deploy.validate_basic().is_err());
  }

}
