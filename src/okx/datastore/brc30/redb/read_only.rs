use super::*;
use crate::okx::datastore::brc30::{
  BRC30DataStoreReadOnly, BRC30Receipt, Balance, InscriptionOperation, Pid, PledgedTick, PoolInfo,
  StakeInfo, TickId, TickInfo, TransferableAsset, UserInfo,
};
use redb::{
  AccessGuard, Error, RangeIter, ReadOnlyTable, ReadTransaction, ReadableTable, RedbKey, RedbValue,
  Table, TableDefinition, WriteTransaction,
};
use std::borrow::Borrow;
use std::ops::RangeBounds;

pub struct BRC30DataStoreReader<'db, 'a> {
  wrapper: ReaderWrapper<'db, 'a>,
}

pub(in crate::okx) fn new_with_wtx<'db, 'a>(
  wtx: &'a WriteTransaction<'db>,
) -> BRC30DataStoreReader<'db, 'a> {
  BRC30DataStoreReader {
    wrapper: ReaderWrapper::Wtx(wtx),
  }
}

impl<'db, 'a> BRC30DataStoreReader<'db, 'a> {
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

  fn len(&self) -> Result<usize, Error> {
    match self {
      Self::RtxTable(rtx_table) => rtx_table.len(),
      Self::WtxTable(wtx_table) => wtx_table.len(),
    }
  }
}

impl<'db, 'a> BRC30DataStoreReadOnly for BRC30DataStoreReader<'db, 'a> {
  type Error = redb::Error;

  //3.3.2 TXID_TO_INSCRIPTION_RECEIPTS
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
  fn get_tick_info(&self, tick_id: &TickId) -> Result<Option<TickInfo>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_TICKINFO)?
        .get(tick_id.to_lowercase().hex().as_str())?
        .map(|v| bincode::deserialize::<TickInfo>(v.value()).unwrap()),
    )
  }

  fn get_all_tick_info(
    &self,
    start: usize,
    limit: Option<usize>,
  ) -> Result<(Vec<TickInfo>, usize), Self::Error> {
    let table = self.wrapper.open_table(BRC30_TICKINFO)?;
    let total = table.len()?;
    return Ok((
      table
        .range(TickId::min_hex().as_str()..TickId::max_hex().as_str())?
        .skip(start)
        .take(limit.unwrap_or(usize::MAX))
        .map(|(_, data)| {
          let tick_info = bincode::deserialize::<TickInfo>(data.value()).unwrap();
          tick_info
        })
        .collect(),
      total,
    ));
  }

  // 3.3.4 BRC30_PID_TO_POOLINFO
  fn get_pid_to_poolinfo(&self, pid: &Pid) -> Result<Option<PoolInfo>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_PID_TO_POOLINFO)?
        .get(pid.to_lowercase().hex().as_str())?
        .map(|v| bincode::deserialize::<PoolInfo>(v.value()).unwrap()),
    )
  }

  fn get_all_poolinfo(
    &self,
    start: usize,
    limit: Option<usize>,
  ) -> Result<(Vec<PoolInfo>, usize), Self::Error> {
    let table = self.wrapper.open_table(BRC30_PID_TO_POOLINFO)?;
    let total = table.len()?;
    return Ok((
      table
        .range(Pid::min_hex().as_str()..Pid::max_hex().as_str())?
        .skip(start)
        .take(limit.unwrap_or(usize::MAX))
        .map(|(_, data)| {
          let pool = bincode::deserialize::<PoolInfo>(data.value()).unwrap();
          pool
        })
        .collect(),
      total,
    ));
  }

  // 3.3.5 BRC30_USER_STAKEINFO
  fn get_user_stakeinfo(
    &self,
    script_key: &ScriptKey,
    pledged_tick: &PledgedTick,
  ) -> Result<Option<StakeInfo>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_USER_STAKEINFO)?
        .get(script_pledged_key(script_key, pledged_tick).as_str())?
        .map(|v| bincode::deserialize::<StakeInfo>(v.value()).unwrap()),
    )
  }

  // 3.3.6 BRC30_PID_TO_USERINFO
  fn get_pid_to_use_info(
    &self,
    script_key: &ScriptKey,
    pid: &Pid,
  ) -> Result<Option<UserInfo>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_PID_TO_USERINFO)?
        .get(script_pid_key(&script_key, &pid).as_str())?
        .map(|v| bincode::deserialize::<UserInfo>(v.value()).unwrap()),
    )
  }

  // 3.3.7 BRC30_STAKE_TICKID_TO_PID
  fn get_tickid_stake_to_pid(
    &self,
    tick_id: &TickId,
    pledged: &PledgedTick,
  ) -> Result<Option<Pid>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_STAKE_TICKID_TO_PID)?
        .get(stake_tickid_key(pledged, tick_id).as_str())?
        .map(|v| bincode::deserialize::<Pid>(v.value()).unwrap()),
    )
  }

  // 3.3.7 get_tickid_to_all_pid
  fn get_tickid_to_all_pid(&self, tick_id: &TickId) -> Result<Vec<Pid>, Self::Error> {
    let min = min_tickid_stake_key(tick_id);
    let max = max_tickid_stake_key(tick_id);
    Ok(
      self
        .wrapper
        .open_table(BRC30_TICKID_STAKE_TO_PID)?
        .range(min.as_str()..max.as_str())?
        .map(|(_, data)| {
          let pid = bincode::deserialize::<Pid>(data.value()).unwrap();
          pid
        })
        .collect(),
    )
  }

  // 3.3.7 get_stake_to_all_pid
  fn get_stake_to_all_pid(&self, pledged: &PledgedTick) -> Result<Vec<Pid>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_STAKE_TICKID_TO_PID)?
        .range(min_stake_tickid_key(pledged).as_str()..max_stake_tickid_key(pledged).as_str())?
        .map(|(_, data)| {
          let pid = bincode::deserialize::<Pid>(data.value()).unwrap();
          pid
        })
        .collect(),
    )
  }

  // 3.3.8 BRC30_BALANCE
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
          let bal = bincode::deserialize::<Balance>(v.value()).unwrap();
          assert_eq!(&bal.tick_id, tick_id);
          bal
        }),
    )
  }

  fn get_balances(&self, script_key: &ScriptKey) -> Result<Vec<(TickId, Balance)>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_BALANCES)?
        .range(
          min_script_tick_id_key(script_key).as_str()..max_script_tick_id_key(&script_key).as_str(),
        )?
        .map(|(_, data)| {
          let bal = bincode::deserialize::<Balance>(data.value()).unwrap();
          (bal.tick_id, bal.clone())
        })
        .collect(),
    )
  }

  // 3.3.9 BRC30_TRANSFERABLE_ASSETS
  fn get_transferable_asset(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableAsset>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_TRANSFERABLE_ASSETS)?
        .get(script_tickid_inscriptionid_key(script_key, tick_id, inscription_id).as_str())?
        .map(|v| bincode::deserialize::<TransferableAsset>(v.value()).unwrap()),
    )
  }

  fn get_transferable(&self, script: &ScriptKey) -> Result<Vec<TransferableAsset>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_TRANSFERABLE_ASSETS)?
        .range(min_script_tick_id_key(script).as_str()..max_script_tick_id_key(script).as_str())?
        .map(|(_, v)| bincode::deserialize::<TransferableAsset>(v.value()).unwrap())
        .collect(),
    )
  }

  fn get_transferable_by_tickid(
    &self,
    script: &ScriptKey,
    tick_id: &TickId,
  ) -> Result<Vec<TransferableAsset>, Self::Error> {
    Ok(
      self
        .get_transferable(script)?
        .iter()
        .filter(|log| log.tick_id == *tick_id)
        .map(|log| log.clone())
        .collect(),
    )
  }

  fn get_transferable_by_id(
    &self,
    script: &ScriptKey,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableAsset>, Self::Error> {
    Ok(
      self
        .get_transferable(script)?
        .iter()
        .find(|log| log.inscription_id == *inscription_id)
        .map(|log| log.clone()),
    )
  }

  // 3.3.10 BRC30_TXID_TO_RECEIPTS
  fn get_txid_to_receipts(&self, txid: &Txid) -> Result<Vec<BRC30Receipt>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_TXID_TO_RECEIPTS)?
        .get(txid.to_string().as_str())?
        .map_or(Vec::new(), |v| {
          bincode::deserialize::<Vec<BRC30Receipt>>(v.value()).unwrap()
        }),
    )
  }

  fn get_transaction_receipts(&self, tx_id: &Txid) -> Result<Vec<BRC30Receipt>, Self::Error> {
    Ok(
      self
        .wrapper
        .open_table(BRC30_TXID_TO_RECEIPTS)?
        .get(tx_id.to_string().as_str())?
        .map_or(Vec::new(), |v| {
          bincode::deserialize::<Vec<BRC30Receipt>>(v.value()).unwrap()
        }),
    )
  }
}
