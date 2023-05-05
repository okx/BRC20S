use super::{deserialize_brc20, Error, Ledger};
use crate::{Address, Inscription, InscriptionId, Transaction, Txid};
use bitcoin::Script;
use redb::Table;
use rust_decimal::{
  prelude::{FromPrimitive, ToPrimitive},
  Decimal,
};
use std::ops::Mul;

#[derive(Debug, Clone)]
pub enum Action {
  // 新铸造的Inscription，包含receiver
  Inscribe(Script),
  // 旧的Inscription转移，包含sender和receiver
  Transfer(Script, Script),
}
impl Action {
  pub fn set_to(&mut self, to: Script) {
    match self {
      Action::Inscribe(to_opt) => *to_opt = to,
      Action::Transfer(_, to_opt) => *to_opt = to,
    }
  }
}
// 用于inscription updater传递给brc20 updater的信息
pub struct InscriptionData {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription: Inscription,
  pub(crate) action: Action,
}
impl InscriptionData {
  pub fn get_inscription_id(&self) -> InscriptionId {
    self.inscription_id
  }

  pub fn get_action(&self) -> Action {
    self.action.clone()
  }

  pub fn set_action(&mut self, action: Action) {
    self.action = action;
  }
}

// pub(super) struct Updater<L: Ledger> {
//   ledger: L,
// }
// impl<T: Ledger> Updater<T> {
//   fn index_transaction(
//     &mut self,
//     tx: &Transaction,
//     txid: Txid,
//     inscriptions_data: Vec<InscriptionData>,
//   ) -> Result<(), Error<T>> {
//     for inscription_data in inscriptions_data {
//       match inscription_data.origin {
//         Origin::New(to) => {
//           let to = to.ok_or()?
//           let operation = deserialize_brc20(protocol)?;
//           operation.update_ledger(tx_id, ledger);
//         }
//         Origin::Old(from, to) => {}
//       }
//     }
//     Ok(())
//   }
// }
// impl<'a, 'db, 'tx, L: 'a + Ledger> Updater<'a, 'db, 'tx, L: 'a + Ledger> {
//   pub(super) fn index_transaction_brc20(
//     &mut self,
//     tx: &Transaction,
//     txid: Txid,
//     inscriptions_data: Vec<InscriptionData>,
//   ) -> Result<(), Error<L>> {
//     //
//     Ok(())
//   }
// }
