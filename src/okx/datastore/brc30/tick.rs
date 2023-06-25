use crate::okx::datastore::brc20::Tick;
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::brc30::params::{
  NATIVE_TOKEN, TICK_BYTE_COUNT, TICK_BYTE_MAX_COUNT, TICK_BYTE_MIN_COUNT, TICK_ID_BYTE_COUNT,
  TICK_ID_STR_COUNT,
};
use crate::okx::protocol::brc30::BRC30Error;
use crate::InscriptionId;
use std::mem;

use crate::okx::datastore::brc30::Pid;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct TickId([u8; TICK_ID_BYTE_COUNT]);

impl FromStr for TickId {
  type Err = BRC30Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let bytes = hex::decode(s);
    if bytes.is_err() {
      return Err(BRC30Error::InternalError(bytes.err().unwrap().to_string()));
    }
    let bytes = bytes.unwrap();
    if bytes.len() != TICK_ID_BYTE_COUNT {
      return Err(BRC30Error::InvalidTickLen(s.to_string()));
    }
    Ok(Self(bytes.as_slice().try_into().unwrap()))
  }
}

impl TickId {
  pub fn from_bytes(bytes: &[u8]) -> Result<Self, BRC30Error> {
    if bytes.len() != TICK_ID_BYTE_COUNT {
      return Err(BRC30Error::InvalidTickLen(hex::encode(bytes)));
    }
    Ok(Self(bytes.try_into().unwrap()))
  }
}
impl PartialEq for TickId {
  fn eq(&self, other: &Self) -> bool {
    self.to_lowercase().0 == other.to_lowercase().0
  }

  fn ne(&self, other: &Self) -> bool {
    !self.eq(other)
  }
}

impl TickId {
  pub fn to_lowercase(&self) -> TickId {
    let binding = self.hex().to_lowercase();
    let lowercase = binding.as_str();
    Self::from_str(lowercase).unwrap()
  }

  pub fn as_bytes(&self) -> &[u8] {
    self.0.as_slice()
  }

  pub fn hex(&self) -> String {
    hex::encode(&self.0)
  }

  pub fn min_hex() -> String {
    Self([0u8; TICK_ID_BYTE_COUNT]).hex()
  }

  pub fn max_hex() -> String {
    Self([0xffu8; TICK_ID_BYTE_COUNT]).hex()
  }
}

impl Serialize for TickId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    self.hex().serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for TickId {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Self::from_str(&String::deserialize(deserializer)?)
      .map_err(|e| de::Error::custom(format!("deserialize tick error: {}", e)))
  }
}

#[derive(Debug, Clone)]
pub struct BRC30Tick(Vec<u8>);

impl FromStr for BRC30Tick {
  type Err = BRC30Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let bytes = s.as_bytes();

    let length = bytes.len();
    if length == 0 {
      return Err(BRC30Error::InvalidTickLen("".to_string()));
    }

    if length == NATIVE_TOKEN.len() && s.to_lowercase() == NATIVE_TOKEN {
      return Ok(Self(bytes.try_into().unwrap()));
    }

    if length > TICK_BYTE_MAX_COUNT || length < TICK_BYTE_MIN_COUNT {
      return Err(BRC30Error::InvalidTickLen(s.to_string()));
    }
    Ok(Self(bytes.try_into().unwrap()))
  }
}

impl PartialEq for BRC30Tick {
  fn eq(&self, other: &Self) -> bool {
    self.to_lowercase().0 == other.to_lowercase().0
  }

  fn ne(&self, other: &Self) -> bool {
    !self.eq(other)
  }
}

impl BRC30Tick {
  pub fn as_str(&self) -> &str {
    // NOTE: BRC30Tick comes from &str by from_str,
    // so it could be calling unwrap when convert to str
    std::str::from_utf8(self.0.as_slice()).unwrap()
  }

  pub fn to_lowercase(&self) -> BRC30Tick {
    Self::from_str(self.as_str().to_lowercase().as_str()).unwrap()
  }

  pub fn as_bytes(&self) -> &[u8] {
    self.0.as_slice()
  }

  pub fn hex(&self) -> String {
    hex::encode(&self.0)
  }

  pub fn min_hex() -> String {
    Self(Vec::new()).hex()
  }

  pub fn max_hex() -> String {
    Self(vec![0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8]).hex()
  }
}

impl Serialize for BRC30Tick {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    self.as_str().serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for BRC30Tick {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Self::from_str(&String::deserialize(deserializer)?)
      .map_err(|e| de::Error::custom(format!("deserialize tick error: {}", e)))
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum PledgedTick {
  UNKNOWN,
  NATIVE,
  BRC20Tick(Tick),
  BRC30Tick(TickId),
}

impl PledgedTick {
  pub fn max_hex() -> String {
    const MAX_SIZE: usize = mem::size_of::<PledgedTick>();
    hex::encode([0xffu8; MAX_SIZE])
  }

  pub fn min_hex() -> String {
    const MAX_SIZE: usize = mem::size_of::<PledgedTick>();
    hex::encode([0u8; MAX_SIZE])
  }
  pub fn to_string(&self) -> String {
    match self {
      PledgedTick::UNKNOWN => "UNKNOWN".to_string(),
      PledgedTick::NATIVE => NATIVE_TOKEN.to_string(),
      PledgedTick::BRC20Tick(tick) => tick.as_str().to_string(),
      PledgedTick::BRC30Tick(tickid) => tickid.to_lowercase().hex(),
    }
  }

  pub fn to_type(&self) -> String {
    match self {
      PledgedTick::UNKNOWN => "unknown".to_string(),
      PledgedTick::NATIVE => NATIVE_TOKEN.to_string(),
      PledgedTick::BRC20Tick(tick) => "brc20".to_string(),
      PledgedTick::BRC30Tick(tickid) => "brc20-s".to_string(),
    }
  }

  pub fn from_str(str: &str) -> Self {
    match str {
      NATIVE_TOKEN => PledgedTick::NATIVE,
      _ => match str.len() {
        TICK_BYTE_COUNT => PledgedTick::BRC20Tick(Tick::from_str(str).unwrap()),
        TICK_ID_STR_COUNT => PledgedTick::BRC30Tick(TickId::from_str(str).unwrap()),
        _ => PledgedTick::UNKNOWN,
      },
    }
  }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TickInfo {
  pub tick_id: TickId,
  pub name: BRC30Tick,
  pub inscription_id: InscriptionId,
  pub allocated: u128,
  pub decimal: u8,
  pub circulation: u128,
  pub supply: u128,
  pub deployer: ScriptKey,
  pub deploy_block: u64,
  pub latest_mint_block: u64,
  pub pids: Vec<Pid>,
}

impl TickInfo {
  pub fn new(
    tick_id: TickId,
    name: &BRC30Tick,
    inscription_id: &InscriptionId,
    allocated: u128,
    decimal: u8,
    minted: u128,
    supply: u128,
    deployer: &ScriptKey,
    deploy_block: u64,
    latest_mint_block: u64,
    pids: Vec<Pid>,
  ) -> Self {
    Self {
      tick_id,
      name: name.to_lowercase(),
      inscription_id: inscription_id.clone(),
      allocated,
      decimal,
      circulation: minted,
      supply,
      deployer: deployer.clone(),
      deploy_block,
      latest_mint_block,
      pids,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tickid_compare_ignore_case() {
    assert_eq!(
      TickId::from_str("f7c515d6b7"),
      TickId::from_str("F7c515D6b7")
    );

    assert_ne!(
      TickId::from_str("f7c515d6b7"),
      TickId::from_str("f6c515d6b7")
    );
  }

  #[test]
  fn test_tickid_serialize() {
    let obj = TickId::from_str("f7c515d6b7").unwrap();
    assert_eq!(serde_json::to_string(&obj).unwrap(), r##""f7c515d6b7""##);
  }

  #[test]
  fn test_tickid_deserialize() {
    assert_eq!(
      serde_json::from_str::<TickId>(r##""f7c515d6b7""##).unwrap(),
      TickId::from_str("f7c515d6b7").unwrap()
    );
  }

  #[test]
  fn test_tick_length_case() {
    assert_eq!(
      BRC30Tick::from_str(""),
      Err(BRC30Error::InvalidTickLen("".to_string()))
    );

    assert_eq!(
      BRC30Tick::from_str("1"),
      Err(BRC30Error::InvalidTickLen("1".to_string()))
    );

    assert_eq!(
      BRC30Tick::from_str("12"),
      Err(BRC30Error::InvalidTickLen("12".to_string()))
    );

    assert_eq!(
      BRC30Tick::from_str("123"),
      Err(BRC30Error::InvalidTickLen("123".to_string()))
    );

    assert_eq!(BRC30Tick::from_str("1234"), BRC30Tick::from_str("1234"));
    assert_eq!(BRC30Tick::from_str("12345"), BRC30Tick::from_str("12345"));
    assert_eq!(BRC30Tick::from_str("123456"), BRC30Tick::from_str("123456"));
    assert_eq!(
      BRC30Tick::from_str("1234567"),
      Err(BRC30Error::InvalidTickLen("1234567".to_string()))
    );
  }

  #[test]
  fn test_tick_compare_ignore_case() {
    assert_eq!(BRC30Tick::from_str("aBc1a"), BRC30Tick::from_str("AbC1A"));

    assert_ne!(BRC30Tick::from_str("aBc1D"), BRC30Tick::from_str("aBc2d"));
  }

  #[test]
  fn test_tick_serialize() {
    let obj = TickId::from_str("f7c515d6b7").unwrap();
    assert_eq!(serde_json::to_string(&obj).unwrap(), r##""f7c515d6b7""##);
  }

  #[test]
  fn test_tick_deserialize() {
    assert_eq!(
      serde_json::from_str::<TickId>(r##""f7c515d6b7""##).unwrap(),
      TickId::from_str("f7c515d6b7").unwrap()
    );
  }

  #[test]
  fn test_tick_str() {
    assert_eq!(
      BRC30Tick::from_str("aBc1a").unwrap().as_str(),
      "aBc1a".to_string()
    );

    assert_eq!(
      BRC30Tick::from_str("aBc1a")
        .unwrap()
        .to_lowercase()
        .as_str(),
      "abc1a".to_string()
    );

    assert_eq!(
      BRC30Tick::from_str("aBc1a").unwrap().as_bytes(),
      "aBc1a".as_bytes()
    );

    assert_eq!(
      BRC30Tick::from_str("aBc1a").unwrap().hex(),
      "6142633161".to_string()
    );

    assert_eq!(BRC30Tick::min_hex(), "".to_string());
    assert_eq!(BRC30Tick::max_hex(), "ffffffffffff".to_string());
  }

  #[test]
  fn test_tick_btc() {
    assert_eq!(BRC30Tick::from_str("btc"), BRC30Tick::from_str("btc"));
    assert_eq!(BRC30Tick::from_str("btc"), BRC30Tick::from_str("BTC"));
    assert_eq!(BRC30Tick::from_str("btc"), BRC30Tick::from_str("Btc"));
    assert_ne!(BRC30Tick::from_str("btc123"), BRC30Tick::from_str("btc"));
    assert_eq!(
      serde_json::from_str::<BRC30Tick>(r##""btc""##).unwrap(),
      BRC30Tick::from_str("btc").unwrap()
    );
  }
}
