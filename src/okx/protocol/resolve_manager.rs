use super::*;
use std::collections::HashSet;

use crate::{
  okx::{
    datastore::{
      BRC20::BRC20DataStoreReadOnly,
      ORD::{operation::InscriptionOp, OrdDataStoreReadOnly},
    },
    protocol::Message,
  },
  Result,
};
use bitcoin::{Network, Transaction, Txid};
use bitcoincore_rpc::Client;
pub struct MsgResolveManager<'a, O: OrdDataStoreReadOnly, P: BRC20DataStoreReadOnly> {
  protocols: HashSet<ProtocolKind>,
  client: &'a Client,
  network: Network,
  ord_store: &'a O,
  brc20_store: &'a P,
}

impl<'a, O: OrdDataStoreReadOnly, P: BRC20DataStoreReadOnly> MsgResolveManager<'a, O, P> {
  pub fn new(client: &'a Client, network: Network, ord_store: &'a O, brc20_store: &'a P) -> Self {
    Self {
      protocols: HashSet::new(),
      client,
      network,
      ord_store,
      brc20_store,
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

        // TODO: Resolve BRC30 message.
        // if self.protocols.contains(&ProtocolKind::BRC30) {
        //   if let Some(msg) = BRC20::resolve_message(
        //     self.client,
        //     self.ord_store,
        //     txid,
        //     block_height,
        //     block_time,
        //     tx,
        //     &op,
        //   )?
        //   .map(|v| Message::BRC30(v))
        //   {
        //     messages.push(msg);
        //   }
        // }
      }
    }
    Ok(messages)
  }
}
