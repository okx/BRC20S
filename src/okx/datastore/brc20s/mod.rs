mod balance;
mod event;
pub mod pool_info;
pub mod redb;
mod stake_info;
mod temp;
mod tick;
mod transfer;
mod user_info;

pub use self::{
  balance::*, event::*, pool_info::*, stake_info::*, temp::*, tick::*, transfer::*, user_info::*,
};
use crate::okx::datastore::ScriptKey;
use crate::InscriptionId;
use bitcoin::Txid;
use std::fmt::{Debug, Display};

pub trait DataStoreReadOnly {
  type Error: Debug + Display;

  // TXID_TO_INSCRIPTION_RECEIPTS
  fn get_txid_to_inscription_receipts(
    &self,
    txid: &Txid,
  ) -> Result<Vec<InscriptionOperation>, Self::Error>;

  // BRC20S_TICKINFO
  fn get_tick_info(&self, tick_id: &TickId) -> Result<Option<TickInfo>, Self::Error>;

  fn get_all_tick_info(
    &self,
    start: usize,
    limit: Option<usize>,
  ) -> Result<(Vec<TickInfo>, usize), Self::Error>;

  // BRC20S_PID_TO_POOLINFO
  fn get_pid_to_poolinfo(&self, pid: &Pid) -> Result<Option<PoolInfo>, Self::Error>;

  fn get_all_poolinfo(
    &self,
    start: usize,
    limit: Option<usize>,
  ) -> Result<(Vec<PoolInfo>, usize), Self::Error>;

  // BRC20S_USER_STAKEINFO
  fn get_user_stakeinfo(
    &self,
    script_key: &ScriptKey,
    pledged_tick: &PledgedTick,
  ) -> Result<Option<StakeInfo>, Self::Error>;

  // BRC20S_PID_TO_USERINFO
  fn get_pid_to_use_info(
    &self,
    script_key: &ScriptKey,
    pid: &Pid,
  ) -> Result<Option<UserInfo>, Self::Error>;

  // BRC20S_STAKE_TICKID_TO_PID
  fn get_tickid_stake_to_pid(
    &self,
    tick_id: &TickId,
    pledged: &PledgedTick,
  ) -> Result<Option<Pid>, Self::Error>;

  // get_tickid_to_all_pid
  fn get_tickid_to_all_pid(&self, tick_id: &TickId) -> Result<Vec<Pid>, Self::Error>;

  // get_stake_to_all_pid
  fn get_stake_to_all_pid(&self, pledged: &PledgedTick) -> Result<Vec<Pid>, Self::Error>;

  // BRC20S_BALANCE
  fn get_balance(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
  ) -> Result<Option<Balance>, Self::Error>;

  fn get_balances(&self, script_key: &ScriptKey) -> Result<Vec<(TickId, Balance)>, Self::Error>;

  // BRC20S_TRANSFERABLE_ASSETS
  fn get_transferable_asset(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableAsset>, Self::Error>;

  fn get_transferable(&self, script: &ScriptKey) -> Result<Vec<TransferableAsset>, Self::Error>;

  fn get_transferable_by_tickid(
    &self,
    script: &ScriptKey,
    tick_id: &TickId,
  ) -> Result<Vec<TransferableAsset>, Self::Error>;

  fn get_transferable_by_id(
    &self,
    script: &ScriptKey,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableAsset>, Self::Error>;

  // BRC20S_TXID_TO_RECEIPTS
  fn get_transaction_receipts(&self, txid: &Txid) -> Result<Vec<Receipt>, Self::Error>;
  fn get_txid_to_receipts(&self, txid: &Txid) -> Result<Vec<Receipt>, Self::Error>;

  fn get_inscribe_transfer_inscription(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<TransferInfo>, Self::Error>;
}

pub trait DataStoreReadWrite: DataStoreReadOnly {
  // TXID_TO_INSCRIPTION_RECEIPTS
  fn set_txid_to_inscription_receipts(
    &self,
    tx_id: &Txid,
    inscription_operations: &Vec<InscriptionOperation>,
  ) -> Result<(), Self::Error>;

  // BRC20S_TICKINFO
  fn set_tick_info(&self, tick_id: &TickId, tick_info: &TickInfo) -> Result<(), Self::Error>;

  // BRC20S_PID_TO_POOLINFO
  fn set_pid_to_poolinfo(&self, pid: &Pid, pool_info: &PoolInfo) -> Result<(), Self::Error>;

  // BRC20S_USER_STAKEINFO
  fn set_user_stakeinfo(
    &self,
    script_key: &ScriptKey,
    pledged_tick: &PledgedTick,
    stake_info: &StakeInfo,
  ) -> Result<(), Self::Error>;

  // BRC20S_PID_TO_USERINFO
  fn set_pid_to_use_info(
    &self,
    script_key: &ScriptKey,
    pid: &Pid,
    user_info: &UserInfo,
  ) -> Result<(), Self::Error>;

  // BRC20S_STAKE_TICKID_TO_PID, BRC20S_TICKID_STAKE_TO_PID
  fn set_tickid_stake_to_pid(
    &self,
    tick_id: &TickId,
    pledged: &PledgedTick,
    pid: &Pid,
  ) -> Result<(), Self::Error>;

  // BRC20S_BALANCE
  fn set_token_balance(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    balance: Balance,
  ) -> Result<(), Self::Error>;

  // BRC20S_TRANSFERABLE_ASSETS
  fn set_transferable_assets(
    &self,
    script_key: &ScriptKey,
    tick_id: &TickId,
    inscription_id: &InscriptionId,
    transferable_asset: &TransferableAsset,
  ) -> Result<(), Self::Error>;

  // BRC20S_TXID_TO_RECEIPTS
  fn add_transaction_receipt(&self, txid: &Txid, receipt: &Receipt) -> Result<(), Self::Error>;
  fn save_transaction_receipts(&self, tx_id: &Txid, receipt: &[Receipt])
    -> Result<(), Self::Error>;

  fn remove_transferable(
    &self,
    script: &ScriptKey,
    tick_id: &TickId,
    inscription_id: &InscriptionId,
  ) -> Result<(), Self::Error>;

  fn insert_inscribe_transfer_inscription(
    &self,
    inscription_id: InscriptionId,
    transfer_info: TransferInfo,
  ) -> Result<(), Self::Error>;

  fn remove_inscribe_transfer_inscription(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<(), Self::Error>;
}
