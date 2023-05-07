use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Deploy {
  #[serde(rename = "tick")]
  pub tick: String,
  #[serde(rename = "max")]
  pub max_supply: String,
  #[serde(rename = "lim")]
  pub mint_limit: Option<String>,
  #[serde(rename = "dec")]
  pub decimals: Option<String>,
}

#[cfg(test)]
mod tests {

  #[test]
  fn test_serialize() {
    todo!("sss")
  }

  #[test]
  fn test_deserialize() {
    todo!("sss")
  }

  #[test]
  fn test_loss_require_key() {
    todo!("sss")
  }

  #[test]
  fn test_loss_option_key() {
    todo!("sss")
  }

  #[test]
  fn test_duplicate_key() {
    todo!("sss")
  }
}
