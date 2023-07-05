use super::*;
use std::collections::{HashMap, HashSet};

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
use bitcoin::hashes::Hash;
use bitcoin::{OutPoint, Transaction, TxOut};
use bitcoincore_rpc::Client;
pub struct MsgResolveManager<
  'a,
  O: OrdDataStoreReadWrite,
  N: BRC20DataStoreReadWrite,
  M: BRC30DataStoreReadWrite,
> {
  protocols: HashSet<ProtocolKind>,
  client: &'a Client,
  ord_store: &'a O,
  brc20_store: &'a N,
  brc30_store: &'a M,
}

impl<'a, O: OrdDataStoreReadWrite, N: BRC20DataStoreReadWrite, M: BRC30DataStoreReadWrite>
  MsgResolveManager<'a, O, N, M>
{
  pub fn new(client: &'a Client, ord_store: &'a O, brc20_store: &'a N, brc30_store: &'a M) -> Self {
    let mut protocols: HashSet<ProtocolKind> = HashSet::new();
    protocols.insert(ProtocolKind::BRC20);
    protocols.insert(ProtocolKind::BRC30);
    Self {
      protocols,
      client,
      ord_store,
      brc20_store,
      brc30_store,
    }
  }

  pub fn resolve_message(
    &self,
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

        // filter coinbase transactions
        if let Some(point) = operation.new_satpoint {
          if point.outpoint.txid.eq(&Hash::all_zeros()) {
            continue;
          }
        }

        // Parse BRC20 message through inscription operation.
        if self.protocols.contains(&ProtocolKind::BRC20) {
          if let Some(msg) =
            brc20::resolve_message(self.brc20_store, &new_inscriptions, &operation)?
              .map(Message::BRC20)
          {
            messages.push(msg);
            continue;
          }
        }

        // Parse BRC30 message through inscription operation.
        if self.protocols.contains(&ProtocolKind::BRC30) {
          if let Some(msg) = brc30::resolve_message(
            self.client,
            self.ord_store,
            self.brc30_store,
            &new_inscriptions,
            &operation,
            &mut outpoint_to_txout_cache,
          )?
          .map(Message::BRC30)
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
