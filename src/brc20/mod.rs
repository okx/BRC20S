mod custom_serde;
mod error;
pub mod ledger;
mod num;
mod operation;
mod params;
mod updater;

pub use self::{
  error::Error,
  num::Num,
  operation::{deserialize_brc20, Deploy, Mint, Operation, Transfer},
  updater::{Action, InscriptionData},
};

use ledger::Ledger;

// pub fn update_ledger<L: Ledger>(
//   inscription_data: InscriptionData,
//   ledger: &mut L,
// ) -> Result<(), Error<L>> {
//   match inscription_data.get_action() {
//     Action::Inscribe(to_opt) => {
//       let operation = deserialize_brc20(inscription_data.inscription.into_body())?;
//       match operation {
//         Operation::Deploy(deploy) => {
//           todo!("not implemented")
//         }
//         Operation::Mint(mint) => {
//           todo!("not implemented")
//         }
//         Operation::Transfer(transfer) => {
//           todo!("not implemented")
//         }
//       }
//     }
//     Action::Transfer(from_opt, to_opt) => {
//       todo!("not implemented")
//     }
//   }

// operation.update_ledger(tx_id, ledger)
// }
