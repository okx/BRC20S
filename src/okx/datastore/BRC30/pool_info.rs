use super::*;
use crate::okx::datastore::BRC30::{PledgedTick, TickId};
use crate::okx::protocol::brc30::{params::PID_BYTE_COUNT, BRC30Error};
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
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct PoolInfo {
  pub pid: Pid,        // pid为池子的唯一标识
  pub ptype: PoolType, // 池子类型
  pub inscription_id: InscriptionId,
  pub stake: PledgedTick,         // 质押的代币
  pub erate: u128,                // 每个区块收益数量
  pub minted: u128,               // 已产出数量
  pub staked: u128,               // 已经质押了token的数量
  pub allocated: u128,            // 该池分配的数量
  pub acc_reward_per_share: u128, // 单位share可获取的收益累积
  pub last_update_block: u64,     // 上次更新acc的块高
  pub only: bool,                 // 是否是独占池
}
