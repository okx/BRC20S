use super::Tick;
use crate::brc20::num::Num;
use crate::brc20::{Error, Ledger};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Mint {
  #[serde(rename = "tick")]
  pub tick: Tick,
  #[serde(rename = "amt")]
  pub amount: Num,
}

impl Mint {
  pub(super) fn update_ledger<L: Ledger>(&self, ledger: &mut L) -> Result<(), Error<L>> {
    todo!("not implemented")
  }
}
