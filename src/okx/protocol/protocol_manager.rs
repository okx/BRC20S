use std::collections::HashMap;

use super::*;
use crate::okx::datastore::brc20;
use crate::okx::datastore::brc20s;
use crate::okx::datastore::ord;
use crate::{index::BlockData, okx::datastore::ord::operation::InscriptionOp, Instant, Result};
use bitcoin::{Network, Txid};
use bitcoincore_rpc::Client;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlockContext {
  pub network: Network,
  pub blockheight: u64,
  pub blocktime: u32,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum ProtocolKind {
  BRC20,
  BRC20S,
}

pub struct ProtocolManager<
  'a,
  O: ord::OrdDataStoreReadWrite,
  P: brc20::BRC20DataStoreReadWrite,
  M: brc20s::DataStoreReadWrite,
> {
  call_man: CallManager<'a, O, P, M>,
  resolve_man: MsgResolveManager<'a, O, P, M>,
}

impl<
    'a,
    O: ord::OrdDataStoreReadWrite,
    P: brc20::BRC20DataStoreReadWrite,
    M: brc20s::DataStoreReadWrite,
  > ProtocolManager<'a, O, P, M>
{
  // Need three datastore, and they're all in the same write transaction.
  pub fn new(
    client: &'a Client,
    ord_store: &'a O,
    brc20_store: &'a P,
    brc20s_store: &'a M,
    first_brc20_height: u64,
    first_brc20s_height: u64,
  ) -> Self {
    Self {
      resolve_man: MsgResolveManager::new(
        client,
        ord_store,
        brc20_store,
        brc20s_store,
        first_brc20_height,
        first_brc20s_height,
      ),
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
    let mut messages_size = 0;
    // skip the coinbase transaction.
    for (tx, txid) in block.txdata.iter().skip(1) {
      if let Some(tx_operations) = operations.remove(txid) {
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
      "Protocol Manager indexed block {} with {} messages in {} ms",
      context.blockheight,
      messages_size,
      (Instant::now() - start).as_millis(),
    );
    Ok(())
  }
}
