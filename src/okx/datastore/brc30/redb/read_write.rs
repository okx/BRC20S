use super::*;
use crate::{
  okx::datastore::brc30::{
    BRC30DataStoreReadWrite, Balance, DataStoreReadOnly, InscriptionOperation, Pid, PoolInfo,
    Receipt, StakeInfo, TickId, TickInfo, TransferInfo, TransferableAsset, UserInfo,
  },
  InscriptionId,
};
use bitcoin::{hashes::Hash, Txid};
use redb::WriteTransaction;

pub struct BRC30DataStore<'db, 'a> {
  wtx: &'a WriteTransaction<'db>,
}

impl<'db, 'a> BRC30DataStore<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self { wtx }
  }
}

impl<'db, 'a> DataStoreReadOnly for BRC30DataStore<'db, 'a> {
  type Error = redb::Error;

  //3.3.2 TXID_TO_INSCRIPTION_RECEIPTS
  fn get_txid_to_inscription_receipts(
    &self,
    txid: &Txid,
  ) -> Result<Vec<InscriptionOperation>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_txid_to_inscription_receipts(txid)
  }

  // 3.3.3 BRC30_TICKINFO
  fn get_tick_info(&self, tick_id: &TickId) -> Result<Option<TickInfo>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_tick_info(tick_id)
  }

  fn get_all_tick_info(
    &self,
    start: usize,
    limit: Option<usize>,
  ) -> Result<(Vec<TickInfo>, usize), Self::Error> {
    read_only::new_with_wtx(self.wtx).get_all_tick_info(start, limit)
  }

  // 3.3.4 BRC30_PID_TO_POOLINFO
  fn get_pid_to_poolinfo(&self, pid: &Pid) -> Result<Option<PoolInfo>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_pid_to_poolinfo(pid)
  }

  fn get_all_poolinfo(
    &self,
    start: usize,
    limit: Option<usize>,
  ) -> Result<(Vec<PoolInfo>, usize), Self::Error> {
    read_only::new_with_wtx(self.wtx).get_all_poolinfo(start, limit)
  }

  // 3.3.5 BRC30_USER_STAKEINFO
  fn get_user_stakeinfo(
    &self,
    script_key: &ScriptKey,
    pledged_tick: &PledgedTick,
  ) -> Result<Option<StakeInfo>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_user_stakeinfo(script_key, pledged_tick)
  }

  // 3.3.6 BRC30_PID_TO_USERINFO
  fn get_pid_to_use_info(
    &self,
    script_key: &ScriptKey,
    pid: &Pid,
  ) -> Result<Option<UserInfo>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_pid_to_use_info(script_key, pid)
  }

  // 3.3.7 BRC30_STAKE_TICKID_TO_PID
  fn get_tickid_stake_to_pid(
    &self,
    tick_id: &TickId,
    pledged: &PledgedTick,
  ) -> Result<Option<Pid>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_tickid_stake_to_pid(tick_id, pledged)
  }

  // 3.3.7 get_tickid_to_all_pid
  fn get_tickid_to_all_pid(&self, tick_id: &TickId) -> Result<Vec<Pid>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_tickid_to_all_pid(tick_id)
  }

  // 3.3.7 get_stake_to_all_pid
  fn get_stake_to_all_pid(&self, pledged: &PledgedTick) -> Result<Vec<Pid>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_stake_to_all_pid(pledged)
  }

  // 3.3.8 BRC30_BALANCE
  fn get_balance(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
  ) -> Result<Option<Balance>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_balance(script_key, tick_id)
  }

  fn get_balances(&self, script_key: &ScriptKey) -> Result<Vec<(TickId, Balance)>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_balances(script_key)
  }

  // 3.3.9 BRC30_TRANSFERABLE_ASSETS
  fn get_transferable_asset(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableAsset>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_transferable_asset(script_key, tick_id, inscription_id)
  }

  fn get_transferable(&self, script: &ScriptKey) -> Result<Vec<TransferableAsset>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_transferable(script)
  }

  fn get_transferable_by_tickid(
    &self,
    script: &ScriptKey,
    tick_id: &TickId,
  ) -> Result<Vec<TransferableAsset>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_transferable_by_tickid(script, tick_id)
  }

  fn get_transferable_by_id(
    &self,
    script: &ScriptKey,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableAsset>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_transferable_by_id(script, inscription_id)
  }

  // 3.3.10 BRC30_TXID_TO_RECEIPTS
  fn get_txid_to_receipts(&self, tx_id: &Txid) -> Result<Vec<Receipt>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_txid_to_receipts(tx_id)
  }

  fn get_transaction_receipts(&self, tx_id: &Txid) -> Result<Vec<Receipt>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_transaction_receipts(tx_id)
  }

  fn get_inscribe_transfer_inscription(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<TransferInfo>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_inscribe_transfer_inscription(inscription_id)
  }
}

impl<'db, 'a> BRC30DataStoreReadWrite for BRC30DataStore<'db, 'a> {
  //3.3.2 TXID_TO_INSCRIPTION_RECEIPTS
  fn set_txid_to_inscription_receipts(
    &self,
    tx_id: &Txid,
    inscription_operations: &Vec<InscriptionOperation>,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(TXID_TO_INSCRIPTION_RECEIPTS)?.insert(
      tx_id.to_string().as_str(),
      bincode::serialize(inscription_operations)
        .unwrap()
        .as_slice(),
    )?;
    Ok(())
  }

  // 3.3.3 BRC30_TICKINFO
  fn set_tick_info(&self, tick_id: &TickId, brc30_tick_info: &TickInfo) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_TICKINFO)?.insert(
      tick_id.to_lowercase().hex().as_str(),
      bincode::serialize(brc30_tick_info).unwrap().as_slice(),
    )?;
    Ok(())
  }

  // 3.3.4 BRC30_PID_TO_POOLINFO
  fn set_pid_to_poolinfo(&self, pid: &Pid, brc30_pool_info: &PoolInfo) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_PID_TO_POOLINFO)?.insert(
      pid.to_lowercase().hex().as_str(),
      bincode::serialize(brc30_pool_info).unwrap().as_slice(),
    )?;
    Ok(())
  }

  // 3.3.5 BRC30_USER_STAKEINFO
  fn set_user_stakeinfo(
    &self,
    script_key: &ScriptKey,
    pledged_tick: &PledgedTick,
    stake_info: &StakeInfo,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_USER_STAKEINFO)?.insert(
      script_pledged_key(script_key, pledged_tick).as_str(),
      bincode::serialize(stake_info).unwrap().as_slice(),
    )?;
    Ok(())
  }

  // 3.3.6 BRC30_PID_TO_USERINFO
  fn set_pid_to_use_info(
    &self,
    script_key: &ScriptKey,
    pid: &Pid,
    user_info: &UserInfo,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_PID_TO_USERINFO)?.insert(
      script_pid_key(&script_key, &pid).as_str(),
      bincode::serialize(user_info).unwrap().as_slice(),
    )?;
    Ok(())
  }

  // 3.3.7 BRC30_STAKE_TICKID_TO_PID, BRC30_TICKID_STAKE_TO_PID
  fn set_tickid_stake_to_pid(
    &self,
    tick_id: &TickId,
    pledged: &PledgedTick,
    pid: &Pid,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_STAKE_TICKID_TO_PID)?.insert(
      stake_tickid_key(pledged, tick_id).as_str(),
      bincode::serialize(pid).unwrap().as_slice(),
    )?;

    self.wtx.open_table(BRC30_TICKID_STAKE_TO_PID)?.insert(
      tickid_stake_key(pledged, tick_id).as_str(),
      bincode::serialize(pid).unwrap().as_slice(),
    )?;
    Ok(())
  }

  // 3.3.8 BRC30_BALANCE
  fn set_token_balance(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    balance: Balance,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_BALANCES)?.insert(
      script_tickid_key(script_key, tick_id).as_str(),
      bincode::serialize(&balance).unwrap().as_slice(),
    )?;
    Ok(())
  }

  // 3.3.9 BRC30_TRANSFERABLE_ASSETS
  fn set_transferable_assets(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    inscription_id: &InscriptionId,
    transferable_asset: &TransferableAsset,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_TRANSFERABLE_ASSETS)?.insert(
      script_tickid_inscriptionid_key(script_key, tick_id, inscription_id).as_str(),
      bincode::serialize(transferable_asset).unwrap().as_slice(),
    )?;
    Ok(())
  }

  // 3.3.10 BRC30_TXID_TO_RECEIPTS
  fn add_transaction_receipt(&self, tx_id: &Txid, receipt: &Receipt) -> Result<(), Self::Error> {
    let mut receipts = self.get_transaction_receipts(tx_id)?;
    receipts.push(receipt.clone());
    self.save_transaction_receipts(tx_id, &receipts)
  }

  fn save_transaction_receipts(
    &self,
    tx_id: &Txid,
    receipts: &[Receipt],
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_TXID_TO_RECEIPTS)?.insert(
      tx_id.to_string().as_str(),
      bincode::serialize(receipts).unwrap().as_slice(),
    )?;
    Ok(())
  }

  fn remove_transferable(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    inscription_id: &InscriptionId,
  ) -> Result<(), Self::Error> {
    self
      .wtx
      .open_table(BRC30_TRANSFERABLE_ASSETS)?
      .remove(script_tickid_inscriptionid_key(script_key, tick_id, inscription_id).as_str())?;
    Ok(())
  }

  fn insert_inscribe_transfer_inscription(
    &self,
    inscription_id: InscriptionId,
    transfer_info: TransferInfo,
  ) -> Result<(), Self::Error> {
    let mut value = [0; 36];
    let (txid, index) = value.split_at_mut(32);
    txid.copy_from_slice(inscription_id.txid.as_inner());
    index.copy_from_slice(&inscription_id.index.to_be_bytes());

    self.wtx.open_table(BRC30_INSCRIBE_TRANSFER)?.insert(
      &value,
      bincode::serialize(&transfer_info).unwrap().as_slice(),
    )?;
    Ok(())
  }

  fn remove_inscribe_transfer_inscription(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<(), Self::Error> {
    let mut value = [0; 36];
    let (txid, index) = value.split_at_mut(32);
    txid.copy_from_slice(inscription_id.txid.as_inner());
    index.copy_from_slice(&inscription_id.index.to_be_bytes());

    self
      .wtx
      .open_table(BRC30_INSCRIBE_TRANSFER)?
      .remove(&value)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::brc20;
  use crate::okx::datastore::brc30::{BRC30DataStoreReadWrite, DataStoreReadOnly, OperationType};
  use crate::okx::datastore::brc30::{Pid, PledgedTick, PoolType, Tick, TickId};
  use crate::okx::protocol::brc30::BRC30Error;
  use crate::SatPoint;
  use bitcoin::Address;
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;

  #[test]
  fn test_balances() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let script = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
    );
    let tick1 = TickId::from_str("f7c515d6b1").unwrap();
    let tick2 = TickId::from_str("f7c515d6b2").unwrap();
    let tick3 = TickId::from_str("f7c515d6b3").unwrap();
    let expect_balance1 = Balance {
      tick_id: tick1.clone(),
      overall_balance: 10,
      transferable_balance: 10,
    };
    let expect_balance2 = Balance {
      tick_id: tick2.clone(),
      overall_balance: 30,
      transferable_balance: 30,
    };
    let expect_balance3 = Balance {
      tick_id: tick3.clone(),
      overall_balance: 100,
      transferable_balance: 30,
    };
    brc30db
      .set_token_balance(&script, &tick1, expect_balance1.clone())
      .unwrap();
    brc30db
      .set_token_balance(&script, &tick2, expect_balance2.clone())
      .unwrap();
    brc30db
      .set_token_balance(&script, &tick3, expect_balance3.clone())
      .unwrap();

    assert_eq!(
      brc30db.get_balance(&script, &tick1).unwrap().unwrap(),
      expect_balance1
    );

    let script2 =
      ScriptKey::from_address(Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap());
    assert_ne!(script.to_string(), script2.to_string());
    let expect_balance22 = Balance {
      tick_id: tick2.clone(),
      overall_balance: 100,
      transferable_balance: 30,
    };
    brc30db
      .set_token_balance(&script2, &tick2, expect_balance22.clone())
      .unwrap();

    let mut all_balances = brc30db.get_balances(&script).unwrap();
    all_balances.sort_by(|a, b| a.0.hex().cmp(&b.0.hex()));
    let mut expect = vec![
      (tick2, expect_balance2),
      (tick1, expect_balance1),
      (tick3, expect_balance3),
    ];
    expect.sort_by(|a, b| a.0.hex().cmp(&b.0.hex()));
    assert_eq!(all_balances, expect);
  }

  #[test]
  fn test_txid_to_inscription_receipts() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let txid =
      Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();

    let op_vec = vec![
      InscriptionOperation {
        txid: Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735")
          .unwrap(),
      },
      InscriptionOperation {
        txid: Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735")
          .unwrap(),
      },
      InscriptionOperation {
        txid: Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735")
          .unwrap(),
      },
    ];

    brc30db
      .set_txid_to_inscription_receipts(&txid, &op_vec)
      .unwrap();

    assert_eq!(
      brc30db.get_txid_to_inscription_receipts(&txid).unwrap(),
      op_vec
    );
  }

  #[test]
  fn test_pid_to_poolinfo() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let inscription_id =
      InscriptionId::from_str("2111111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();

    let pid_1 = Pid::from_str("1234567890#01").unwrap();
    let pid_2 = Pid::from_str("1234567890#02").unwrap();
    let pid_3 = Pid::from_str("1234567890#03").unwrap();
    let pid_4 = Pid::from_str("1234567890#04").unwrap();
    let pid_5 = Pid::from_str("1234567890#05").unwrap();

    let pool_info_1 = PoolInfo {
      pid: pid_1.clone(),
      ptype: PoolType::Pool,
      inscription_id: inscription_id.clone(),
      stake: PledgedTick::Native,
      erate: 0,
      minted: 0,
      staked: 0,
      dmax: 0,
      acc_reward_per_share: "0".to_string(),
      last_update_block: 0,
      only: true,
    };
    let mut pool_info_2 = pool_info_1.clone();
    pool_info_2.pid = pid_2.clone();
    let mut pool_info_3 = pool_info_1.clone();
    pool_info_3.pid = pid_3.clone();
    let mut pool_info_4 = pool_info_1.clone();
    pool_info_4.pid = pid_4.clone();
    let mut pool_info_5 = pool_info_1.clone();
    pool_info_5.pid = pid_5.clone();

    brc30db.set_pid_to_poolinfo(&pid_1, &pool_info_1).unwrap();
    brc30db.set_pid_to_poolinfo(&pid_2, &pool_info_2).unwrap();
    brc30db.set_pid_to_poolinfo(&pid_3, &pool_info_3).unwrap();
    brc30db.set_pid_to_poolinfo(&pid_4, &pool_info_4).unwrap();
    brc30db.set_pid_to_poolinfo(&pid_5, &pool_info_5).unwrap();

    assert_eq!(
      brc30db.get_pid_to_poolinfo(&pid_1).unwrap().unwrap(),
      pool_info_1
    );

    assert_eq!(
      brc30db.get_all_poolinfo(0, None).unwrap(),
      (
        vec![
          pool_info_1.clone(),
          pool_info_2.clone(),
          pool_info_3.clone(),
          pool_info_4.clone(),
          pool_info_5.clone(),
        ],
        5
      )
    );
    assert_eq!(
      brc30db.get_all_poolinfo(0, Some(3)).unwrap(),
      (
        vec![
          pool_info_1.clone(),
          pool_info_2.clone(),
          pool_info_3.clone(),
        ],
        5
      )
    );

    assert_eq!(
      brc30db.get_all_poolinfo(0, Some(5)).unwrap(),
      (
        vec![
          pool_info_1.clone(),
          pool_info_2.clone(),
          pool_info_3.clone(),
          pool_info_4.clone(),
          pool_info_5.clone()
        ],
        5
      )
    );

    assert_eq!(
      brc30db.get_all_poolinfo(0, Some(9)).unwrap(),
      (
        vec![
          pool_info_1.clone(),
          pool_info_2.clone(),
          pool_info_3.clone(),
          pool_info_4.clone(),
          pool_info_5.clone()
        ],
        5
      )
    );

    assert_eq!(
      brc30db.get_all_poolinfo(3, Some(1)).unwrap(),
      (vec![pool_info_4.clone()], 5)
    );

    assert_eq!(
      brc30db.get_all_poolinfo(3, Some(9)).unwrap(),
      (vec![pool_info_4.clone(), pool_info_5.clone()], 5)
    );

    assert_eq!(brc30db.get_all_poolinfo(5, Some(9)).unwrap(), (vec![], 5));
  }

  #[test]
  fn test_user_stakeinfo() {
    let script = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
    );

    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let pledged_tick_20 = PledgedTick::BRC20Tick(brc20::Tick::from_str("tk20").unwrap());
    let pid_20 = Pid::from_str("0000000000#01").unwrap();
    let stake_info_20 = StakeInfo {
      stake: pledged_tick_20.clone(),
      pool_stakes: vec![(pid_20.clone(), true, 123)],
      max_share: 0,
      total_only: 0,
    };

    let pledged_tick_30 = PledgedTick::BRC30Tick(TickId::from_str("f7c515d630").unwrap());
    let pid_30 = Pid::from_str("0000000000#02").unwrap();
    let stake_info_30 = StakeInfo {
      stake: pledged_tick_30.clone(),
      pool_stakes: vec![(pid_30.clone(), true, 123)],
      max_share: 0,
      total_only: 0,
    };

    let pledged_tick_btc = PledgedTick::Native;
    let pid_btc = Pid::from_str("0000000000#03").unwrap();
    let stake_info_btc = StakeInfo {
      stake: pledged_tick_btc.clone(),
      pool_stakes: vec![(pid_btc.clone(), true, 123)],
      max_share: 0,
      total_only: 0,
    };

    let pledged_tick_unknown = PledgedTick::Unknown;
    let pid_btc = Pid::from_str("0000000000#03").unwrap();
    let stake_info_unknown = StakeInfo {
      stake: pledged_tick_unknown.clone(),
      pool_stakes: vec![(pid_btc.clone(), true, 123)],
      max_share: 0,
      total_only: 0,
    };

    brc30db
      .set_user_stakeinfo(&script, &pledged_tick_20, &stake_info_20)
      .unwrap();

    brc30db
      .set_user_stakeinfo(&script, &pledged_tick_20, &stake_info_30)
      .unwrap();

    brc30db
      .set_user_stakeinfo(&script, &pledged_tick_30, &stake_info_btc)
      .unwrap();

    brc30db
      .set_user_stakeinfo(&script, &pledged_tick_btc, &stake_info_btc)
      .unwrap();

    brc30db
      .set_user_stakeinfo(&script, &pledged_tick_unknown, &stake_info_unknown)
      .unwrap();

    assert_eq!(
      brc30db
        .get_user_stakeinfo(&script, &pledged_tick_20)
        .unwrap()
        .unwrap(),
      stake_info_30
    );

    assert_eq!(
      brc30db
        .get_user_stakeinfo(&script, &pledged_tick_30)
        .unwrap()
        .unwrap(),
      stake_info_btc
    );

    assert_eq!(
      brc30db
        .get_user_stakeinfo(&script, &pledged_tick_btc)
        .unwrap()
        .unwrap(),
      stake_info_btc
    );

    assert_eq!(
      brc30db
        .get_user_stakeinfo(&script, &pledged_tick_unknown)
        .unwrap()
        .unwrap(),
      stake_info_unknown
    );
  }

  #[test]
  fn test_tick_info() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let inscription_id =
      InscriptionId::from_str("2111111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();
    let tick_id_1 = TickId::from_str("f7c515d6b1").unwrap();
    let tick_id_2 = TickId::from_str("f7c515d6b2").unwrap();
    let tick_id_3 = TickId::from_str("f7c515d6b3").unwrap();
    let tick_id_4 = TickId::from_str("f7c515d6b4").unwrap();
    let tick_id_5 = TickId::from_str("f7c515d6b5").unwrap();

    let pid = Pid::from_str("1234567890#01").unwrap();
    let tick_info1 = TickInfo {
      tick_id: tick_id_1.clone(),
      name: Tick::from_str("aBc1ab").unwrap(),
      inscription_id: inscription_id.clone(),
      allocated: 100,
      decimal: 1,
      circulation: 100,
      supply: 100,
      deployer: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
      deploy_block: 100,
      latest_mint_block: 100,
      pids: vec![pid],
    };

    let mut tick_info2 = tick_info1.clone();
    tick_info2.decimal = 2;
    let mut tick_info3 = tick_info1.clone();
    tick_info3.decimal = 3;
    let mut tick_info4 = tick_info1.clone();
    tick_info4.decimal = 4;
    let mut tick_info5 = tick_info1.clone();
    tick_info5.decimal = 5;

    brc30db.set_tick_info(&tick_id_1, &tick_info1).unwrap();
    brc30db.set_tick_info(&tick_id_2, &tick_info2).unwrap();
    brc30db.set_tick_info(&tick_id_3, &tick_info3).unwrap();
    brc30db.set_tick_info(&tick_id_4, &tick_info4).unwrap();
    brc30db.set_tick_info(&tick_id_5, &tick_info5).unwrap();

    assert_eq!(
      brc30db.get_tick_info(&tick_id_1).unwrap().unwrap(),
      tick_info1.clone()
    );

    assert_eq!(
      brc30db.get_all_tick_info(0, None).unwrap(),
      (
        vec![
          tick_info1.clone(),
          tick_info2.clone(),
          tick_info3.clone(),
          tick_info4.clone(),
          tick_info5.clone(),
        ],
        5
      )
    );
    assert_eq!(
      brc30db.get_all_tick_info(0, Some(3)).unwrap(),
      (
        vec![tick_info1.clone(), tick_info2.clone(), tick_info3.clone(),],
        5
      )
    );

    assert_eq!(
      brc30db.get_all_tick_info(0, Some(5)).unwrap(),
      (
        vec![
          tick_info1.clone(),
          tick_info2.clone(),
          tick_info3.clone(),
          tick_info4.clone(),
          tick_info5.clone()
        ],
        5
      )
    );

    assert_eq!(
      brc30db.get_all_tick_info(0, Some(9)).unwrap(),
      (
        vec![
          tick_info1.clone(),
          tick_info2.clone(),
          tick_info3.clone(),
          tick_info4.clone(),
          tick_info5.clone()
        ],
        5
      )
    );

    assert_eq!(
      brc30db.get_all_tick_info(3, Some(1)).unwrap(),
      (vec![tick_info4.clone()], 5)
    );

    assert_eq!(
      brc30db.get_all_tick_info(3, Some(9)).unwrap(),
      (vec![tick_info4.clone(), tick_info5.clone()], 5)
    );

    assert_eq!(brc30db.get_all_tick_info(5, Some(9)).unwrap(), (vec![], 5));
  }

  #[test]
  fn test_pid_to_use_info() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let pid = Pid::from_str("1234567890#01").unwrap();
    let user_info = UserInfo {
      pid: pid.clone(),
      staked: 0,
      minted: 0,
      pending_reward: 0,
      reward_debt: 0,
      latest_updated_block: 0,
    };
    let script_key =
      ScriptKey::from_address(Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap());

    brc30db
      .set_pid_to_use_info(&script_key, &pid, &user_info)
      .unwrap();

    assert_eq!(
      brc30db
        .get_pid_to_use_info(&script_key, &pid)
        .unwrap()
        .unwrap(),
      user_info
    );
  }

  #[test]
  fn test_transferable_assets() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let script_key =
      ScriptKey::from_address(Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap());
    let tick_id = TickId::from_str("f7c515d6b7").unwrap();
    let inscription_id =
      InscriptionId::from_str("2111111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();
    let transferable_asset = TransferableAsset {
      inscription_id,
      amount: 100,
      tick_id: tick_id.clone(),
      owner: script_key.clone(),
    };

    brc30db
      .set_transferable_assets(
        &script_key,
        &tick_id,
        &inscription_id,
        &transferable_asset.clone(),
      )
      .unwrap();

    assert_eq!(
      brc30db
        .get_transferable_asset(&script_key, &tick_id, &inscription_id)
        .unwrap()
        .unwrap(),
      transferable_asset
    );
  }

  #[test]
  fn test_txid_to_receipts() {
    let addr =
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap();
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let txid =
      Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();

    let inscription_id =
      InscriptionId::from_str("2111111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();

    let op_vec = vec![
      Receipt {
        inscription_id: inscription_id.clone(),
        inscription_number: 0,
        old_satpoint: SatPoint {
          outpoint: Default::default(),
          offset: 0,
        },
        new_satpoint: SatPoint {
          outpoint: Default::default(),
          offset: 0,
        },
        op: OperationType::Transfer,
        from: ScriptKey::Address(addr.clone()),
        to: ScriptKey::Address(addr.clone()),
        result: Err(BRC30Error::InvalidTickLen("abcde".to_string())),
      },
      Receipt {
        inscription_id: inscription_id.clone(),
        inscription_number: 0,
        old_satpoint: SatPoint {
          outpoint: Default::default(),
          offset: 0,
        },
        new_satpoint: SatPoint {
          outpoint: Default::default(),
          offset: 0,
        },
        op: OperationType::Transfer,
        from: ScriptKey::Address(addr.clone()),
        to: ScriptKey::Address(addr.clone()),
        result: Err(BRC30Error::InvalidTickLen("abcde".to_string())),
      },
      Receipt {
        inscription_id: inscription_id.clone(),
        inscription_number: 0,
        old_satpoint: SatPoint {
          outpoint: Default::default(),
          offset: 0,
        },
        new_satpoint: SatPoint {
          outpoint: Default::default(),
          offset: 0,
        },
        op: OperationType::Transfer,
        from: ScriptKey::Address(addr.clone()),
        to: ScriptKey::Address(addr.clone()),
        result: Err(BRC30Error::InvalidTickLen("abcde".to_string())),
      },
    ];

    brc30db.save_transaction_receipts(&txid, &op_vec).unwrap();

    assert_eq!(brc30db.get_txid_to_receipts(&txid).unwrap(), op_vec);
  }

  #[test]
  fn test_stake_tickid_to_pid() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let pledged_tick_20 = PledgedTick::BRC20Tick(brc20::Tick::from_str("tk20").unwrap());
    let pid_20 = Pid::from_str("1234567890#01").unwrap();

    let pledged_tick_30 = PledgedTick::BRC30Tick(TickId::from_str("f7c515d630").unwrap());
    let pid_30 = Pid::from_str("1234567891#01").unwrap();

    let tick1 = TickId::from_str("f7c515d6b1").unwrap();
    let tick2 = TickId::from_str("f7c515d6b2").unwrap();
    let tick3 = TickId::from_str("f7c515d6b3").unwrap();

    let pledged_tick_btc = PledgedTick::BRC30Tick(TickId::from_str("f7c515d600").unwrap());
    let pid_btc = Pid::from_str("1234567890#01").unwrap();

    brc30db
      .set_tickid_stake_to_pid(&tick1, &pledged_tick_20, &pid_20)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick2, &pledged_tick_30, &pid_30)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick3, &pledged_tick_btc, &pid_btc)
      .unwrap();

    assert_eq!(
      brc30db
        .get_tickid_stake_to_pid(&tick1, &pledged_tick_20)
        .unwrap()
        .unwrap(),
      pid_20
    );
    assert_eq!(
      brc30db
        .get_tickid_stake_to_pid(&tick2, &pledged_tick_30)
        .unwrap()
        .unwrap(),
      pid_30
    );
    assert_eq!(
      brc30db
        .get_tickid_stake_to_pid(&tick3, &pledged_tick_btc)
        .unwrap()
        .unwrap(),
      pid_btc
    );
  }

  #[test]
  fn test_tickid_stake_to_all_pid() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let pledged_tick_20 = PledgedTick::BRC20Tick(brc20::Tick::from_str("tk20").unwrap());
    let pid_20 = Pid::from_str("1234567890#01").unwrap();

    let pledged_tick_30 = PledgedTick::BRC30Tick(TickId::from_str("f7c515d630").unwrap());
    let pid_30 = Pid::from_str("1234567891#01").unwrap();

    let tick1 = TickId::from_str("f7c515d6b1").unwrap();
    let tick2 = TickId::from_str("f7c515d6b2").unwrap();
    let tick3 = TickId::from_str("f7c515d6b3").unwrap();

    let pledged_tick_30_1 = PledgedTick::BRC30Tick(TickId::from_str("f7c515d600").unwrap());
    let pid_30_1 = Pid::from_str("1234567892#01").unwrap();

    let pledged_tick_btc = PledgedTick::Native;
    let pid_btc = Pid::from_str("1234567892#02").unwrap();

    let pledged_tick_unknown = PledgedTick::Unknown;
    let pid_unknown = Pid::from_str("1234567892#03").unwrap();

    brc30db
      .set_tickid_stake_to_pid(&tick1, &pledged_tick_20, &pid_20)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick1, &pledged_tick_30, &pid_30)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick1, &pledged_tick_30_1, &pid_30_1)
      .unwrap();

    brc30db
      .set_tickid_stake_to_pid(&tick1, &pledged_tick_btc, &pid_btc)
      .unwrap();

    brc30db
      .set_tickid_stake_to_pid(&tick1, &pledged_tick_unknown, &pid_unknown)
      .unwrap();

    brc30db
      .set_tickid_stake_to_pid(&tick2, &pledged_tick_20, &pid_20)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick2, &pledged_tick_30, &pid_30)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick2, &pledged_tick_30_1, &pid_30_1)
      .unwrap();

    brc30db
      .set_tickid_stake_to_pid(&tick3, &pledged_tick_20, &pid_20)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick3, &pledged_tick_30, &pid_30)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick3, &pledged_tick_30_1, &pid_30_1)
      .unwrap();

    assert_eq!(
      brc30db
        .get_tickid_stake_to_pid(&tick1, &pledged_tick_20)
        .unwrap()
        .unwrap(),
      pid_20
    );
    assert_eq!(
      brc30db
        .get_tickid_stake_to_pid(&tick2, &pledged_tick_30)
        .unwrap()
        .unwrap(),
      pid_30
    );
    assert_eq!(
      brc30db
        .get_tickid_stake_to_pid(&tick1, &pledged_tick_30)
        .unwrap()
        .unwrap(),
      pid_30
    );

    assert_eq!(
      brc30db
        .get_tickid_stake_to_pid(&tick1, &pledged_tick_30_1)
        .unwrap()
        .unwrap(),
      pid_30_1
    );

    assert_eq!(
      brc30db
        .get_tickid_stake_to_pid(&tick1, &pledged_tick_btc)
        .unwrap()
        .unwrap(),
      pid_btc
    );

    assert_eq!(
      brc30db
        .get_tickid_stake_to_pid(&tick1, &pledged_tick_unknown)
        .unwrap()
        .unwrap(),
      pid_unknown
    );

    assert_eq!(
      brc30db.get_tickid_to_all_pid(&tick1).unwrap(),
      vec![
        pid_unknown.clone(),
        pid_btc.clone(),
        pid_20.clone(),
        pid_30_1.clone(),
        pid_30.clone(),
      ]
    );

    assert_eq!(
      brc30db.get_stake_to_all_pid(&pledged_tick_30).unwrap(),
      vec![pid_30.clone(), pid_30.clone(), pid_30.clone()]
    );

    assert_eq!(
      brc30db.get_stake_to_all_pid(&pledged_tick_btc).unwrap(),
      vec![pid_btc.clone()]
    );

    assert_eq!(
      brc30db.get_stake_to_all_pid(&pledged_tick_20).unwrap(),
      vec![pid_20.clone(), pid_20.clone(), pid_20.clone()]
    );

    assert_eq!(
      brc30db.get_stake_to_all_pid(&pledged_tick_unknown).unwrap(),
      vec![pid_unknown.clone()]
    );
  }

  #[test]
  fn test_all_get_transferable() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30DataStore::new(&wtx);

    let script_key1 = ScriptKey::from_address(
      Address::from_str("bc1pgllnmtxs0g058qz7c6qgaqq4qknwrqj9z7rqn9e2dzhmcfmhlu4sfadf5e").unwrap(),
    );
    let tick_id1 = TickId::from_str("17c515d6b7").unwrap();
    let inscription_id1 =
      InscriptionId::from_str("1111111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();
    let transferable_asset1 = TransferableAsset {
      inscription_id: inscription_id1,
      amount: 100,
      tick_id: tick_id1.clone(),
      owner: script_key1.clone(),
    };

    brc30db
      .set_transferable_assets(
        &script_key1,
        &tick_id1,
        &inscription_id1,
        &transferable_asset1.clone(),
      )
      .unwrap();

    assert_eq!(
      brc30db
        .get_transferable_asset(&script_key1, &tick_id1, &inscription_id1)
        .unwrap()
        .unwrap(),
      transferable_asset1
    );

    let tick_id2 = TickId::from_str("f7c515d6b7").unwrap();
    let inscription_id2 =
      InscriptionId::from_str("2111111111111111111111111111111111111111111111111111111111111111i1")
        .unwrap();
    let transferable_asset2 = TransferableAsset {
      inscription_id: inscription_id2,
      amount: 100,
      tick_id: tick_id2.clone(),
      owner: script_key1.clone(),
    };

    brc30db
      .set_transferable_assets(
        &script_key1,
        &tick_id2,
        &inscription_id2,
        &transferable_asset2.clone(),
      )
      .unwrap();

    assert_eq!(
      brc30db
        .get_transferable_asset(&script_key1, &tick_id2, &inscription_id2)
        .unwrap()
        .unwrap(),
      transferable_asset2
    );

    assert_eq!(
      brc30db.get_transferable(&script_key1).unwrap(),
      vec![transferable_asset1.clone(), transferable_asset2.clone()]
    );
  }
}
