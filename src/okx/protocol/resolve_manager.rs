use {
  super::*,
  crate::{
    okx::{
      datastore::{
        ord::{operation::InscriptionOp, DataStoreReadWrite},
        StateRWriter,
      },
      protocol::Message,
    },
    Inscription, Result,
  },
  anyhow::anyhow,
  bitcoin::{OutPoint, Transaction, TxOut},
  bitcoincore_rpc::Client,
  std::collections::HashMap,
};

pub struct MsgResolveManager<'a, RW: StateRWriter> {
  client: &'a Client,
  state_store: &'a RW,
  config: &'a ProtocolConfig,
}

impl<'a, RW: StateRWriter> MsgResolveManager<'a, RW> {
  pub fn new(client: &'a Client, state_store: &'a RW, config: &'a ProtocolConfig) -> Self {
    Self {
      client,
      state_store,
      config,
    }
  }

  pub fn resolve_message(
    &self,
    context: BlockContext,
    tx: &Transaction,
    operations: &[InscriptionOp],
  ) -> Result<Vec<Message>> {
    log::debug!(
      "Resolve Manager indexed transaction {}, operations size: {}, data: {:?}",
      tx.txid(),
      operations.len(),
      operations
    );
    let mut messages = Vec::new();
    let mut operation_iter = operations.iter().peekable();
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
            brc20::Message::resolve(self.state_store.brc20(), &new_inscriptions, operation)?
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

        // Parse BRC20S message through inscription operation.
        if self
          .config
          .first_brc20s_height
          .map(|height| context.blockheight >= height)
          .unwrap_or(false)
        {
          if let Some(msg) = brc20s::Message::resolve(
            self.client,
            self.state_store.ord(),
            self.state_store.brc20s(),
            &new_inscriptions,
            operation,
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
    //self.update_outpoint_to_txout(outpoint_to_txout_cache)?;
    Ok(messages)
  }

  #[allow(dead_code)]
  fn update_outpoint_to_txout(&self, outpoint_to_txout_cache: HashMap<OutPoint, TxOut>) -> Result {
    for (outpoint, txout) in outpoint_to_txout_cache {
      self
        .state_store
        .ord()
        .set_outpoint_to_txout(outpoint, &txout)
        .or(Err(anyhow!(
          "failed to get tx out! error: {} not found",
          outpoint
        )))?;
    }
    Ok(())
  }
}
