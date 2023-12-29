use crate::index::{InscriptionEntryValue, InscriptionIdValue, OutPointValue};
use crate::inscription_id::InscriptionId;
use crate::okx::datastore::brc20::redb::table::{
  add_transaction_receipt, get_balance, get_balances, get_inscribe_transfer_inscription,
  get_token_info, get_tokens_info, get_transaction_receipts, get_transferable,
  get_transferable_by_id, get_transferable_by_tick, insert_inscribe_transfer_inscription,
  insert_token_info, insert_transferable, remove_inscribe_transfer_inscription,
  remove_transferable, save_transaction_receipts, update_mint_token_info, update_token_balance,
};
use crate::okx::datastore::brc20::{
  Balance, Brc20Reader, Brc20ReaderWriter, Receipt, Tick, TokenInfo, TransferInfo, TransferableLog,
};
use crate::okx::datastore::ord::collections::CollectionKind;
use crate::okx::datastore::ord::redb::table::save_transaction_operations;
use crate::okx::datastore::ord::redb::table::{
  get_collection_inscription_id, get_collections_of_inscription, get_number_by_inscription_id,
  get_transaction_operations, get_txout_by_outpoint, set_inscription_attributes,
  set_inscription_by_collection_key,
};
use crate::okx::datastore::ord::{InscriptionOp, OrdReader, OrdReaderWriter};
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::BlockContext;
use bitcoin::{OutPoint, TxOut, Txid};
use redb::Table;
use std::collections::HashMap;

#[allow(non_snake_case)]
pub struct Context<'a, 'db, 'txn> {
  pub(crate) chain: BlockContext,
  pub(crate) tx_out_cache: &'a mut HashMap<OutPoint, TxOut>,

  // ord tables
  pub(crate) ORD_TX_TO_OPERATIONS: Table<'db, 'txn, &'static str, &'static [u8]>,
  pub(crate) COLLECTIONS_KEY_TO_INSCRIPTION_ID: Table<'db, 'txn, &'static str, &'static [u8; 36]>,
  pub(crate) COLLECTIONS_INSCRIPTION_ID_TO_KINDS:
    Table<'db, 'txn, &'static [u8; 36], &'static [u8]>,
  pub(crate) INSCRIPTION_ID_TO_INSCRIPTION_ENTRY:
    Table<'db, 'txn, &'static InscriptionIdValue, InscriptionEntryValue>,
  pub(crate) OUTPOINT_TO_ENTRY: Table<'db, 'txn, &'static OutPointValue, &'static [u8]>,

  // BRC20 tables
  pub(crate) BRC20_BALANCES: Table<'db, 'txn, &'static str, &'static [u8]>,
  pub(crate) BRC20_TOKEN: Table<'db, 'txn, &'static str, &'static [u8]>,
  pub(crate) BRC20_EVENTS: Table<'db, 'txn, &'static str, &'static [u8]>,
  pub(crate) BRC20_TRANSFERABLELOG: Table<'db, 'txn, &'static str, &'static [u8]>,
  pub(crate) BRC20_INSCRIBE_TRANSFER: Table<'db, 'txn, &'static [u8; 36], &'static [u8]>,
}

impl<'a, 'db, 'txn> OrdReader for Context<'a, 'db, 'txn> {
  type Error = anyhow::Error;

  fn get_number_by_inscription_id(
    &self,
    inscription_id: &InscriptionId,
  ) -> crate::Result<Option<i64>, Self::Error> {
    get_number_by_inscription_id(&self.INSCRIPTION_ID_TO_INSCRIPTION_ENTRY, inscription_id)
  }

  fn get_outpoint_to_txout(
    &self,
    outpoint: &OutPoint,
  ) -> crate::Result<Option<TxOut>, Self::Error> {
    get_txout_by_outpoint(&self.OUTPOINT_TO_ENTRY, outpoint)
  }

  fn get_transaction_operations(
    &self,
    txid: &Txid,
  ) -> crate::Result<Vec<InscriptionOp>, Self::Error> {
    get_transaction_operations(&self.ORD_TX_TO_OPERATIONS, txid)
  }

  fn get_collections_of_inscription(
    &self,
    inscription_id: &InscriptionId,
  ) -> crate::Result<Option<Vec<CollectionKind>>, Self::Error> {
    get_collections_of_inscription(&self.COLLECTIONS_INSCRIPTION_ID_TO_KINDS, inscription_id)
  }

  fn get_collection_inscription_id(
    &self,
    collection_key: &str,
  ) -> crate::Result<Option<InscriptionId>, Self::Error> {
    get_collection_inscription_id(&self.COLLECTIONS_KEY_TO_INSCRIPTION_ID, collection_key)
  }
}

impl<'a, 'db, 'txn> OrdReaderWriter for Context<'a, 'db, 'txn> {
  fn save_transaction_operations(
    &mut self,
    txid: &Txid,
    operations: &[InscriptionOp],
  ) -> crate::Result<(), Self::Error> {
    save_transaction_operations(&mut self.ORD_TX_TO_OPERATIONS, txid, operations)
  }

  fn set_inscription_by_collection_key(
    &mut self,
    key: &str,
    inscription_id: &InscriptionId,
  ) -> crate::Result<(), Self::Error> {
    set_inscription_by_collection_key(
      &mut self.COLLECTIONS_KEY_TO_INSCRIPTION_ID,
      key,
      inscription_id,
    )
  }

  fn set_inscription_attributes(
    &mut self,
    inscription_id: &InscriptionId,
    kind: &[CollectionKind],
  ) -> crate::Result<(), Self::Error> {
    set_inscription_attributes(
      &mut self.COLLECTIONS_INSCRIPTION_ID_TO_KINDS,
      inscription_id,
      kind,
    )
  }
}

impl<'a, 'db, 'txn> Brc20Reader for Context<'a, 'db, 'txn> {
  type Error = anyhow::Error;

  fn get_balances(&self, script_key: &ScriptKey) -> crate::Result<Vec<Balance>, Self::Error> {
    get_balances(&self.BRC20_BALANCES, script_key)
  }

  fn get_balance(
    &self,
    script_key: &ScriptKey,
    tick: &Tick,
  ) -> crate::Result<Option<Balance>, Self::Error> {
    get_balance(&self.BRC20_BALANCES, script_key, tick)
  }

  fn get_token_info(&self, tick: &Tick) -> crate::Result<Option<TokenInfo>, Self::Error> {
    get_token_info(&self.BRC20_TOKEN, tick)
  }

  fn get_tokens_info(&self) -> crate::Result<Vec<TokenInfo>, Self::Error> {
    get_tokens_info(&self.BRC20_TOKEN)
  }

  fn get_transaction_receipts(&self, txid: &Txid) -> crate::Result<Vec<Receipt>, Self::Error> {
    get_transaction_receipts(&self.BRC20_EVENTS, txid)
  }

  fn get_transferable(
    &self,
    script: &ScriptKey,
  ) -> crate::Result<Vec<TransferableLog>, Self::Error> {
    get_transferable(&self.BRC20_TRANSFERABLELOG, script)
  }

  fn get_transferable_by_tick(
    &self,
    script: &ScriptKey,
    tick: &Tick,
  ) -> crate::Result<Vec<TransferableLog>, Self::Error> {
    get_transferable_by_tick(&self.BRC20_TRANSFERABLELOG, script, tick)
  }

  fn get_transferable_by_id(
    &self,
    script: &ScriptKey,
    inscription_id: &InscriptionId,
  ) -> crate::Result<Option<TransferableLog>, Self::Error> {
    get_transferable_by_id(&self.BRC20_TRANSFERABLELOG, script, inscription_id)
  }

  fn get_inscribe_transfer_inscription(
    &self,
    inscription_id: &InscriptionId,
  ) -> crate::Result<Option<TransferInfo>, Self::Error> {
    get_inscribe_transfer_inscription(&self.BRC20_INSCRIBE_TRANSFER, inscription_id)
  }
}

impl<'a, 'db, 'txn> Brc20ReaderWriter for Context<'a, 'db, 'txn> {
  fn update_token_balance(
    &mut self,
    script_key: &ScriptKey,
    new_balance: Balance,
  ) -> crate::Result<(), Self::Error> {
    update_token_balance(&mut self.BRC20_BALANCES, script_key, new_balance)
  }

  fn insert_token_info(
    &mut self,
    tick: &Tick,
    new_info: &TokenInfo,
  ) -> crate::Result<(), Self::Error> {
    insert_token_info(&mut self.BRC20_TOKEN, tick, new_info)
  }

  fn update_mint_token_info(
    &mut self,
    tick: &Tick,
    minted_amt: u128,
    minted_block_number: u64,
  ) -> crate::Result<(), Self::Error> {
    update_mint_token_info(&mut self.BRC20_TOKEN, tick, minted_amt, minted_block_number)
  }

  fn save_transaction_receipts(
    &mut self,
    txid: &Txid,
    receipts: &[Receipt],
  ) -> crate::Result<(), Self::Error> {
    save_transaction_receipts(&mut self.BRC20_EVENTS, txid, receipts)
  }

  fn add_transaction_receipt(
    &mut self,
    txid: &Txid,
    receipt: &Receipt,
  ) -> crate::Result<(), Self::Error> {
    add_transaction_receipt(&mut self.BRC20_EVENTS, txid, receipt)
  }

  fn insert_transferable(
    &mut self,
    script: &ScriptKey,
    tick: &Tick,
    inscription: TransferableLog,
  ) -> crate::Result<(), Self::Error> {
    insert_transferable(&mut self.BRC20_TRANSFERABLELOG, script, tick, inscription)
  }

  fn remove_transferable(
    &mut self,
    script: &ScriptKey,
    tick: &Tick,
    inscription_id: &InscriptionId,
  ) -> crate::Result<(), Self::Error> {
    remove_transferable(
      &mut self.BRC20_TRANSFERABLELOG,
      script,
      tick,
      inscription_id,
    )
  }

  fn insert_inscribe_transfer_inscription(
    &mut self,
    inscription_id: &InscriptionId,
    transfer_info: TransferInfo,
  ) -> crate::Result<(), Self::Error> {
    insert_inscribe_transfer_inscription(
      &mut self.BRC20_INSCRIBE_TRANSFER,
      inscription_id,
      transfer_info,
    )
  }

  fn remove_inscribe_transfer_inscription(
    &mut self,
    inscription_id: &InscriptionId,
  ) -> crate::Result<(), Self::Error> {
    remove_inscribe_transfer_inscription(&mut self.BRC20_INSCRIBE_TRANSFER, inscription_id)
  }
}
