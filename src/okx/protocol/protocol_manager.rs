use {
  super::*,
  crate::{
    index::BlockData,
    okx::{
      datastore::{ord::operation::InscriptionOp, StateRWriter},
      protocol::ord as ord_proto,
    },
    Instant, Result,
    rpc::BRCZeroRpcClient,
  },
  bitcoin::Txid,
  bitcoincore_rpc::Client,
  std::collections::HashMap,
};

pub struct ProtocolManager<'a, RW: StateRWriter> {
  state_store: &'a RW,
  brc0_client: &'a BRCZeroRpcClient,
  config: &'a ProtocolConfig,
  call_man: CallManager<'a, RW>,
  resolve_man: MsgResolveManager<'a, RW>,
}

impl<'a, RW: StateRWriter> ProtocolManager<'a, RW> {
  // Need three datastore, and they're all in the same write transaction.
  pub fn new(client: &'a Client, brc0_client: &'a BRCZeroRpcClient,state_store: &'a RW, config: &'a ProtocolConfig) -> Self {
    Self {
      state_store,
      brc0_client,
      config,
      call_man: CallManager::new(state_store),
      resolve_man: MsgResolveManager::new(client, state_store, config),
    }
  }

  pub(crate) fn index_block(
    &self,
    context: BlockContext,
    block: &BlockData,
    operations: HashMap<Txid, Vec<InscriptionOp>>,
  ) -> Result {
    let start = Instant::now();
    let mut inscriptions_size = 0;
    let mut messages_size = 0;
    let mut brczero_messages_in_block: Vec<BrcZeroMsg> = Vec::new();
    // skip the coinbase transaction.
    for (tx, txid) in block.txdata.iter() {
      // skip coinbase transaction.
      if tx
        .input
        .first()
        .map(|tx_in| tx_in.previous_output.is_null())
        .unwrap_or_default()
      {
        continue;
      }
      // index inscription operations.
      if let Some(tx_operations) = operations.get(txid) {
        // save all transaction operations to ord database.
        if self.config.enable_ord_receipts
          && context.blockheight >= self.config.first_inscription_height
        {
          ord_proto::save_transaction_operations(self.state_store.ord(), txid, tx_operations)?;
          inscriptions_size += tx_operations.len();
        }

        // Resolve and execute messages.
        let messages = self
          .resolve_man
          .resolve_message(context, tx, tx_operations)?;
        for msg in messages.iter() {
          self.call_man.execute_message(context, msg)?;
        }
        messages_size += messages.len();
        let mut brczero_messages_in_tx = self
            .resolve_man
            .resolve_brczero_inscription(context, tx, tx_operations.clone(),&block.header.block_hash())?;
        brczero_messages_in_block.append(&mut brczero_messages_in_tx);
      }
    }
    if context.blockheight >= self.config.first_brczero_height {
      self.call_man.send_to_brc0(self.brc0_client, context, brczero_messages_in_block,&block.header.block_hash())?;
    }

    let mut bitmap_count = 0;
    if self.config.enable_index_bitmap {
      bitmap_count = ord_proto::bitmap::index_bitmap(self.state_store.ord(), context, &operations)?;
    }

    log::info!(
      "Protocol Manager indexed block {} with ord inscriptions {}, messages {}, bitmap {} in {} ms",
      context.blockheight,
      inscriptions_size,
      messages_size,
      bitmap_count,
      (Instant::now() - start).as_millis(),
    );
    Ok(())
  }
}
