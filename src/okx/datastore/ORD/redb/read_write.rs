use super::*;
use crate::InscriptionId;
use crate::Result;
use bitcoin::Script;
use redb::{TableDefinition, WriteTransaction};

use crate::okx::datastore::ORD::{OrdDataStoreReadOnly, OrdDataStoreReadWrite};

const OUTPOINT_TO_SCRIPT: TableDefinition<&str, &[u8]> = TableDefinition::new("OUTPOINT_TO_SCRIPT");

pub struct OrdDbReadWriter<'db, 'a> {
  wtx: &'a WriteTransaction<'db>,
}

impl<'db, 'a> OrdDbReadWriter<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self { wtx }
  }
}

impl<'db, 'a> OrdDataStoreReadOnly for OrdDbReadWriter<'db, 'a> {
  type Error = redb::Error;

  fn get_number_by_inscription_id(&self, inscription_id: InscriptionId) -> Result<i64> {
    read_only::new_with_wtx(self.wtx).get_number_by_inscription_id(inscription_id)
  }

  fn get_outpoint_to_script(&self, outpoint: &str) -> Result<Option<Script>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_outpoint_to_script(outpoint)
  }
}

impl<'db, 'a> OrdDataStoreReadWrite for OrdDbReadWriter<'db, 'a> {
  // 3.3.1 OUTPOINT_TO_SCRIPT, todo, replace outpoint
  fn set_outpoint_to_script(&self, outpoint: &str, script: &Script) -> Result<(), Self::Error> {
    self
      .wtx
      .open_table(OUTPOINT_TO_SCRIPT)?
      .insert(outpoint, bincode::serialize(script).unwrap().as_slice())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::BRC30::{BRC30Tick, Pid, PledgedTick, PoolType, TickId};
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;

  #[test]
  fn test_outpoint_to_script() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = OrdDbReadWriter::new(&wtx);

    let outpoint1: &str = "outpoint-1";
    let script = Script::from_str("12345678").unwrap();

    brc30db.set_outpoint_to_script(&outpoint1, &script).unwrap();

    assert_eq!(
      brc30db.get_outpoint_to_script(&outpoint1).unwrap().unwrap(),
      script
    );
  }
}
