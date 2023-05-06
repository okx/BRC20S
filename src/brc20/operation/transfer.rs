use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Transfer {
  #[serde(rename = "tick")]
  pub tick: String,
  #[serde(rename = "amt")]
  pub amount: String,
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
  fn test_duplicate_key() {
    todo!("sss")
  }
}
