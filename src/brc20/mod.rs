use crate::{Inscription, Result};
use error::JSONError;
mod custom_serde;
mod error;
pub mod ledger;
mod num;
mod operation;
mod params;
mod types;
mod updater;

pub use self::{
  error::{BRC20Error, Error},
  num::Num,
  operation::{deserialize_brc20, Deploy, Mint, Operation, Transfer},
  types::*,
  updater::{Action, BRC20Updater, InscribeAction, InscriptionData},
};

use ledger::{LedgerRead, LedgerReadWrite};

pub fn deserialize_brc20_operation(inscription: Inscription) -> Result<Operation> {
  let content_body = std::str::from_utf8(inscription.body().ok_or(JSONError::InvalidJson)?)?;
  if content_body.len() < 40 {
    return Err(JSONError::NotBRC20Json.into());
  }

  let content_type = inscription
    .content_type()
    .ok_or(JSONError::InvalidContentType)?;

  if content_type != "text/plain"
    && content_type != "text/plain;charset=utf-8"
    && content_type != "text/plain;charset=UTF-8"
    && content_type != "application/json"
  {
    if !content_type.starts_with("text/plain;") {
      return Err(JSONError::UnSupportContentType.into());
    }
  }
  Ok(deserialize_brc20(content_body)?)
}
