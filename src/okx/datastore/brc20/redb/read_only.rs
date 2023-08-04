use super::*;
use crate::okx::datastore::brc20::{
  Balance, DataStoreReadOnly, Receipt, Tick, TokenInfo, TransferInfo, TransferableLog,
};
use redb::{
  AccessGuard, Range, ReadOnlyTable, ReadTransaction, ReadableTable, RedbKey, RedbValue,
  StorageError, Table, TableDefinition, WriteTransaction,
};
use std::borrow::Borrow;
use std::ops::RangeBounds;

pub fn try_init_tables<'db, 'a>(
  wtx: &'a WriteTransaction<'db>,
  rtx: &'a ReadTransaction<'db>,
) -> Result<bool, redb::Error> {
  if rtx.open_table(BRC20_BALANCES).is_err() {
    wtx.open_table(BRC20_BALANCES)?;
    wtx.open_table(BRC20_TOKEN)?;
    wtx.open_table(BRC20_EVENTS)?;
    wtx.open_table(BRC20_TRANSFERABLELOG)?;
    wtx.open_table(BRC20_INSCRIBE_TRANSFER)?;
  }

  Ok(true)
}

pub struct DataStoreReader<'db, 'a> {
  wrapper: ReaderWrapper<'db, 'a>,
}

pub(super) fn new_with_wtx<'db, 'a>(wtx: &'a WriteTransaction<'db>) -> DataStoreReader<'db, 'a> {
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
}

impl<'db, 'a> DataStoreReadOnly for DataStoreReader<'db, 'a> {
  type Error = redb::Error;

  fn get_balances(&self, script_key: &ScriptKey) -> Result<Vec<Balance>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC20_BALANCES)?
        .range(min_script_tick_key(script_key).as_str()..max_script_tick_key(script_key).as_str())?
        .flat_map(|result| {
          result.map(|(_, data)| bincode::deserialize::<Balance>(data.value()).unwrap())
        })
        .collect(),
    )
  }

  fn get_balance(
    &self,
    script_key: &ScriptKey,
    tick: &Tick,
  ) -> Result<Option<Balance>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC20_BALANCES)?
        .get(script_tick_key(script_key, tick).as_str())?
        .map(|v| bincode::deserialize::<Balance>(v.value()).unwrap()),
    )
  }

  fn get_token_info(&self, tick: &Tick) -> Result<Option<TokenInfo>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC20_TOKEN)?
        .get(tick.to_lowercase().hex().as_str())?
        .map(|v| bincode::deserialize::<TokenInfo>(v.value()).unwrap()),
    )
  }

  fn get_tokens_info(&self) -> Result<Vec<TokenInfo>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC20_TOKEN)?
        .range::<&str>(..)?
        .flat_map(|result| {
          result.map(|(_, data)| bincode::deserialize::<TokenInfo>(data.value()).unwrap())
        })
        .collect(),
    )
  }

  fn get_transaction_receipts(&self, txid: &Txid) -> Result<Vec<Receipt>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC20_EVENTS)?
        .get(txid.to_string().as_str())?
        .map_or(Vec::new(), |v| {
          bincode::deserialize::<Vec<Receipt>>(v.value()).unwrap()
        }),
    )
  }

  fn get_transferable(&self, script: &ScriptKey) -> Result<Vec<TransferableLog>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC20_TRANSFERABLELOG)?
        .range(min_script_tick_key(script).as_str()..max_script_tick_key(script).as_str())?
        .flat_map(|result| {
          result.map(|(_, v)| bincode::deserialize::<Vec<TransferableLog>>(v.value()).unwrap())
        })
        .flatten()
        .collect(),
    )
  }

  fn get_transferable_by_tick(
    &self,
    script: &ScriptKey,
    tick: &Tick,
  ) -> Result<Vec<TransferableLog>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC20_TRANSFERABLELOG)?
        .get(script_tick_key(script, tick).as_str())?
        .map_or(Vec::new(), |v| {
          bincode::deserialize::<Vec<TransferableLog>>(v.value()).unwrap()
        }),
    )
  }

  fn get_transferable_by_id(
    &self,
    script: &ScriptKey,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableLog>, Self::Error> {
    Ok(
      self
        .get_transferable(script)?
        .iter()
        .find(|log| log.inscription_id == *inscription_id)
        .cloned(),
    )
  }

  fn get_inscribe_transfer_inscription(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<TransferInfo>, Self::Error> {
    let mut value = [0; 36];
    let (txid, index) = value.split_at_mut(32);
    txid.copy_from_slice(inscription_id.txid.as_ref());
    index.copy_from_slice(&inscription_id.index.to_be_bytes());
    Ok(
      self
        .wrapper
        .open_table(BRC20_INSCRIBE_TRANSFER)?
        .get(&value)?
        .map(|v| bincode::deserialize::<TransferInfo>(v.value()).unwrap()),
    )
  }
}
