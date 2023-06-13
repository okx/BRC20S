use super::*;
use crate::okx::datastore::BRC30::{PledgedTick, TickId};
use crate::okx::protocol::BRC30::{params::PID_BYTE_COUNT, BRC30Error};
use crate::InscriptionId;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Pid([u8; PID_BYTE_COUNT]);

impl FromStr for Pid {
  type Err = BRC30Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let bytes = s.as_bytes();

    if bytes.len() != PID_BYTE_COUNT {
      return Err(BRC30Error::InvalidTickLen(s.to_string()));
    }
    Ok(Self(bytes.try_into().unwrap()))
  }
}

impl PartialEq for Pid {
  fn eq(&self, other: &Self) -> bool {
    self.to_lowercase().0 == other.to_lowercase().0
  }

  fn ne(&self, other: &Self) -> bool {
    !self.eq(other)
  }
}

impl Pid {
  pub fn as_str(&self) -> &str {
    // NOTE: Pid comes from &str by from_str,
    // so it could be calling unwrap when convert to str
    std::str::from_utf8(self.0.as_slice()).unwrap()
  }

  pub fn to_lowercase(&self) -> Pid {
    Self::from_str(self.as_str().to_lowercase().as_str()).unwrap()
  }

  pub fn as_bytes(&self) -> &[u8] {
    self.0.as_slice()
  }

  pub fn hex(&self) -> String {
    hex::encode(&self.0)
  }

  pub fn min_hex() -> String {
    Self([0u8; PID_BYTE_COUNT]).hex()
  }

  pub fn max_hex() -> String {
    Self([0xffu8; PID_BYTE_COUNT]).hex()
  }
}

impl Serialize for Pid {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    self.as_str().serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for Pid {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Self::from_str(&String::deserialize(deserializer)?)
      .map_err(|e| de::Error::custom(format!("deserialize tick error: {}", e)))
  }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum PoolType {
  Pool,
  Fixed,
  Unknown,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct PoolInfo {
  pub pid: Pid,
  pub ptype: PoolType,
  pub inscription_id: InscriptionId,
  pub stake: PledgedTick,
  pub erate: u128,
  pub minted: u128,
  pub staked: u128,
  pub allocated: u128,
  pub acc_reward_per_share: u128,
  pub last_update_block: u64,
  pub only: bool,
}

impl PoolInfo {
  pub fn new(
    pid: &Pid,
    ptype: &PoolType,
    inscription_id: &InscriptionId,
    stake: &PledgedTick,
    erate: u128,
    minted: u128,
    staked: u128,
    allocated: u128,
    acc_reward_per_share: u128,
    last_update_block: u64,
    only: bool,
  ) -> Self {
    Self {
      pid:pid.clone(),
      ptype:ptype.clone(),
      inscription_id:inscription_id.clone(),
      stake:stake.clone(),
      erate,
      minted,
      staked,
      allocated,
      acc_reward_per_share,
      last_update_block,
      only,
    }
  }
}
