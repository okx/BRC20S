pub(super) mod balance;
pub(super) mod errors;
pub(super) mod events;
pub mod redb;
pub(super) mod tick;
pub(super) mod token_info;
pub(super) mod transfer;
pub(super) mod transferable_log;

pub use self::{
  balance::Balance, errors::BRC20Error, events::Receipt, events::*, tick::*, token_info::TokenInfo,
  transfer::TransferInfo, transferable_log::TransferableLog,
};
use super::ScriptKey;
use crate::{InscriptionId, Result};
use bitcoin::Txid;
use std::collections::HashMap;
use std::fmt::{Debug, Display};

pub trait DataStoreReadOnly {
  type Error: Debug + Display;

  fn get_balances(&self, script_key: &ScriptKey) -> Result<Vec<Balance>, Self::Error>;
  fn get_balance(
    &self,
    script_key: &ScriptKey,
    tick: &Tick,
  ) -> Result<Option<Balance>, Self::Error>;

  fn get_acc_count(&self) -> Result<u64, Self::Error>;
  fn get_all_acc_balance(
    &self,
    start: usize,
    limit: Option<usize>,
  ) -> Result<HashMap<String, Vec<Balance>>, Self::Error>;

  fn get_token_info(&self, tick: &Tick) -> Result<Option<TokenInfo>, Self::Error>;
  fn get_tokens_info(&self) -> Result<Vec<TokenInfo>, Self::Error>;

  fn get_transaction_receipts(&self, txid: &Txid) -> Result<Vec<Receipt>, Self::Error>;

  fn get_transferable(&self, script: &ScriptKey) -> Result<Vec<TransferableLog>, Self::Error>;
  fn get_transferable_by_tick(
    &self,
    script: &ScriptKey,
    tick: &Tick,
  ) -> Result<Vec<TransferableLog>, Self::Error>;
  fn get_transferable_by_id(
    &self,
    script: &ScriptKey,
    inscription_id: &InscriptionId,
  ) -> Result<Option<TransferableLog>, Self::Error>;

  fn get_inscribe_transfer_inscription(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<TransferInfo>, Self::Error>;
}

pub trait DataStoreReadWrite: DataStoreReadOnly {
  fn update_token_balance(
    &self,
    script_key: &ScriptKey,
    new_balance: Balance,
  ) -> Result<(), Self::Error>;

  fn insert_token_info(&self, tick: &Tick, new_info: &TokenInfo) -> Result<(), Self::Error>;

  fn update_mint_token_info(
    &self,
    tick: &Tick,
    minted_amt: u128,
    minted_block_number: u64,
  ) -> Result<(), Self::Error>;

  fn save_transaction_receipts(&self, txid: &Txid, receipts: &[Receipt])
    -> Result<(), Self::Error>;

  fn add_transaction_receipt(&self, txid: &Txid, receipt: &Receipt) -> Result<(), Self::Error>;

  fn insert_transferable(
    &self,
    script: &ScriptKey,
    tick: &Tick,
    inscription: TransferableLog,
  ) -> Result<(), Self::Error>;

  fn remove_transferable(
    &self,
    script: &ScriptKey,
    tick: &Tick,
    inscription_id: InscriptionId,
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
