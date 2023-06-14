use crate::InscriptionId;
use crate::Result;
use bitcoin::Script;
use std::fmt::{Debug, Display};

pub use self::redb::{OrdDbReadWriter, OrdDbReader};
pub mod operation;
pub mod redb;

pub trait OrdDataStoreReadOnly {
  type Error: Debug + Display;

  fn get_number_by_inscription_id(&self, inscription_id: InscriptionId) -> Result<i64>;

  fn get_outpoint_to_script(&self, outpoint: &str) -> Result<Option<Script>, Self::Error>;
}

pub trait OrdDataStoreReadWrite: OrdDataStoreReadOnly {
  fn set_outpoint_to_script(&self, outpoint: &str, script: &Script) -> Result<(), Self::Error>;
}
