use super::*;

use crate::okx::datastore::btc::{Balance, DataStoreReadOnly};
use redb::{
  AccessGuard, Range, ReadOnlyTable, ReadTransaction, ReadableTable, RedbKey, RedbValue,
  StorageError, Table, TableDefinition, WriteTransaction,
};
use std::{borrow::Borrow, ops::RangeBounds};

pub fn try_init_tables<'db, 'a>(
  wtx: &'a WriteTransaction<'db>,
  rtx: &'a ReadTransaction<'db>,
) -> Result<bool, redb::Error> {
  if rtx.open_table(BTC_BALANCE).is_err() {
    wtx.open_table(BTC_BALANCE)?;
  }
  Ok(true)
}

pub struct DataStoreReader<'db, 'a> {
  wrapper: ReaderWrapper<'db, 'a>,
}

pub(in crate::okx) fn new_with_wtx<'db, 'a>(
  wtx: &'a WriteTransaction<'db>,
) -> DataStoreReader<'db, 'a> {
  DataStoreReader {
    wrapper: ReaderWrapper::Wtx(wtx),
  }
}

impl<'db, 'a> DataStoreReader<'db, 'a> {
  pub fn new(rtx: &'a ReadTransaction<'db>) -> Self {
    Self {
      wrapper: ReaderWrapper::Rtx(rtx),
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
  fn get<'a>(
    &self,
    key: impl Borrow<K::SelfType<'a>>,
  ) -> Result<Option<AccessGuard<'_, V>>, StorageError>
  where
    K: 'a,
  {
    match self {
      Self::RtxTable(rtx_table) => rtx_table.get(key),
      Self::WtxTable(wtx_table) => wtx_table.get(key),
    }
  }

  #[allow(dead_code)]
  fn range<'a: 'b, 'b, KR>(
    &'a self,
    range: impl RangeBounds<KR> + 'b,
  ) -> Result<Range<'a, K, V>, StorageError>
  where
    K: 'a,
    KR: Borrow<K::SelfType<'b>> + 'b,
  {
    match self {
      Self::RtxTable(rtx_table) => rtx_table.range(range),
      Self::WtxTable(wtx_table) => wtx_table.range(range),
    }
  }

  #[allow(dead_code)]
  fn len(&self) -> Result<u64, StorageError> {
    match self {
      Self::RtxTable(rtx_table) => rtx_table.len(),
      Self::WtxTable(wtx_table) => wtx_table.len(),
    }
  }
}

impl<'db, 'a> DataStoreReadOnly for DataStoreReader<'db, 'a> {
  type Error = redb::Error;

  fn get_balance(&self, script_key: &ScriptKey) -> Result<Option<Balance>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BTC_BALANCE)?
        .get(btc_script_key(script_key).as_str())?
        .map(|v| bincode::deserialize::<Balance>(v.value()).unwrap()),
    )
  }
}
