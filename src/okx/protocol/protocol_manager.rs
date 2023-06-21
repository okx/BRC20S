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
  pub fn new(
    client: &'a Client,
    network: Network,
    ord_store: &'a O,
    brc20_store: &'a P,
    brc30_store: &'a M,
  ) -> Self {
    Self {
      resolve_man: MsgResolveManager::new(client, network, ord_store),
      call_man: CallManager::new(ord_store, brc20_store, brc30_store),
    }
  }

  pub fn index_block(
    &self,
    block_height: u64,
    block: &BlockData,
    operation: Vec<InscriptionOp>,
  ) -> Result<()> {
    let mut operations_peeker = operation.into_iter().peekable();
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
      for msg in self.resolve_man.resolve_message(
        txid,
        block_height,
        block.header.time,
        tx,
        tx_operations,
      )? {
        self.call_man.execute_message(&msg)?;
      }
    }
    Ok(())
  }
}
