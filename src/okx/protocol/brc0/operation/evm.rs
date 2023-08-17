use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Evm {
  #[serde(rename = "d")]
  pub d: String,
}
