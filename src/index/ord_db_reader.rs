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

pub trait OrdDbReader {
  type Error: Debug + Display;

  fn get_number_by_inscription_id(&self, inscription_id: InscriptionId) -> Result<u64>;
}

pub(crate) struct OrdDatabaseReader<'db, 'a> {
  wrapper: ReaderWrapper<'db, 'a>,
}

pub(crate) fn new_with_wtx<'db, 'a>(wtx: &'a WriteTransaction<'db>) -> OrdDatabaseReader<'db, 'a> {
  OrdDatabaseReader {
    wrapper: ReaderWrapper::Wtx(wtx),
  }
}

impl<'db, 'a> OrdDatabaseReader<'db, 'a> {
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

impl<'db, 'a> OrdDbReader for OrdDatabaseReader<'db, 'a> {
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
}
