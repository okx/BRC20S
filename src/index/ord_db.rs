use super::*;
use crate::index::INSCRIPTION_ID_TO_INSCRIPTION_ENTRY;
use crate::InscriptionId;
use crate::Result;
use bitcoin::Script;
use redb::{
  AccessGuard, Error, RangeIter, ReadOnlyTable, ReadTransaction, ReadableTable, RedbKey, RedbValue,
  Table, TableDefinition, WriteTransaction,
};
use std::borrow::Borrow;
use std::ops::RangeBounds;

use std::fmt::{Debug, Display};

const OUTPOINT_TO_SCRIPT: TableDefinition<&str, &[u8]> = TableDefinition::new("OUTPOINT_TO_SCRIPT");

pub trait OrdDbReadAPI {
  type Error: Debug + Display;

  fn get_number_by_inscription_id(&self, inscription_id: InscriptionId) -> Result<u64>;

  fn get_outpoint_to_script(&self, outpoint: &str) -> Result<Option<Script>, Self::Error>;
}

pub trait OrdDbReadWriteAPI: OrdDbReadAPI {
  fn set_outpoint_to_script(&self, outpoint: &str, script: &Script) -> Result<(), Self::Error>;
}

pub(crate) struct OrdDbReader<'db, 'a> {
  wrapper: ReaderWrapper<'db, 'a>,
}

pub(crate) fn new_with_wtx<'db, 'a>(wtx: &'a WriteTransaction<'db>) -> OrdDbReader<'db, 'a> {
  OrdDbReader {
    wrapper: ReaderWrapper::Wtx(wtx),
  }
}

impl<'db, 'a> OrdDbReader<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self {
      wrapper: ReaderWrapper::Wtx(wtx),
    }
  }
}

enum ReaderWrapper<'db, 'a> {
  Rtx(&'a ReadTransaction<'db>),
  Wtx(&'a WriteTransaction<'db>),
}

impl<'db, 'a> ReaderWrapper<'db, 'a> {
  fn open_table<K: RedbKey + 'static, V: RedbValue + 'static>(
    &self,
    definition: TableDefinition<'_, K, V>,
  ) -> Result<TableWrapper<'db, '_, K, V>, redb::Error> {
    match self {
      Self::Rtx(rtx) => Ok(TableWrapper::RtxTable(rtx.open_table(definition)?)),
      Self::Wtx(wtx) => Ok(TableWrapper::WtxTable(wtx.open_table(definition)?)),
    }
  }
}

enum TableWrapper<'db, 'txn, K: RedbKey + 'static, V: RedbValue + 'static> {
  RtxTable(ReadOnlyTable<'txn, K, V>),
  WtxTable(Table<'db, 'txn, K, V>),
}

impl<'db, 'txn, K: RedbKey + 'static, V: RedbValue + 'static> TableWrapper<'db, 'txn, K, V> {
  fn get<'a>(&self, key: impl Borrow<K::SelfType<'a>>) -> Result<Option<AccessGuard<'_, V>>, Error>
  where
    K: 'a,
  {
    match self {
      Self::RtxTable(rtx_table) => rtx_table.get(key),
      Self::WtxTable(wtx_table) => wtx_table.get(key),
    }
  }

  fn range<'a: 'b, 'b, KR>(
    &'a self,
    range: impl RangeBounds<KR> + 'b,
  ) -> Result<RangeIter<'a, K, V>, Error>
  where
    K: 'a,
    KR: Borrow<K::SelfType<'b>> + 'b,
  {
    match self {
      Self::RtxTable(rtx_table) => rtx_table.range(range),
      Self::WtxTable(wtx_table) => wtx_table.range(range),
    }
  }
}

impl<'db, 'a> OrdDbReadAPI for OrdDbReader<'db, 'a> {
  type Error = redb::Error;

  fn get_number_by_inscription_id(&self, inscription_id: InscriptionId) -> Result<u64> {
    Ok(
      self
        .wrapper
        .open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY)?
        .get(&inscription_id.store())?
        .ok_or(anyhow!(
          "failed to find inscription number for {}",
          inscription_id
        ))?
        .value()
        .2,
    )
  }

  fn get_outpoint_to_script(&self, outpoint: &str) -> Result<Option<Script>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(OUTPOINT_TO_SCRIPT)?
        .get(outpoint)?
        .map(|v| bincode::deserialize::<Script>(v.value()).unwrap()),
    )
  }
}

pub struct OrdDbReadWriter<'db, 'a> {
  wtx: &'a WriteTransaction<'db>,
}

impl<'db, 'a> OrdDbReadWriter<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self { wtx }
  }
}

impl<'db, 'a> OrdDbReadAPI for OrdDbReadWriter<'db, 'a> {
  type Error = redb::Error;

  fn get_number_by_inscription_id(&self, inscription_id: InscriptionId) -> Result<u64> {
    new_with_wtx(self.wtx).get_number_by_inscription_id(inscription_id)
  }

  fn get_outpoint_to_script(&self, outpoint: &str) -> Result<Option<Script>, Self::Error> {
    new_with_wtx(self.wtx).get_outpoint_to_script(outpoint)
  }
}

impl<'db, 'a> OrdDbReadWriteAPI for OrdDbReadWriter<'db, 'a> {
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
  use crate::brc30::ledger::BRC30DbReadWriteAPI;
  use crate::brc30::BRC30Tick;
  use crate::SatPoint;
  use bitcoin::Address;
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
