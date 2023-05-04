mod deploy;
mod mint;
mod tick;
mod transfer;

use crate::brc20::error::{BRC20Error, JSONError};
use crate::brc20::params::*;
use crate::brc20::{Error, Ledger};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};

pub use self::{deploy::Deploy, mint::Mint, tick::Tick, transfer::Transfer};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "op")]
pub enum Operation {
  #[serde(rename = "deploy")]
  Deploy(Deploy),
  #[serde(rename = "mint")]
  Mint(Mint),
  #[serde(rename = "transfer")]
  Transfer(Transfer),
}

pub fn deserialize_brc20(s: &str) -> Result<Operation, JSONError> {
  let value: Value = serde_json::from_str(s).map_err(|_| JSONError::InvalidJson)?;
  if value.get("p") != Some(&json!(PROTOCOL_LITERAL)) {
    return Err(JSONError::NotBRC20Json);
  }

  let mut op = serde_json::from_str::<Operation>(s)
    .map_err(|e| JSONError::ParseOperationJsonError(e.to_string()))?;

  Ok(op)
}

impl Operation {
  pub fn update_ledger<L: Ledger>(self, tx_id: &str, ledger: &mut L) -> Result<(), Error<L>> {
    match self {
      Self::Deploy(deploy) => deploy.update_ledger(tx_id, ledger),
      Self::Mint(mint) => mint.update_ledger(ledger),
      Self::Transfer(transfer) => transfer.update_ledger(ledger),
    }
  }
}

impl Operation {
  fn check(&self) -> Result<(), BRC20Error> {
    match self {
      Self::Deploy(deploy) => deploy.check(),
      Self::Mint(_mint) => Ok(()),         // do nothing
      Self::Transfer(_transfer) => Ok(()), // do nothing
    }
  }

  fn reset_decimals(&mut self) {
    if let Self::Deploy(deploy) = self {
      deploy.reset_decimals();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::brc20::num::Num;
  use std::str::FromStr;

  #[test]
  fn test_deploy_deserialize() {
    let max_supply = Num::from_str("21000000").unwrap();
    let mint_limit = Num::from_str("1000").unwrap();

    let json_str = format!(
      r##"{{
  "p": "brc-20",
  "op": "deploy",
  "tick": "ordi",
  "max": "{max_supply}",
  "lim": "{mint_limit}"
}}"##
    );

    assert_eq!(
      deserialize_brc20(&json_str),
      Ok(Operation::Deploy(Deploy {
        tick: Tick::from("ordi"),
        max_supply,
        mint_limit: Some(mint_limit),
        decimals: default_decimals(),
      }))
    );
  }

  #[test]
  fn test_mint_deserialize() {
    let amount = Num::from_str("1000").unwrap();

    let json_str = format!(
      r##"{{
  "p": "brc-20",
  "op": "mint",
  "tick": "ordi",
  "amt": "{amount}"
}}"##
    );

    assert_eq!(
      deserialize_brc20(&json_str),
      Ok(Operation::Mint(Mint {
        tick: Tick::from("ordi"),
        amount,
      }))
    );
  }

  #[test]
  fn test_transfer_deserialize() {
    let amount = Num::from_str("100").unwrap();

    let json_str = format!(
      r##"{{
  "p": "brc-20",
  "op": "transfer",
  "tick": "ordi",
  "amt": "{amount}"
}}"##
    );

    assert_eq!(
      deserialize_brc20(&json_str),
      Ok(Operation::Transfer(Transfer {
        tick: Tick::from("ordi"),
        amount,
      }))
    );
  }

}
