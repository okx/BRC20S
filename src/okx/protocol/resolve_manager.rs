use super::*;
use crate::okx::datastore::brc20 as brc20_store;
use crate::okx::datastore::brc20s as brc20s_store;
use crate::okx::datastore::ord as ord_store;
use std::collections::HashMap;

use crate::{
  okx::{datastore::ord::operation::InscriptionOp, protocol::Message},
  Inscription, Result,
};
use anyhow::anyhow;
use bitcoin::{OutPoint, Transaction, TxOut};
use bitcoincore_rpc::Client;
pub struct MsgResolveManager<
  'a,
  O: ord_store::OrdDataStoreReadWrite,
  N: brc20_store::DataStoreReadWrite,
  M: brc20s_store::DataStoreReadWrite,
> {
  client: &'a Client,
  ord_store: &'a O,
  brc20_store: &'a N,
  brc20s_store: &'a M,
  config: &'a Config,
}

impl<
    'a,
    O: ord_store::OrdDataStoreReadWrite,
    N: brc20_store::DataStoreReadWrite,
    M: brc20s_store::DataStoreReadWrite,
  > MsgResolveManager<'a, O, N, M>
{
  pub fn new(
    client: &'a Client,
    ord_store: &'a O,
    brc20_store: &'a N,
    brc20s_store: &'a M,
    config: &'a Config,
  ) -> Self {
    Self {
      client,
      ord_store,
      brc20_store,
      brc20s_store,
      config,
    }
  }

  pub fn resolve_message(
    &self,
    context: BlockContext,
    tx: &Transaction,
    operations: Vec<InscriptionOp>,
  ) -> Result<Vec<Message>> {
    log::debug!(
      "Resolve Manager indexed transaction {}, operations size: {}, data: {:?}",
      tx.txid(),
      operations.len(),
      operations
    );
    let mut messages = Vec::new();
    let mut operation_iter = operations.into_iter().peekable();
    let new_inscriptions = Inscription::from_transaction(tx)
      .into_iter()
      .map(|v| v.inscription)
      .collect::<Vec<Inscription>>();

    let mut outpoint_to_txout_cache: HashMap<OutPoint, TxOut> = HashMap::new();
    for input in &tx.input {
      // "operations" is a list of all the operations in the current block, and they are ordered.
      // We just need to find the operation corresponding to the current transaction here.
      while let Some(operation) = operation_iter.peek() {
        if operation.old_satpoint.outpoint != input.previous_output {
          break;
        }
        let operation = operation_iter.next().unwrap();

        // Parse BRC20 message through inscription operation.
        if self
          .config
          .first_brc20_height
          .map(|height| context.blockheight >= height)
          .unwrap_or(false)
        {
          if let Some(msg) =
            brc20::Message::resolve(self.brc20_store, &new_inscriptions, &operation)?
          {
            log::debug!(
              "BRC20 resolved the message from {:?}, msg {:?}",
              operation,
              msg
            );
            messages.push(Message::BRC20(msg));
            continue;
          }
        }

        // Parse BRC30 message through inscription operation.
        if self
          .config
          .first_brc20s_height
          .map(|height| context.blockheight >= height)
          .unwrap_or(false)
        {
          if let Some(msg) = brc20s::Message::resolve(
            self.client,
            self.ord_store,
            self.brc20s_store,
            &new_inscriptions,
            &operation,
            &mut outpoint_to_txout_cache,
          )? {
            log::debug!(
              "BRC20S resolved the message from {:?}, msg {:?}",
              operation,
              msg
            );
            messages.push(Message::BRC20S(msg));
            continue;
          }
        }
      }
    }
    self.update_outpoint_to_txout(outpoint_to_txout_cache)?;
    Ok(messages)
  }

  fn update_outpoint_to_txout(&self, outpoint_to_txout_cache: HashMap<OutPoint, TxOut>) -> Result {
    for (outpoint, txout) in outpoint_to_txout_cache {
      self
        .ord_store
        .set_outpoint_to_txout(outpoint, &txout)
        .or(Err(anyhow!(
          "failed to get tx out! error: {} not found",
          outpoint
        )))?;
    }
    Ok(())
  }
}
