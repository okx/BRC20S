use super::*;
use crate::okx::datastore::btc::{Balance, DataStoreReadOnly, DataStoreReadWrite};
use redb::WriteTransaction;

pub struct DataStore<'db, 'a> {
  wtx: &'a WriteTransaction<'db>,
}

impl<'db, 'a> DataStore<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self { wtx }
  }
}

impl<'db, 'a> DataStoreReadOnly for DataStore<'db, 'a> {
  type Error = redb::Error;

  fn get_balance(&self, script_key: &ScriptKey) -> Result<Option<Balance>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_balance(script_key)
  }
}

impl<'db, 'a> DataStoreReadWrite for DataStore<'db, 'a> {
  fn update_balance(
    &self,
    script_key: &ScriptKey,
    new_balance: Balance,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BTC_BALANCE)?.insert(
      btc_script_key(script_key).as_str(),
      bincode::serialize(&new_balance).unwrap().as_slice(),
    )?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::btc::{Balance, DataStoreReadOnly, DataStoreReadWrite};
  use bitcoin::Address;
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;

  #[test]
  fn test_set_get_balance() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let btcdb = DataStore::new(&wtx);

    let script = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
        .unwrap()
        .assume_checked(),
    );

    let script2 = ScriptKey::from_address(
      Address::from_str("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq")
        .unwrap()
        .assume_checked(),
    );

    let expect_balance = Balance { balance: 30 };

    btcdb
      .update_balance(&script, expect_balance.clone())
      .unwrap();

    assert_eq!(
      btcdb.get_balance(&script).unwrap(),
      Some(expect_balance.clone())
    );
    assert_eq!(btcdb.get_balance(&script).unwrap(), Some(expect_balance));
    assert_eq!(btcdb.get_balance(&script2).unwrap(), None)
  }
}
