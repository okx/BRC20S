pub use self::operation::{Action, InscriptionOp};

use {
  crate::{InscriptionId, Result},
  bitcoin::{OutPoint, TxOut, Txid},
  collections::CollectionKind,
  std::fmt::{Debug, Display},
};
pub mod bitmap;
pub mod collections;
pub mod operation;
pub mod redb;

pub trait OrdReader {
  type Error: Debug + Display;
  fn get_number_by_inscription_id(
    &self,
    inscription_id: &InscriptionId,
  ) -> Result<Option<i64>, Self::Error>;

  fn get_outpoint_to_txout(&self, outpoint: &OutPoint) -> Result<Option<TxOut>, Self::Error>;

  fn get_transaction_operations(&self, txid: &Txid) -> Result<Vec<InscriptionOp>, Self::Error>;

  fn get_collections_of_inscription(
    &self,
    inscription_id: &InscriptionId,
  ) -> Result<Option<Vec<CollectionKind>>, Self::Error>;

  fn get_collection_inscription_id(
    &self,
    collection_key: &str,
  ) -> Result<Option<InscriptionId>, Self::Error>;
}

pub trait OrdReaderWriter: OrdReader {
  fn save_transaction_operations(
    &mut self,
    txid: &Txid,
    operations: &[InscriptionOp],
  ) -> Result<(), Self::Error>;

  fn set_inscription_by_collection_key(
    &mut self,
    key: &str,
    inscription_id: &InscriptionId,
  ) -> Result<(), Self::Error>;

  fn set_inscription_attributes(
    &mut self,
    inscription_id: &InscriptionId,
    kind: &[CollectionKind],
  ) -> Result<(), Self::Error>;
}
