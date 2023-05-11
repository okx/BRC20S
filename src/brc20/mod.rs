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
  updater::{Action, BRC20Updater, InscribeAction, InscriptionData, TransferAction},
};

use ledger::{LedgerRead, LedgerReadWrite};

const MEDIA_TYPE_TEXT: &str = "text/plain;charset=utf-8";
const MEDIA_TYPE_JSON: &str = "application/json";

pub fn deserialize_brc20_operation(inscription: Inscription) -> Result<Operation> {
  match inscription
    .content_type()
    .ok_or(JSONError::InvalidContentType)?
  {
    MEDIA_TYPE_TEXT | MEDIA_TYPE_JSON => Ok(deserialize_brc20(std::str::from_utf8(
      inscription.body().ok_or(JSONError::InvalidJson)?,
    )?)?),
    &_ => Err(JSONError::UnSupportContentType.into()),
  }
}
