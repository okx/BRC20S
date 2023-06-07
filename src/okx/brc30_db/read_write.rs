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
    inscriptionOperations: &InscriptionOperation,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(TXID_TO_INSCRIPTION_RECEIPTS)?.insert(
      tx_id.to_string().as_str(),
      bincode::serialize(inscriptionOperations)
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
  fn set_balance(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    balance: &Balance,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(BRC30_BALANCE)?.insert(
      script_tickid_key(script_key, tick_id).as_str(),
      bincode::serialize(balance).unwrap().as_slice(),
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
