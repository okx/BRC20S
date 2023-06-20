use super::*;
use crate::okx::datastore::BRC30::{
  BRC30DataStoreReadOnly, BRC30DataStoreReadWrite, BRC30Receipt, Balance, InscriptionOperation,
  Pid, PoolInfo, PoolType, StakeInfo, TickId, TickInfo, TransferableAsset, UserInfo,
};

use crate::okx::datastore::BRC20::Tick;

use crate::InscriptionId;
use bitcoin::Txid;
use redb::WriteTransaction;

pub struct BRC30DataStore<'db, 'a> {
  wtx: &'a WriteTransaction<'db>,
}

impl<'db, 'a> BRC30DataStore<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self { wtx }
  }
}

impl<'db, 'a> BRC30DataStoreReadOnly for BRC30DataStore<'db, 'a> {
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

  // 3.3.4 BRC30_PID_TO_POOLINFO
  fn get_pid_to_poolinfo(&self, pid: &Pid) -> Result<Option<PoolInfo>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_pid_to_poolinfo(pid)
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

  fn get_transferable_by_id(
    &self,
    script: &ScriptKey,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableAsset>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_transferable_by_id(script, inscription_id)
  }

  // 3.3.10 BRC30_TXID_TO_RECEIPTS
  fn get_txid_to_receipts(&self, tx_id: &Txid) -> Result<Vec<BRC30Receipt>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_txid_to_receipts(tx_id)
  }

  fn get_transaction_receipts(&self, tx_id: &Txid) -> Result<Vec<BRC30Receipt>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_transaction_receipts(tx_id)
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
  fn set_txid_to_receipts(&self, tx_id: &Txid, receipt: &BRC30Receipt) -> Result<(), Self::Error> {
    let mut receipts = self.get_transaction_receipts(tx_id)?;
    receipts.push(receipt.clone());
    self.save_transaction_receipts(tx_id, &receipts)
  }

  fn save_transaction_receipts(
    &self,
    tx_id: &Txid,
    receipts: &[BRC30Receipt],
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::BRC30::{
    BRC30DataStoreReadOnly, BRC30DataStoreReadWrite, BRC30OperationType,
  };
  use crate::okx::datastore::BRC30::{BRC30Tick, Pid, PledgedTick, PoolType, TickId};
  use crate::okx::protocol::BRC30::operation::BRC30Operation;
  use crate::okx::protocol::BRC30::BRC30Error;
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
    let tick1 = TickId::from_str("abcdd").unwrap();
    let tick2 = TickId::from_str("12345").unwrap();
    let tick3 = TickId::from_str(";23!@").unwrap();
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

    let pid = Pid::from_str("1234567890#01").unwrap();

    let pool_info = PoolInfo {
      pid: pid.clone(),
      ptype: PoolType::Pool,
      inscription_id: inscription_id.clone(),
      stake: PledgedTick::NATIVE,
      erate: 0,
      minted: 0,
      staked: 0,
      dmax: 0,
      acc_reward_per_share: 0,
      last_update_block: 0,
      only: true,
    };

    brc30db.set_pid_to_poolinfo(&pid, &pool_info).unwrap();

    assert_eq!(
      brc30db.get_pid_to_poolinfo(&pid).unwrap().unwrap(),
      pool_info
    );
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

    let pledged_tick_20 = PledgedTick::BRC20Tick(Tick::from_str("tk20").unwrap());
    let pid_20 = Pid::from_str("0000000000#01").unwrap();
    let stake_info_20 = StakeInfo {
      stake: pledged_tick_20.clone(),
      max_share: 123,
      total_only: 123,
      pool_stakes: vec![(pid_20.clone(), true, 123)],
    };

    let pledged_tick_30 = PledgedTick::BRC30Tick(TickId::from_str("tck30").unwrap());
    let pid_30 = Pid::from_str("0000000000#02").unwrap();
    let stake_info_30 = StakeInfo {
      stake: pledged_tick_30.clone(),
      max_share: 123,
      total_only: 123,
      pool_stakes: vec![(pid_30.clone(), true, 123)],
    };

    let pledged_tick_btc = PledgedTick::BRC30Tick(TickId::from_str("btc00").unwrap());
    let pid_btc = Pid::from_str("0000000000#03").unwrap();
    let stake_info_btc = StakeInfo {
      stake: pledged_tick_btc.clone(),
      max_share: 123,
      total_only: 123,
      pool_stakes: vec![(pid_btc.clone(), true, 123)],
    };

    brc30db
      .set_user_stakeinfo(&script, &pledged_tick_20, &stake_info_20)
      .unwrap();

    brc30db
      .set_user_stakeinfo(&script, &pledged_tick_20, &stake_info_30)
      .unwrap();

    brc30db
      .set_user_stakeinfo(&script, &pledged_tick_20, &stake_info_btc)
      .unwrap();

    assert_eq!(
      brc30db
        .get_user_stakeinfo(&script, &pledged_tick_20)
        .unwrap()
        .unwrap(),
      stake_info_20
    );

    assert_eq!(
      brc30db
        .get_user_stakeinfo(&script, &pledged_tick_30)
        .unwrap()
        .unwrap(),
      stake_info_30
    );

    assert_eq!(
      brc30db
        .get_user_stakeinfo(&script, &pledged_tick_btc)
        .unwrap()
        .unwrap(),
      stake_info_btc
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
    let tick_id = TickId::from_str("abcdd").unwrap();

    let pid = Pid::from_str("1234567890#01").unwrap();
    let tick_info = TickInfo {
      tick_id: tick_id.clone(),
      name: BRC30Tick::from_str("aBc1ab").unwrap(),
      inscription_id: inscription_id.clone(),
      allocated: 100,
      decimal: 8,
      minted: 100,
      supply: 100,
      deployer: ScriptKey::from_address(
        Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap(),
      ),
      deploy_block: 100,
      latest_mint_block: 100,
      pids: vec![pid],
    };

    brc30db.set_tick_info(&tick_id, &tick_info).unwrap();

    assert_eq!(brc30db.get_tick_info(&tick_id).unwrap().unwrap(), tick_info);
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
      reward: 0,
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
    let tick_id = TickId::from_str("abcdd").unwrap();
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
      BRC30Receipt {
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
        op: BRC30OperationType::Transfer,
        from: ScriptKey::Address(addr.clone()),
        to: ScriptKey::Address(addr.clone()),
        result: Err(BRC30Error::InvalidTickLen("abcde".to_string())),
      },
      BRC30Receipt {
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
        op: BRC30OperationType::Transfer,
        from: ScriptKey::Address(addr.clone()),
        to: ScriptKey::Address(addr.clone()),
        result: Err(BRC30Error::InvalidTickLen("abcde".to_string())),
      },
      BRC30Receipt {
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
        op: BRC30OperationType::Transfer,
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

    let pledged_tick_20 = PledgedTick::BRC20Tick(Tick::from_str("tk20").unwrap());
    let pid_20 = Pid::from_str("1234567890#01").unwrap();
    let stake_info_20 = StakeInfo {
      stake: pledged_tick_20.clone(),
      max_share: 123,
      total_only: 123,
      pool_stakes: vec![(pid_20.clone(), true, 123)],
    };

    let pledged_tick_30 = PledgedTick::BRC30Tick(TickId::from_str("tck30").unwrap());
    let pid_30 = Pid::from_str("1234567891#01").unwrap();
    let stake_info_30 = StakeInfo {
      stake: pledged_tick_30.clone(),
      max_share: 123,
      total_only: 123,
      pool_stakes: vec![(pid_30.clone(), true, 123)],
    };

    let tick1 = TickId::from_str("abcdd").unwrap();
    let tick2 = TickId::from_str("12345").unwrap();
    let tick3 = TickId::from_str(";23!@").unwrap();

    let pledged_tick_btc = PledgedTick::BRC30Tick(TickId::from_str("btc00").unwrap());
    let pid_btc = Pid::from_str("1234567890#01").unwrap();
    let stake_info_btc = StakeInfo {
      stake: pledged_tick_btc.clone(),
      max_share: 123,
      total_only: 123,
      pool_stakes: vec![(pid_btc.clone(), true, 123)],
    };

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

    let pledged_tick_20 = PledgedTick::BRC20Tick(Tick::from_str("tk20").unwrap());
    let pid_20 = Pid::from_str("1234567890#01").unwrap();
    let stake_info_20 = StakeInfo {
      stake: pledged_tick_20.clone(),
      max_share: 123,
      total_only: 123,
      pool_stakes: vec![(pid_20.clone(), true, 123)],
    };

    let pledged_tick_30 = PledgedTick::BRC30Tick(TickId::from_str("tck30").unwrap());
    let pid_30 = Pid::from_str("1234567891#01").unwrap();
    let stake_info_30 = StakeInfo {
      stake: pledged_tick_30.clone(),
      max_share: 123,
      total_only: 123,
      pool_stakes: vec![(pid_30.clone(), true, 123)],
    };

    let tick1 = TickId::from_str("abcdd").unwrap();
    let tick2 = TickId::from_str("12345").unwrap();
    let tick3 = TickId::from_str(";23!@").unwrap();

    let pledged_tick_btc = PledgedTick::BRC30Tick(TickId::from_str("btc00").unwrap());
    let pid_btc = Pid::from_str("1234567892#01").unwrap();
    let stake_info_btc = StakeInfo {
      stake: pledged_tick_btc.clone(),
      max_share: 123,
      total_only: 123,
      pool_stakes: vec![(pid_btc.clone(), true, 123)],
    };

    brc30db
      .set_tickid_stake_to_pid(&tick1, &pledged_tick_20, &pid_20)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick1, &pledged_tick_30, &pid_30)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick1, &pledged_tick_btc, &pid_btc)
      .unwrap();

    brc30db
      .set_tickid_stake_to_pid(&tick2, &pledged_tick_20, &pid_20)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick2, &pledged_tick_30, &pid_30)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick2, &pledged_tick_btc, &pid_btc)
      .unwrap();

    brc30db
      .set_tickid_stake_to_pid(&tick3, &pledged_tick_20, &pid_20)
      .unwrap();
    brc30db
      .set_tickid_stake_to_pid(&tick3, &pledged_tick_30, &pid_30)
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
        .get_tickid_stake_to_pid(&tick1, &pledged_tick_30)
        .unwrap()
        .unwrap(),
      pid_30
    );

    assert_eq!(
      brc30db
        .get_tickid_stake_to_pid(&tick1, &pledged_tick_btc)
        .unwrap()
        .unwrap(),
      pid_btc
    );

    assert_eq!(
      brc30db.get_tickid_to_all_pid(&tick1).unwrap(),
      vec![pid_btc.clone(), pid_30.clone(), pid_20.clone()]
    );

    assert_eq!(
      brc30db.get_stake_to_all_pid(&pledged_tick_30).unwrap(),
      vec![pid_30.clone(), pid_30.clone(), pid_30.clone()]
    );
  }
}
