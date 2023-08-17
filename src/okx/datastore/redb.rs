use super::{
  brc20::redb::DataStore as BRC20DataStore, brc20s::redb::DataStore as BRC20SDataStore,
  btc::redb::DataStore as BTCDataStore, ord::OrdDbReadWriter, BRC20DataStoreReadWrite,
  BRC20SDataStoreReadWrite, BTCDataStoreReadWrite, DataStoreReadWriter, OrdDataStoreReadWrite,
};
use redb::WriteTransaction;
use std::sync::Arc;

pub struct DataEngineReadWriter<'a> {
  ord_store: Arc<dyn OrdDataStoreReadWrite<Error = redb::Error> + 'a>,
  btc_store: Arc<dyn BTCDataStoreReadWrite<Error = redb::Error> + 'a>,
  brc20_store: Arc<dyn BRC20DataStoreReadWrite<Error = redb::Error> + 'a>,
  brc20s_store: Arc<dyn BRC20SDataStoreReadWrite<Error = redb::Error> + 'a>,
}

impl<'a> DataStoreReadWriter<'a> for DataEngineReadWriter<'a> {
  type Error = redb::Error;
  fn ord_store(&self) -> Arc<dyn OrdDataStoreReadWrite<Error = Self::Error> + 'a> {
    self.ord_store.clone()
  }

  fn btc_store(&self) -> Arc<dyn BTCDataStoreReadWrite<Error = Self::Error> + 'a> {
    self.btc_store.clone()
  }

  fn brc20_store(&self) -> Arc<dyn BRC20DataStoreReadWrite<Error = Self::Error> + 'a> {
    self.brc20_store.clone()
  }
  fn brc20s_store(&self) -> Arc<dyn BRC20SDataStoreReadWrite<Error = Self::Error> + 'a> {
    self.brc20s_store.clone()
  }
}

impl<'a, 'db: 'a> DataEngineReadWriter<'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self {
      ord_store: Arc::new(OrdDbReadWriter::new(wtx)),
      btc_store: Arc::new(BTCDataStore::new(wtx)),
      brc20_store: Arc::new(BRC20DataStore::new(wtx)),
      brc20s_store: Arc::new(BRC20SDataStore::new(wtx)),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::{btc::Balance, ScriptKey};
  use bitcoin::Address;
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;
  #[test]
  fn test_data_engine() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let store = DataEngineReadWriter::new(&wtx);

    let script = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
        .unwrap()
        .assume_checked(),
    );

    let expect_balance = Balance { balance: 30 };
    store
      .btc_store()
      .update_balance(&script, expect_balance.clone())
      .unwrap();

    assert_eq!(
      store.btc_store().get_balance(&script).unwrap(),
      Some(expect_balance.clone())
    );
  }
}
