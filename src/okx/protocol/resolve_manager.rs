use super::*;
use std::collections::{HashMap, HashSet};

use crate::{
  okx::{
    datastore::{
      ord::operation::InscriptionOp, BRC20DataStoreReadWrite, BRC20SDataStoreReadWrite,
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
  M: BRC20SDataStoreReadWrite,
> {
  protocols: HashSet<ProtocolKind>,
  client: &'a Client,
  ord_store: &'a O,
  brc20_store: &'a N,
  brc20s_store: &'a M,
  first_brc20_height: u64,
  first_brc20s_height: u64,
}

impl<'a, O: OrdDataStoreReadWrite, N: BRC20DataStoreReadWrite, M: BRC20SDataStoreReadWrite>
  MsgResolveManager<'a, O, N, M>
{
  pub fn new(
    client: &'a Client,
    ord_store: &'a O,
    brc20_store: &'a N,
    brc20s_store: &'a M,
    first_brc20_height: u64,
    first_brc20s_height: u64,
  ) -> Self {
    let mut protocols: HashSet<ProtocolKind> = HashSet::new();
    protocols.insert(ProtocolKind::BRC20);
    protocols.insert(ProtocolKind::BRC20S);
    Self {
      protocols,
      client,
      ord_store,
      brc20_store,
      brc20s_store,
      first_brc20_height,
      first_brc20s_height,
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
      // TODO: BTC transfer to BRC20S passive withdrawal.

      // if self.protocols.contains(&ProtocolKind::BRC20S) {
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
        if self.protocols.contains(&ProtocolKind::BRC20)
          && context.blockheight >= self.first_brc20_height
        {
          if let Some(msg) =
            brc20::resolve_message(self.brc20_store, &new_inscriptions, &operation)?
              .map(Message::BRC20)
          {
            messages.push(msg);
            continue;
          }
        }

        // Parse BRC20S message through inscription operation.
        if self.protocols.contains(&ProtocolKind::BRC20S)
          && context.blockheight >= self.first_brc20s_height
        {
          if let Some(msg) = brc20s::resolve_message(
            self.client,
            self.ord_store,
            self.brc20s_store,
            &new_inscriptions,
            &operation,
            &mut outpoint_to_txout_cache,
          )?
          .map(Message::BRC20S)
          {
            messages.push(msg);
            continue;
          }
        }
      }
    }
    for (outpoint, txout) in outpoint_to_txout_cache {
      self
        .ord_store
        .set_outpoint_to_txout(outpoint, &txout)
        .or(Err(anyhow!(
          "failed to get tx out! error: {} not found",
          outpoint
        )))?;
    }
    Ok(messages)
  }
}
