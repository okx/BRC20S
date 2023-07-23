use redb::{
  AccessGuard, ReadOnlyTable, ReadTransaction, ReadableTable, RedbKey, RedbValue, StorageError,
  Table, TableDefinition, WriteTransaction,
};
use std::{borrow::Borrow, io};

use bitcoin::{
  consensus::{Decodable, Encodable},
  hashes::Hash,
  OutPoint, TxOut, Txid,
};

use crate::{
  index::{INSCRIPTION_ID_TO_INSCRIPTION_ENTRY, OUTPOINT_TO_ENTRY},
  okx::datastore::ord::{InscriptionOp, OrdDataStoreReadOnly},
  InscriptionId, Result,
};

use super::ORD_TX_TO_OPERATIONS;

pub struct OrdDbReader<'db, 'a> {
  wrapper: ReaderWrapper<'db, 'a>,
}

pub(crate) fn new_with_wtx<'db, 'a>(wtx: &'a WriteTransaction<'db>) -> OrdDbReader<'db, 'a> {
  OrdDbReader {
    wrapper: ReaderWrapper::Wtx(wtx),
  }
}

impl<'db, 'a> OrdDbReader<'db, 'a> {
  #[allow(dead_code)]
  pub fn new(rtx: &'a ReadTransaction<'db>) -> Self {
    Self {
      wrapper: ReaderWrapper::Rtx(rtx),
    }
  }
}
#[allow(dead_code)]
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
}

impl<'db, 'a> OrdDataStoreReadOnly for OrdDbReader<'db, 'a> {
  type Error = redb::Error;
  fn get_number_by_inscription_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<i64>, Self::Error> {
    let mut key = [0; 36];
    let (txid, index) = key.split_at_mut(32);
    txid.copy_from_slice(inscription_id.txid.as_inner());
    index.copy_from_slice(&inscription_id.index.to_be_bytes());
    Ok(
      self
        .wrapper
        .open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY)?
        .get(&key)?
        .map(|value| value.value().2),
    )
  }

  fn get_outpoint_to_txout(&self, outpoint: OutPoint) -> Result<Option<TxOut>, Self::Error> {
    let mut value = [0; 36];
    outpoint
      .consensus_encode(&mut value.as_mut_slice())
      .unwrap();
    Ok(
      self
        .wrapper
        .open_table(OUTPOINT_TO_ENTRY)?
        .get(&value)?
        .map(|x| Decodable::consensus_decode(&mut io::Cursor::new(x.value())).unwrap()),
    )
  }

  fn get_transaction_operations(&self, txid: &Txid) -> Result<Vec<InscriptionOp>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(ORD_TX_TO_OPERATIONS)?
        .get(txid.to_string().as_str())?
        .map_or(Vec::new(), |v| {
          bincode::deserialize::<Vec<InscriptionOp>>(v.value()).unwrap()
        }),
    )
  }
}
