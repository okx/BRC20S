use super::*;
use crate::okx::datastore::ScriptKey;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScriptPubkey {
  Address(String),
  NonStandard(String),
}
impl Default for ScriptPubkey {
  fn default() -> Self {
    ScriptPubkey::NonStandard(String::new())
  }
}

impl From<ScriptKey> for ScriptPubkey {
  fn from(script_key: ScriptKey) -> Self {
    match script_key {
      ScriptKey::Address(address) => ScriptPubkey::Address(address.to_string()),
      ScriptKey::ScriptHash(hash) => ScriptPubkey::NonStandard(hash.to_string()),
    }
  }
}
