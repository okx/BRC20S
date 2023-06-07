use super::*;
use crate::brc30::ledger::LedgerRead;
use crate::brc30::{
  BRC30PoolInfo, BRC30Receipt, BRC30TickInfo, Balance, InscriptionOperation, Pid, TickId,
  TransferableAsset, UserInfo,
};
use bitcoin::Script;
use redb::{
  AccessGuard, Error, RangeIter, ReadOnlyTable, ReadTransaction, ReadableTable, RedbKey, RedbValue,
  Table, TableDefinition, WriteTransaction,
};
use std::borrow::Borrow;
use std::ops::RangeBounds;

pub struct BRC30DatabaseReader<'db, 'a> {
  wrapper: ReaderWrapper<'db, 'a>,
}

pub(super) fn new_with_wtx<'db, 'a>(
  wtx: &'a WriteTransaction<'db>,
) -> BRC30DatabaseReader<'db, 'a> {
  BRC30DatabaseReader {
    wrapper: ReaderWrapper::Wtx(wtx),
  }
}

impl<'db, 'a> BRC30DatabaseReader<'db, 'a> {
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

impl<'db, 'a> LedgerRead for BRC30DatabaseReader<'db, 'a> {
  type Error = redb::Error;

  // 3.3.1 OUTPOINT_TO_SCRIPT, todo, replace outpoint
  fn get_outpoint_to_script(&self, outpoint: &str) -> Result<Option<Script>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(OUTPOINT_TO_SCRIPT)?
        .get(outpoint)?
        .map(|v| bincode::deserialize::<Script>(v.value()).unwrap()),
    )
  }

  //3.3.2 TXID_TO_INSCRIPTION_RECEIPTS, todo, replace <Vec<InscriptionOperation>
  fn get_txid_to_inscription_receipts(
    &self,
    txid: &Txid,
  ) -> Result<Vec<InscriptionOperation>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(TXID_TO_INSCRIPTION_RECEIPTS)?
        .get(txid.to_string().as_str())?
        .map_or(Vec::new(), |v| {
          bincode::deserialize::<Vec<InscriptionOperation>>(v.value()).unwrap()
        }),
    )
  }

  // 3.3.3 BRC30_TICKINFO
  fn get_tick_info(&self, tick_id: &TickId) -> Result<Option<BRC30TickInfo>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_TICKINFO)?
        .get(tick_id.to_lowercase().hex().as_str())?
        .map(|v| bincode::deserialize::<BRC30TickInfo>(v.value()).unwrap()),
    )
  }

  // 3.3.4 BRC30_PID_TO_POOLINFO
  fn get_pid_to_poolinfo(&self, pid: &Pid) -> Result<Option<BRC30PoolInfo>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_PID_TO_POOLINFO)?
        .get(pid.to_lowercase().hex().as_str())?
        .map(|v| bincode::deserialize::<BRC30PoolInfo>(v.value()).unwrap()),
    )
  }

  // 3.3.5 BRC30_PID_TO_USERINFO
  fn get_pid_to_use_info(&self, pid: &Pid) -> Result<Option<UserInfo>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_PID_TO_USERINFO)?
        .get(pid.to_lowercase().hex().as_str())?
        .map(|v| bincode::deserialize::<UserInfo>(v.value()).unwrap()),
    )
  }

  // 3.3.6 BRC30_STAKE_TICKID_TO_PID 和 BRC30_TICKID_STAKE_TO_PID, TODO zhujianguo
  // fn get_stake_tick_id_to_pid(&self);

  // 3.3.7 BRC30_BALANCE
  fn get_balance(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
  ) -> Result<Option<Balance>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_BALANCES)?
        .get(script_tickid_key(script_key, tick_id).as_str())?
        .map(|v| {
          let bal = bincode::deserialize::<StoreBalance>(v.value()).unwrap();
          assert_eq!(&bal.tick_id, tick_id);
          bal.balance
        }),
    )

    // Ok(
    //   self
    //     .wrapper
    //     .open_table(BRC20_BALANCES)?
    //     .get(script_tick_key(script_key, tick).as_str())?
    //     .map(|v| {
    //       let bal = bincode::deserialize::<StoreBalance>(v.value()).unwrap();
    //       assert_eq!(&bal.tick, tick);
    //       bal.balance
    //     }),
    // )
  }

  fn get_balances(&self, script_key: &ScriptKey) -> Result<Vec<(TickId, Balance)>, Self::Error>{
    Ok(
      self
        .wrapper
        .open_table(BRC30_BALANCES)?
        .range(min_script_tick_id_key(script_key).as_str()..max_script_tick_id_key(&script_key).as_str())?
        .map(|(_, data)| {
          let bal = bincode::deserialize::<StoreBalance>(data.value()).unwrap();
          (bal.tick_id, bal.balance)
        })
        .collect(),
    )
  }

  // 3.3.8 BRC30_TRANSFERABLE_ASSETS
  fn get_transferable_assets(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableAsset>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_TRANSFERABLE_ASSETS)?
        .get(script_tickid_key(script_key, tick_id).as_str())?
        .map(|v| bincode::deserialize::<TransferableAsset>(v.value()).unwrap()),
    )
  }

  // 3.3.9 BRC30_TXID_TO_RECEIPTS, TODO replace BRC30ActionReceipt
  fn get_txid_to_receipts(&self, tick_id: &TickId) -> Result<Vec<BRC30Receipt>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(TXID_TO_INSCRIPTION_RECEIPTS)?
        .get(tick_id.to_lowercase().as_str())?
        .map_or(Vec::new(), |v| {
          bincode::deserialize::<Vec<BRC30Receipt>>(v.value()).unwrap()
        }),
    )
  }
}
