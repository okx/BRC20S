pub use self::{
  operation::{Action, InscriptionOp},
  redb::{OrdDbReadWriter, OrdDbReader},
};
use crate::InscriptionId;
use crate::Result;
use bitcoin::OutPoint;
use bitcoin::TxOut;
use std::fmt::{Debug, Display};
pub mod operation;
pub mod redb;

pub trait OrdDataStoreReadOnly {
  type Error: Debug + Display;
  fn get_number_by_inscription_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<i64>, Self::Error>;

  fn get_outpoint_to_txout(&self, outpoint: OutPoint) -> Result<Option<TxOut>, Self::Error>;
}

pub trait OrdDataStoreReadWrite: OrdDataStoreReadOnly {
  fn set_outpoint_to_txout(&self, outpoint: OutPoint, txout: &TxOut) -> Result;
}
