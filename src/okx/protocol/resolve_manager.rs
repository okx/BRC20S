use super::*;
use std::collections::HashSet;

use crate::{
  okx::{
    datastore::ord::{operation::InscriptionOp, OrdDataStoreReadOnly},
    protocol::Message,
  },
  Inscription, Result,
};
use bitcoin::{Network, Transaction};
use bitcoincore_rpc::Client;
pub struct MsgResolveManager<'a, O: OrdDataStoreReadOnly> {
  protocols: HashSet<ProtocolKind>,
  client: &'a Client,
  network: Network,
  ord_store: &'a O,
}

impl<'a, O: OrdDataStoreReadOnly> MsgResolveManager<'a, O> {
  pub fn new(client: &'a Client, network: Network, ord_store: &'a O) -> Self {
    let mut protocols: HashSet<ProtocolKind> = HashSet::new();
    protocols.insert(ProtocolKind::BRC20);
    protocols.insert(ProtocolKind::BRC30);
    Self {
      protocols,
      client,
      network,
      ord_store,
    }
  }

  pub fn resolve_message(
    &self,
    block_height: u64,
    block_time: u32,
    tx: &Transaction,
    operations: Vec<InscriptionOp>,
  ) -> Result<Vec<Message>> {
    let mut messages = Vec::new();
    let mut operation_iter = operations.into_iter().peekable();
    let new_inscriptions = Inscription::from_transaction(tx)
      .into_iter()
      .map(|v| v.inscription)
      .collect();
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
        if self.protocols.contains(&ProtocolKind::BRC20) {
          if let Some(msg) = brc20::resolve_message(
            self.client,
            self.network,
            self.ord_store,
            block_height,
            block_time,
            &new_inscriptions,
            &operation,
          )?
          .map(Message::BRC20)
          {
            messages.push(msg);
          }
        }

        // Parse BRC30 message through inscription operation.
        if self.protocols.contains(&ProtocolKind::BRC30) {
          if let Some(msg) = brc30::resolve_message(
            self.client,
            self.network,
            self.ord_store,
            block_height,
            block_time,
            &new_inscriptions,
            &operation,
          )?
          .map(Message::BRC30)
          {
            messages.push(msg);
          }
        }
      }
    }
    Ok(messages)
  }
}
