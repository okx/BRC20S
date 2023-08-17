pub(crate) mod balance;
pub mod brc20;
pub mod brc20s;
pub mod btc;
pub mod ord;
mod redb;
mod script_key;

use std::{
  fmt::{Debug, Display},
  sync::Arc,
};

pub use self::{
  brc20::{
    DataStoreReadOnly as BRC20DataStoreReadOnly, DataStoreReadWrite as BRC20DataStoreReadWrite,
  },
  brc20s::{
    DataStoreReadOnly as BRC20SDataStoreReadOnly, DataStoreReadWrite as BRC20SDataStoreReadWrite,
  },
  btc::{DataStoreReadOnly as BTCDataStoreReadOnly, DataStoreReadWrite as BTCDataStoreReadWrite},
  ord::{OrdDataStoreReadOnly, OrdDataStoreReadWrite},
  script_key::ScriptKey,
};

pub trait DataStoreReader<'a> {
  type Error: Debug + Display;
  fn ord_store(&self) -> Arc<dyn OrdDataStoreReadOnly<Error = Self::Error> + 'a>;
  fn btc_store(&self) -> Arc<dyn BTCDataStoreReadOnly<Error = Self::Error> + 'a>;
  fn brc20_store(&self) -> Arc<dyn BRC20DataStoreReadOnly<Error = Self::Error> + 'a>;
  fn brc20s_store(&self) -> Arc<dyn BRC20SDataStoreReadOnly<Error = Self::Error> + 'a>;
}

pub trait DataStoreReadWriter<'a> {
  type Error: Debug + Display;
  fn ord_store(&self) -> Arc<dyn OrdDataStoreReadWrite<Error = Self::Error> + 'a>;
  fn btc_store(&self) -> Arc<dyn BTCDataStoreReadWrite<Error = Self::Error> + 'a>;
  fn brc20_store(&self) -> Arc<dyn BRC20DataStoreReadWrite<Error = Self::Error> + 'a>;
  fn brc20s_store(&self) -> Arc<dyn BRC20SDataStoreReadWrite<Error = Self::Error> + 'a>;

  // fn read(&self) -> dyn DataStoreReader<Error = Self::Error>;
}
