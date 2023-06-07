use super::*;
use crate::brc30::ledger::{LedgerRead, LedgerReadWrite};
use crate::brc30::{
  BRC30PoolInfo, BRC30Receipt, BRC30TickInfo, Balance, InscriptionOperation, Pid, TickId,
  TransferableAsset, UserInfo,
};
use crate::InscriptionId;
use bitcoin::Script;
use bitcoin::Txid;
use redb::WriteTransaction;

pub struct BRC30Database<'db, 'a> {
  wtx: &'a WriteTransaction<'db>,
}

impl<'db, 'a> BRC30Database<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self { wtx }
  }
}

impl<'db, 'a> LedgerRead for BRC30Database<'db, 'a> {
  type Error = redb::Error;

  // 3.3.1 OUTPOINT_TO_SCRIPT, todo, replace outpoint
  fn get_outpoint_to_script(&self, outpoint: &str) -> Result<Option<Script>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_outpoint_to_script(outpoint)
  }

  //3.3.2 TXID_TO_INSCRIPTION_RECEIPTS, todo, replace <Vec<InscriptionOperation>
  fn get_txid_to_inscription_receipts(
    &self,
    txid: &Txid,
  ) -> Result<Vec<InscriptionOperation>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_txid_to_inscription_receipts(txid)
  }

  // 3.3.3 BRC30_TICKINFO
  fn get_tick_info(&self, tick_id: &TickId) -> Result<Option<BRC30TickInfo>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_tick_info(tick_id)
  }

  // 3.3.4 BRC30_PID_TO_POOLINFO
  fn get_pid_to_poolinfo(&self, pid: &Pid) -> Result<Option<BRC30PoolInfo>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_pid_to_poolinfo(pid)
  }

  // 3.3.5 BRC30_PID_TO_USERINFO
  fn get_pid_to_use_info(&self, pid: &Pid) -> Result<Option<UserInfo>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_pid_to_use_info(pid)
  }

  // 3.3.6 BRC30_STAKE_TICKID_TO_PID 和 BRC30_TICKID_STAKE_TO_PID, TODO zhujianguo
  // fn get_stake_tick_id_to_pid(&self);

  // 3.3.7 BRC30_BALANCE
  fn get_balance(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
  ) -> Result<Option<Balance>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_balance(script_key, tick_id)
  }

  fn get_balances(&self, script_key: &ScriptKey) -> Result<Vec<(TickId, Balance)>, Self::Error>{
    read_only::new_with_wtx(self.wtx).get_balances(script_key)
  }

  // 3.3.8 BRC30_TRANSFERABLE_ASSETS
  fn get_transferable_assets(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableAsset>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_transferable_assets(script_key, tick_id, inscription_id)
  }

  // 3.3.9 BRC30_TXID_TO_RECEIPTS, TODO replace BRC30ActionReceipt
  fn get_txid_to_receipts(&self, tick_id: &TickId) -> Result<Vec<BRC30Receipt>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_txid_to_receipts(tick_id)
  }
}

impl<'db, 'a> LedgerReadWrite for BRC30Database<'db, 'a> {
  // 3.3.1 OUTPOINT_TO_SCRIPT, todo, replace outpoint
  fn set_outpoint_to_script(&self, outpoint: &str, script: &Script) -> Result<(), Self::Error> {
    self
      .wtx
      .open_table(OUTPOINT_TO_SCRIPT)?
      .insert(outpoint, bincode::serialize(script).unwrap().as_slice())?;
    Ok(())
  }

  //3.3.2 TXID_TO_INSCRIPTION_RECEIPTS, todo, replace <Vec<InscriptionOperation>
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
  fn set_tick_info(
    &self,
    tick_id: &TickId,
    brc30_tick_info: &BRC30TickInfo,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_TICKINFO)?.insert(
      tick_id.as_str(),
      bincode::serialize(brc30_tick_info).unwrap().as_slice(),
    )?;
    Ok(())
  }

  // 3.3.4 BRC30_PID_TO_POOLINFO
  fn set_pid_to_poolinfo(
    &self,
    pid: &Pid,
    brc30_pool_info: &BRC30PoolInfo,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_PID_TO_POOLINFO)?.insert(
      pid.as_str(),
      bincode::serialize(brc30_pool_info).unwrap().as_slice(),
    )?;
    Ok(())
  }

  // 3.3.5 BRC30_PID_TO_USERINFO
  fn set_pid_to_use_info(&self, pid: &Pid, user_info: &UserInfo) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_PID_TO_USERINFO)?.insert(
      pid.as_str(),
      bincode::serialize(user_info).unwrap().as_slice(),
    )?;
    Ok(())
  }

  // 3.3.6 BRC30_STAKE_TICKID_TO_PID 和 BRC30_TICKID_STAKE_TO_PID, TODO zhujianguo
  fn set_stake_tick_id_to_pid(&self) -> Result<(), Self::Error> {
    // self.wtx.open_table(BRC30_PID_TO_USERINFO)?.insert(
    //   pid.as_str(),
    //   bincode::serialize(user_info).unwrap().as_slice(),
    // )?;
    Ok(())
  }

  // 3.3.7 BRC30_BALANCE
  fn update_token_balance(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    balance: Balance,
  ) -> Result<(), Self::Error> {
    let bal = StoreBalance {
      tick_id: tick_id.clone(),
      balance,
    };
    self.wtx.open_table(BRC30_BALANCES)?.insert(
      script_tickid_key(script_key, tick_id).as_str(),
      bincode::serialize(&bal).unwrap().as_slice(),
    )?;
    Ok(())
  }

  // 3.3.8 BRC30_TRANSFERABLE_ASSETS
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

  // 3.3.9 BRC30_TXID_TO_RECEIPTS
  fn save_txid_to_receipts(
    &self,
    txid: &Txid,
    receipts: &[BRC30Receipt],
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_TRANSFERABLE_ASSETS)?.insert(
      txid.to_string().as_str(),
      bincode::serialize(receipts).unwrap().as_slice(),
    )?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::SatPoint;
  use bitcoin::Address;
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;
  use crate::brc30::ledger::LedgerReadWrite;

  #[test]
  fn test_balances() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30Database::new(&wtx);

    let script = ScriptKey::from_address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
    );
    let tick1 = TickId::from_str("abcdd").unwrap();
    let tick2 = TickId::from_str("12345").unwrap();
    let tick3 = TickId::from_str(";23!@").unwrap();
    let expect_balance1 = Balance {
      overall_balance: 10,
      transferable_balance: 10,
    };
    let expect_balance2 = Balance {
      overall_balance: 30,
      transferable_balance: 30,
    };
    let expect_balance3 = Balance {
      overall_balance: 100,
      transferable_balance: 30,
    };
    brc30db
      .update_token_balance(&script, &tick1, expect_balance1.clone())
      .unwrap();
    brc30db
      .update_token_balance(&script, &tick2, expect_balance2.clone())
      .unwrap();
    brc30db
      .update_token_balance(&script, &tick3, expect_balance3.clone())
      .unwrap();

    assert_eq!(brc30db.get_balance(&script, &tick1).unwrap().unwrap(), expect_balance1);

    let script2 =
      ScriptKey::from_address(Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap());
    assert_ne!(script.to_string(), script2.to_string());
    let expect_balance22 = Balance {
      overall_balance: 100,
      transferable_balance: 30,
    };
    brc30db
      .update_token_balance(&script2, &tick1, expect_balance22.clone())
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
  fn test_outpoint_to_script() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30Database::new(&wtx);

    let outpoint1: &str = "outpoint-1";
    let script= Script::from_str("12345678").unwrap();

    brc30db
      .set_outpoint_to_script(&outpoint1, &script)
      .unwrap();

    assert_eq!(brc30db.get_outpoint_to_script(&outpoint1).unwrap().unwrap(), script);
  }

  #[test]
  fn test_txid_to_inscription_receipts() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = BRC30Database::new(&wtx);

    let txid =
      Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();

   let op_vec = vec![InscriptionOperation{
      txid: Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap(),
   }, InscriptionOperation{
     txid: Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap(),
   }, InscriptionOperation{
     txid: Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap(),
   }, ];

    brc30db
      .set_txid_to_inscription_receipts(&txid, &op_vec)
      .unwrap();

    assert_eq!(brc30db.get_txid_to_inscription_receipts(&txid).unwrap(), op_vec);
  }
}
