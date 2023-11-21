use std::collections::HashMap;

use super::*;
use crate::{
  index::BlockData,
  okx::datastore::ord::operation::InscriptionOp,
  okx::datastore::{brc20, brc20s, ord},
  okx::protocol::Message,
  rpc::BRCZeroRpcClient,
  Instant, Result,
};
use anyhow::anyhow;
use bitcoin::{Network, Txid};
use bitcoincore_rpc::Client;
// use web3::error::TransportError::Message;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlockContext {
  pub network: Network,
  pub blockheight: u64,
  pub blocktime: u32,
  pub blockhash: bitcoin::BlockHash,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum ProtocolKind {
  BRC20,
  BRC20S,
  BRC0,
}

pub struct ProtocolManager<
  'a,
  O: ord::OrdDataStoreReadWrite,
  P: brc20::DataStoreReadWrite,
  M: brc20s::DataStoreReadWrite,
> {
  brc0_client: &'a BRCZeroRpcClient,
  ord_store: &'a O,
  first_inscription_height: u64,
  first_brczero_height: u64,
  call_man: CallManager<'a, O, P, M>,
  resolve_man: MsgResolveManager<'a, O, P, M>,
}

impl<
    'a,
    O: ord::OrdDataStoreReadWrite,
    P: brc20::DataStoreReadWrite,
    M: brc20s::DataStoreReadWrite,
  > ProtocolManager<'a, O, P, M>
{
  // Need three datastore, and they're all in the same write transaction.
  pub fn new(
    client: &'a Client,
    brc0_client: &'a BRCZeroRpcClient,
    ord_store: &'a O,
    brc20_store: &'a P,
    brc20s_store: &'a M,
    first_inscription_height: u64,
    first_brc20_height: u64,
    first_brc20s_height: u64,
    first_brczero_height: u64,
  ) -> Self {
    Self {
      resolve_man: MsgResolveManager::new(
        client,
        ord_store,
        brc20_store,
        brc20s_store,
        first_brc20_height,
        first_brc20s_height,
        first_brczero_height,
      ),
      brc0_client,
      ord_store,
      first_inscription_height,
      first_brczero_height,
      call_man: CallManager::new(ord_store, brc20_store, brc20s_store),
    }
  }

  pub(crate) fn index_block(
    &self,
    context: BlockContext,
    block: &BlockData,
    mut operations: HashMap<Txid, Vec<InscriptionOp>>,
  ) -> Result {
    let start = Instant::now();
    let mut inscriptions_size = 0;
    let mut messages_size = 0;
    let mut messages_in_block: Vec<Message> = Vec::new();
    let mut brczero_messages_in_block: Vec<BrcZeroMsg> = Vec::new();
    // skip the coinbase transaction.
    for (tx, txid) in block.txdata.iter().skip(1) {
      if let Some(tx_operations) = operations.remove(txid) {
        // save transaction operations.
        if context.blockheight >= self.first_inscription_height {
          self
            .ord_store
            .save_transaction_operations(txid, &tx_operations)
            .map_err(|e| {
              anyhow!("failed to set transaction ordinals operations to state! error: {e}")
            })?;
          inscriptions_size += tx_operations.len();
        }

        // Resolve messages for some protocol and execute it.
        // let mut messages_in_tx = self
        //   .resolve_man
        //   .resolve_message(context, tx, tx_operations.clone())?;
        //
        // messages_size += messages_in_tx.len();
        // messages_in_block.append(&mut messages_in_tx);

        // Resolve inscription for brc-zero and execute it.
        let mut brczero_messages_in_tx = self
            .resolve_man
            .resolve_brczero_inscription(context, tx, tx_operations.clone())?;
        brczero_messages_in_block.append(&mut brczero_messages_in_tx);
      }
    }

    // send inscription to brczero
    if context.blockheight >= self.first_brczero_height {
      self.call_man.send_to_brc0(self.brc0_client, context, brczero_messages_in_block)?;
    }

    // Execute messages.
    // if context.blockheight >= self.first_brczero_height {
    //   self
    //     .call_man
    //     .execute_block_message(self.brc0_client, context, messages_in_block)?;
    // }

    log::info!(
      "Protocol Manager indexed block {} with {} messages, ord inscriptions {} in {} ms",
      context.blockheight,
      messages_size,
      inscriptions_size,
      (Instant::now() - start).as_millis(),
    );
    Ok(())
  }
}
