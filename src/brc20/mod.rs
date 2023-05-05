mod custom_serde;
mod error;
pub mod ledger;
mod num;
mod operation;
mod params;

pub use self::{
  error::Error,
  num::Num,
  operation::{deserialize_brc20, Deploy, Mint, Operation, Transfer},
};

use ledger::Ledger;

pub fn update_ledger<L: Ledger>(
  protocol: &str,
  tx_id: &str,
  ledger: &mut L,
) -> Result<(), Error<L>> {
  let operation = deserialize_brc20(protocol)?;

  operation.update_ledger(tx_id, ledger)
}
