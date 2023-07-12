use super::*;
use std::collections::HashMap;

use crate::{
  okx::{
    datastore::{
      ord::operation::InscriptionOp, BRC20DataStoreReadWrite, BRC30DataStoreReadWrite,
      OrdDataStoreReadWrite,
    },
    protocol::Message,
  },
  Inscription, Result,
};
use anyhow::anyhow;
use bitcoin::{OutPoint, Transaction, TxOut};
use bitcoincore_rpc::Client;
pub struct MsgResolveManager<
  'a,
  O: OrdDataStoreReadWrite,
  N: BRC20DataStoreReadWrite,
  M: BRC30DataStoreReadWrite,
> {
  protocol_start_height: HashMap<ProtocolKind, u64>,
  client: &'a Client,
  ord_store: &'a O,
  brc20_store: &'a N,
  brc30_store: &'a M,
}

impl<'a, O: OrdDataStoreReadWrite, N: BRC20DataStoreReadWrite, M: BRC30DataStoreReadWrite>
  MsgResolveManager<'a, O, N, M>
{
  pub fn new(
    client: &'a Client,
    ord_store: &'a O,
    brc20_store: &'a N,
    brc30_store: &'a M,
    first_brc20_height: u64,
    first_brc20s_height: u64,
  ) -> Self {
    let mut protocol_start_height: HashMap<ProtocolKind, u64> = HashMap::new();
    protocol_start_height.insert(ProtocolKind::BRC20, first_brc20_height);
    protocol_start_height.insert(ProtocolKind::BRC30, first_brc20s_height);
    Self {
      client,
      ord_store,
      brc20_store,
      brc30_store,
      protocol_start_height,
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
      .collect();

    let mut outpoint_to_txout_cache: HashMap<OutPoint, TxOut> = HashMap::new();
    for input in &tx.input {
      // TODO: BTC transfer to BRC30 passive withdrawal.

      // if self.protocols.contains(&ProtocolKind::BRC30) {
      //   messages.push(BTC::resolve_message(txid, block_height, block_time, tx));
      // }

      // "operations" is a list of all the operations in the current block, and they are ordered.
      // We just need to find the operation corresponding to the current transaction here.
      while let Some(operation) = operation_iter.peek() {
        if operation.old_satpoint.outpoint != input.previous_output {
          break;
        }
        let operation = operation_iter.next().unwrap();

        // Parse BRC20 message through inscription operation.
        if self
          .protocol_start_height
          .get(&ProtocolKind::BRC20)
          .map(|height| context.blockheight >= height.clone())
          .unwrap_or(false)
        {
          if let Some(msg) =
            brc20::BRC20Message::resolve(self.brc20_store, &new_inscriptions, &operation)?
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
          .protocol_start_height
          .get(&ProtocolKind::BRC30)
          .map(|height| context.blockheight >= height.clone())
          .unwrap_or(false)
        {
          if let Some(msg) = brc30::BRC30Message::resolve(
            self.client,
            self.ord_store,
            self.brc30_store,
            &new_inscriptions,
            &operation,
            &mut outpoint_to_txout_cache,
          )? {
            log::debug!(
              "BRC20S resolved the message from {:?}, msg {:?}",
              operation,
              msg
            );
            messages.push(Message::BRC30(msg));
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
