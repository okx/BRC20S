use super::*;
use std::collections::HashSet;

use crate::{
  okx::{
    datastore::ORD::{operation::InscriptionOp, OrdDataStoreReadOnly},
    protocol::Message,
  },
  Result,
};
use bitcoin::{Network, Transaction, Txid};
use bitcoincore_rpc::Client;
pub struct MsgResolveManager<'a, O: OrdDataStoreReadOnly> {
  protocols: HashSet<ProtocolKind>,
  client: &'a Client,
  network: Network,
  ord_store: &'a O,
}

impl<'a, O: OrdDataStoreReadOnly> MsgResolveManager<'a, O> {
  pub fn new(client: &'a Client, network: Network, ord_store: &'a O) -> Self {
    Self {
      protocols: HashSet::new(),
      client,
      network,
      ord_store,
    }
  }

  pub fn resolve_message(
    &self,
    txid: &Txid,
    block_height: u64,
    block_time: u32,
    tx: &Transaction,
    operation: Vec<InscriptionOp>,
  ) -> Result<Vec<Message>> {
    let mut messages = Vec::new();
    let mut operation_peeker = operation.into_iter().peekable();
    for input in &tx.input {
      // TODO: BTC transfer to BRC30 passive withdrawal.
      // if self.protocols.contains(&ProtocolKind::BRC30) {
      //   messages.push(BTC::resolve_message(txid, block_height, block_time, tx));
      // }
      while let Some(op) = operation_peeker.peek() {
        if op.old_satpoint.outpoint != input.previous_output {
          break;
        }
        let op = operation_peeker.next().unwrap();

        // Resolve BRC20 message.
        if self.protocols.contains(&ProtocolKind::BRC20) {
          if let Some(msg) = BRC20::resolve_message(
            self.client,
            self.network,
            self.ord_store,
            txid,
            block_height,
            block_time,
            tx,
            &op,
          )?
          .map(|v| Message::BRC20(v))
          {
            messages.push(msg);
          }
        }

        // Resolve BRC30 message.
        if self.protocols.contains(&ProtocolKind::BRC30) {
          if let Some(msg) = BRC30::resolve_message(
            self.client,
            self.network,
            self.ord_store,
            txid,
            block_height,
            block_time,
            tx,
            &op,
          )?
          .map(|v| Message::BRC30(v))
          {
            messages.push(msg);
          }
        }
      }
    }
    Ok(messages)
  }
}
