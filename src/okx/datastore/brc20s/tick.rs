use crate::okx::datastore::brc20;
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::brc20s::params::{
  NATIVE_TOKEN, TICK_BYTE_COUNT, TICK_BYTE_MAX_COUNT, TICK_BYTE_MIN_COUNT, TICK_ID_BYTE_COUNT,
  TICK_ID_STR_COUNT,
};
use crate::okx::protocol::brc20s::BRC20SError;
use crate::InscriptionId;
use std::mem;

use crate::okx::datastore::brc20s::Pid;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickId([u8; TICK_ID_BYTE_COUNT]);

impl FromStr for TickId {
  type Err = BRC20SError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let bytes = hex::decode(s.to_lowercase().as_str());
    if bytes.is_err() {
      return Err(BRC20SError::InternalError(bytes.err().unwrap().to_string()));
    }
    let bytes = bytes.unwrap();
    if bytes.len() != TICK_ID_BYTE_COUNT {
      return Err(BRC20SError::InvalidTickLen(s.to_string()));
    }
    Ok(Self(bytes.as_slice().try_into().unwrap()))
  }
}

impl From<Pid> for TickId {
  fn from(pid: Pid) -> Self {
    TickId::from_str(&pid.as_str().to_string()[..TICK_ID_STR_COUNT]).unwrap()
  }
}

impl TickId {
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
pub struct Tick(Vec<u8>);

impl FromStr for Tick {
  type Err = BRC20SError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let bytes = s.as_bytes();

    let length = bytes.len();
    if length == 0 {
      return Err(BRC20SError::InvalidTickLen("".to_string()));
    }

    if length == NATIVE_TOKEN.len() && s.to_lowercase() == NATIVE_TOKEN {
      return Ok(Self(bytes.try_into().unwrap()));
    }

    if length > TICK_BYTE_MAX_COUNT || length < TICK_BYTE_MIN_COUNT {
      return Err(BRC20SError::InvalidTickLen(s.to_string()));
    }
    Ok(Self(bytes.try_into().unwrap()))
  }
}

impl PartialEq for Tick {
  fn eq(&self, other: &Self) -> bool {
    self.to_lowercase().0 == other.to_lowercase().0
  }

  fn ne(&self, other: &Self) -> bool {
    !self.eq(other)
  }
}

impl Tick {
  pub fn as_str(&self) -> &str {
    // NOTE: Tick comes from &str by from_str,
    // so it could be calling unwrap when convert to str
    std::str::from_utf8(self.0.as_slice()).unwrap()
  }

  pub fn to_lowercase(&self) -> Tick {
    Self::from_str(self.as_str().to_lowercase().as_str()).unwrap()
  }

  #[allow(dead_code)]
  pub fn as_bytes(&self) -> &[u8] {
    self.0.as_slice()
  }

  pub fn hex(&self) -> String {
    hex::encode(&self.0)
  }

  #[allow(dead_code)]
  pub fn min_hex() -> String {
    Self(Vec::new()).hex()
  }

  #[allow(dead_code)]
  pub fn max_hex() -> String {
    Self(vec![0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8]).hex()
  }
}

impl Serialize for Tick {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    self.as_str().serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for Tick {
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
  Unknown,
  Native,
  BRC20Tick(brc20::Tick),
  BRC20STick(TickId),
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
      PledgedTick::Unknown => "Unknown".to_string(),
      PledgedTick::Native => NATIVE_TOKEN.to_string(),
      PledgedTick::BRC20Tick(tick) => tick.as_str().to_string(),
      PledgedTick::BRC20STick(tickid) => tickid.hex(),
    }
  }

  pub fn to_type(&self) -> String {
    match self {
      PledgedTick::Unknown => "Unknown".to_string(),
      PledgedTick::Native => NATIVE_TOKEN.to_uppercase().to_string(),
      PledgedTick::BRC20Tick(_) => "BRC20".to_string(),
      PledgedTick::BRC20STick(_) => "BRC20-S".to_string(),
    }
  }

  pub fn from_str(str: &str) -> Self {
    match str {
      NATIVE_TOKEN => PledgedTick::Native,
      _ => match str.len() {
        TICK_BYTE_COUNT => PledgedTick::BRC20Tick(brc20::Tick::from_str(str).unwrap()),
        TICK_ID_STR_COUNT => PledgedTick::BRC20STick(TickId::from_str(str).unwrap()),
        _ => PledgedTick::Unknown,
      },
    }
  }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct TickInfo {
  pub tick_id: TickId,
  pub name: Tick,
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
    name: &Tick,
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
  use bitcoin::hashes::hex::ToHex;
  use bitcoin::hashes::{sha256, Hash, HashEngine};

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
      Tick::from_str(""),
      Err(BRC20SError::InvalidTickLen("".to_string()))
    );

    assert_eq!(
      Tick::from_str("1"),
      Err(BRC20SError::InvalidTickLen("1".to_string()))
    );

    assert_eq!(
      Tick::from_str("12"),
      Err(BRC20SError::InvalidTickLen("12".to_string()))
    );

    assert_eq!(
      Tick::from_str("123"),
      Err(BRC20SError::InvalidTickLen("123".to_string()))
    );

    assert_eq!(Tick::from_str("1234"), Tick::from_str("1234"));
    assert_eq!(Tick::from_str("12345"), Tick::from_str("12345"));
    assert_eq!(Tick::from_str("123456"), Tick::from_str("123456"));
    assert_eq!(
      Tick::from_str("1234567"),
      Err(BRC20SError::InvalidTickLen("1234567".to_string()))
    );
  }

  #[test]
  fn test_tick_compare_ignore_case() {
    assert_eq!(Tick::from_str("aBc1a"), Tick::from_str("AbC1A"));

    assert_ne!(Tick::from_str("aBc1D"), Tick::from_str("aBc2d"));
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
      Tick::from_str("aBc1a").unwrap().as_str(),
      "aBc1a".to_string()
    );

    assert_eq!(
      Tick::from_str("aBc1a").unwrap().to_lowercase().as_str(),
      "abc1a".to_string()
    );

    assert_eq!(
      Tick::from_str("aBc1a").unwrap().as_bytes(),
      "aBc1a".as_bytes()
    );

    assert_eq!(
      Tick::from_str("aBc1a").unwrap().hex(),
      "6142633161".to_string()
    );

    assert_eq!(Tick::min_hex(), "".to_string());
    assert_eq!(Tick::max_hex(), "ffffffffffff".to_string());
  }

  #[test]
  fn test_tick_btc() {
    assert_eq!(Tick::from_str("btc"), Tick::from_str("btc"));
    assert_eq!(Tick::from_str("btc"), Tick::from_str("BTC"));
    assert_eq!(Tick::from_str("btc"), Tick::from_str("Btc"));
    assert_ne!(Tick::from_str("btc123"), Tick::from_str("btc"));
    assert_eq!(
      serde_json::from_str::<Tick>(r##""btc""##).unwrap(),
      Tick::from_str("btc").unwrap()
    );
  }

  #[test]
  fn test_tid_err() {
    assert_eq!(
      TickId::from_str("btc").unwrap_err(),
      BRC20SError::InternalError("Odd number of digits".to_string())
    );

    assert_eq!(
      TickId::from_str("12345678").unwrap_err(),
      BRC20SError::InvalidTickLen("12345678".to_string())
    );

    let mut enc = sha256::Hash::engine();
    enc.input("123".as_bytes());
    let hash = sha256::Hash::from_engine(enc);
    assert_eq!(
      TickId::from_str(hash[0..TICK_ID_BYTE_COUNT - 1].to_hex().as_str()).unwrap_err(),
      BRC20SError::InvalidTickLen("a665a459".to_string())
    );

    assert_eq!(
      TickId::from_str(hash[0..TICK_ID_BYTE_COUNT + 1].to_hex().as_str()).unwrap_err(),
      BRC20SError::InvalidTickLen("a665a4592042".to_string())
    );

    assert_eq!(
      TickId::from_str(hash[0..TICK_ID_BYTE_COUNT].to_hex().as_str())
        .unwrap()
        .ne(&TickId::from_str("1234567890").unwrap()),
      true
    );
  }

  #[test]
  fn test_pid_to_tid() {
    let pid = Pid::from_str("A012345679#01").unwrap();
    let tid = TickId::from(pid);
    assert_eq!(tid, TickId::from_str("A012345679").unwrap());
    assert_eq!(tid, TickId::from_str("a012345679").unwrap());

    let pid = Pid::from_str("a012345679#01").unwrap();
    let tid = TickId::from(pid);
    assert_eq!(tid, TickId::from_str("A012345679").unwrap());
    assert_eq!(tid, TickId::from_str("a012345679").unwrap());
  }
}
