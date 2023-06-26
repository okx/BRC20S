use super::*;
use crate::okx::datastore::BRC30DataStoreReadWrite;
use crate::{
  index::BlockData,
  okx::datastore::{
    ord::{operation::InscriptionOp, OrdDataStoreReadWrite},
    BRC20DataStoreReadWrite,
  },
  Result,
};
use bitcoin::Network;
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
  BRC30,
}

pub struct ProtocolManager<
  'a,
  O: OrdDataStoreReadWrite,
  P: BRC20DataStoreReadWrite,
  M: BRC30DataStoreReadWrite,
> {
  call_man: CallManager<'a, O, P, M>,
  resolve_man: MsgResolveManager<'a, O>,
}

impl<'a, O: OrdDataStoreReadWrite, P: BRC20DataStoreReadWrite, M: BRC30DataStoreReadWrite>
  ProtocolManager<'a, O, P, M>
{
  // Need three datastore, and they're all in the same write transaction.
  pub fn new(client: &'a Client, ord_store: &'a O, brc20_store: &'a P, brc30_store: &'a M) -> Self {
    Self {
      resolve_man: MsgResolveManager::new(client, ord_store),
      call_man: CallManager::new(ord_store, brc20_store, brc30_store),
    }
  }

  pub fn index_block(
    &self,
    context: BlockContext,
    block: &BlockData,
    operation: Vec<InscriptionOp>,
  ) -> Result {
    let mut operations_peeker = operation.into_iter().peekable();
    // skip the coinbase transaction.
    for (tx, txid) in block.txdata.iter().skip(1) {
      let mut tx_operations: Vec<InscriptionOp> = Vec::new();

      // Collect the inscription operations of this transaction.
      while let Some(op) = operations_peeker.peek() {
        if op.txid != *txid {
          break;
        }
        tx_operations.push(operations_peeker.next().unwrap());
      }

      // Resolve and execute messages.
      for msg in self.resolve_man.resolve_message(tx, tx_operations)? {
        self.call_man.execute_message(context, &msg)?;
      }
    }
    Ok(())
  }
}
