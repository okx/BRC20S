use std::collections::HashMap;

use super::*;
use crate::{
  index::BlockData,
  okx::datastore::ord::operation::InscriptionOp,
  okx::datastore::{
    brc20, brc20s,
    btc::{self},
    ord,
  },
  okx::protocol::{
    btc::{self as btc_proto},
    ord as ord_proto,
  },
  Instant, Result,
};
use bitcoin::{Network, Txid};
use bitcoincore_rpc::Client;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlockContext {
  pub network: Network,
  pub blockheight: u64,
  pub blocktime: u32,
}

pub struct ProtocolManager<
  'a,
  O: ord::OrdDataStoreReadWrite,
  L: btc::DataStoreReadWrite,
  P: brc20::DataStoreReadWrite,
  M: brc20s::DataStoreReadWrite,
> {
  ord_store: &'a O,
  btc_store: &'a L,
  config: &'a Config,
  call_man: CallManager<'a, O, L, P, M>,
  resolve_man: MsgResolveManager<'a, O, P, M>,
}

impl<
    'a,
    O: ord::OrdDataStoreReadWrite,
    L: btc::DataStoreReadWrite,
    P: brc20::DataStoreReadWrite,
    M: brc20s::DataStoreReadWrite,
  > ProtocolManager<'a, O, L, P, M>
{
  // Need three datastore, and they're all in the same write transaction.
  pub fn new(
    client: &'a Client,
    ord_store: &'a O,
    btc_store: &'a L,
    brc20_store: &'a P,
    brc20s_store: &'a M,
    config: &'a Config,
  ) -> Self {
    Self {
      resolve_man: MsgResolveManager::new(client, ord_store, brc20_store, brc20s_store, config),
      ord_store,
      btc_store,
      config,
      call_man: CallManager::new(ord_store, btc_store, brc20_store, brc20s_store),
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

    // skip the coinbase transaction.
    for (tx, txid) in block.txdata.iter() {
      // index btc balance.
      if self.config.index_btc_balance {
        for msg in
          btc_proto::index_transaction_balance(context, self.ord_store, self.btc_store, tx)?
        {
          // Passive withdrawal executed by BTC transaction.
          if self
            .config
            .first_brc20s_height
            .map(|height| context.blockheight >= height)
            .unwrap_or(false)
          {
            self.call_man.execute_message(context, &Message::BTC(msg))?;
          }
        }
      }

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
      if let Some(tx_operations) = operations.remove(txid) {
        // save all transaction operations to ord database.
        if context.blockheight >= self.config.first_inscription_height {
          ord_proto::save_transaction_operations(self.ord_store, txid, &tx_operations)?;
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
      }
    }

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
