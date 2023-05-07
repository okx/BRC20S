use bitcoin::{Address, Network, Script, ScriptHash};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

// 用做数据库的key
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum ScriptKey {
  Address(Address),
  ScriptHash(ScriptHash),
}

impl ScriptKey {
  pub fn from_address(address: Address) -> Self {
    ScriptKey::Address(address)
  }
  pub fn from_script(script: &Script, network: Network) -> Self {
    if let Some(address) = Address::from_script(script, network).ok() {
      ScriptKey::Address(address)
    } else {
      ScriptKey::ScriptHash(script.script_hash())
    }
  }
}

impl Display for ScriptKey {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        ScriptKey::Address(address) => address.to_string(),
        ScriptKey::ScriptHash(script_hash) => script_hash.to_string(),
      }
    )
  }
}
