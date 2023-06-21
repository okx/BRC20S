pub use self::{
  operation::{Action, InscriptionOp},
  redb::{OrdDbReadWriter, OrdDbReader},
};
use crate::InscriptionId;
use crate::Result;
use bitcoin::OutPoint;
use bitcoin::TxOut;
pub mod operation;
pub mod redb;

pub trait OrdDataStoreReadOnly {
  fn get_number_by_inscription_id(&self, inscription_id: InscriptionId) -> Result<i64>;

  fn get_outpoint_to_txout(&self, outpoint: OutPoint) -> Result<Option<TxOut>>;
}

pub trait OrdDataStoreReadWrite: OrdDataStoreReadOnly {
  fn set_outpoint_to_txout(&self, outpoint: OutPoint, txout: &TxOut) -> Result;
}
