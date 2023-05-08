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

use ledger::Ledger;

pub fn deserialize_brc20_operation(inscription: Inscription) -> Result<Operation> {
  Ok(deserialize_brc20(std::str::from_utf8(
    inscription.body().ok_or(JSONError::InvalidJson)?,
  )?)?)
}
