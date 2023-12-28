use bitcoin::{BlockHash, Transaction};
use {
  super::*,
  crate::{
    index::BlockData,
    okx::{
      datastore::{ord::operation::InscriptionOp, StateRWriter},
      protocol::ord as ord_proto,
    },
    Instant, Result,
  },
  bitcoin::Txid,
  bitcoincore_rpc::Client,
  std::collections::HashMap,
};

pub struct ProtocolManager<'a, RW: StateRWriter> {
  state_store: &'a RW,
  config: &'a ProtocolConfig,
  call_man: CallManager<'a, RW>,
  resolve_man: MsgResolveManager<'a, RW>,
}

#[derive(Default)]
struct TxIndexResult {
  inscriptions_size: usize,
  messages_size: usize,
  brczero_messages: Vec<BrcZeroMsg>,
}

impl TxIndexResult {
  fn update(&mut self, mut other: Self) {
    self.inscriptions_size += other.inscriptions_size;
    self.messages_size += other.messages_size;
    self.brczero_messages.append(&mut other.brczero_messages);
  }
}

impl<'a, RW: StateRWriter> ProtocolManager<'a, RW> {
  // Need three datastore, and they're all in the same write transaction.
  pub fn new(client: &'a Client, state_store: &'a RW, config: &'a ProtocolConfig) -> Self {
    Self {
      state_store,
      config,
      call_man: CallManager::new(state_store),
      resolve_man: MsgResolveManager::new(client, state_store, config),
    }
  }

  fn index_tx(
    &self,
    context: BlockContext,
    txid: &Txid,
    tx: &Transaction,
    tx_operations: &Vec<InscriptionOp>,
    block_hash: &BlockHash,
  ) -> Result<TxIndexResult> {
    let mut result = TxIndexResult::default();
    // save all transaction operations to ord database.
    if self.config.enable_ord_receipts
      && context.blockheight >= self.config.first_inscription_height
    {
      ord_proto::save_transaction_operations(self.state_store.ord(), txid, tx_operations)?;
      result.inscriptions_size = tx_operations.len();
    }

    // Resolve and execute messages.
    let messages = self
      .resolve_man
      .resolve_message(context, tx, tx_operations)?;
    for msg in messages.iter() {
      self.call_man.execute_message(context, msg)?;
    }
    let brczero_messages_in_tx =
      self
        .resolve_man
        .resolve_brczero_inscription(context, tx, tx_operations, &block_hash)?;

    result.brczero_messages = brczero_messages_in_tx;
    result.messages_size = messages.len();

    Ok(result)
  }

  pub(crate) fn index_block(
    &self,
    context: BlockContext,
    block: &BlockData,
    mode: ExecuteMode,
  ) -> Result {
    let start = Instant::now();

    let mut block_result = TxIndexResult::default();
    let block_hash = block.header.block_hash();

    let operations = match mode {
      ExecuteMode::Sync(operations) => {
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
            let result = self.index_tx(context, txid, tx, tx_operations, &block_hash)?;
            block_result.update(result);
          }
        }

        operations
      }
      ExecuteMode::Async(operation_receiver) => {
        let mut block_iter = block.txdata.iter();
        let mut operations = std::collections::HashMap::new();

        // index inscription operations.
        while let Ok((tx_id, tx_operations)) = operation_receiver.recv() {
          let (tx, txid) = block_iter.find(|(_, txid)| &tx_id == txid).unwrap();

          if tx
            .input
            .first()
            .map(|tx_in| tx_in.previous_output.is_null())
            .unwrap_or_default()
          {
            operations.insert(tx_id, tx_operations);
            continue;
          }

          let result = self.index_tx(context, txid, tx, &tx_operations, &block_hash)?;

          block_result.update(result);
          operations.insert(tx_id, tx_operations);
        }

        operations
      }
    };

    if context.blockheight >= self.config.first_brczero_height {
      self
        .call_man
        .send_to_brc0(context, block_result.brczero_messages, &block_hash)?;
    }

    let mut bitmap_count = 0;
    if self.config.enable_index_bitmap {
      bitmap_count = ord_proto::bitmap::index_bitmap(self.state_store.ord(), context, &operations)?;
    }

    log::info!(
      "Protocol Manager indexed block {} with ord inscriptions {}, messages {}, bitmap {} in {} ms",
      context.blockheight,
      block_result.inscriptions_size,
      block_result.messages_size,
      bitmap_count,
      (Instant::now() - start).as_millis(),
    );
    Ok(())
  }
}

pub enum ExecuteMode {
  Sync(HashMap<Txid, Vec<InscriptionOp>>),
  Async(std::sync::mpsc::Receiver<(Txid, Vec<InscriptionOp>)>),
}
