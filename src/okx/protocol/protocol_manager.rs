use super::*;
use crate::{
  index::BlockData,
  okx::datastore::{
    BRC20DataStoreReadWrite,
    ORD::{operation::InscriptionOperation, OrdDataStoreReadWrite},
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

pub struct ProtocolManager<'a, O: OrdDataStoreReadWrite, P: BRC20DataStoreReadWrite> {
  call_man: CallManager<'a, O, P>,
  resolve_man: MsgResolveManager<'a, O, P>,
}

impl<'a, O: OrdDataStoreReadWrite, P: BRC20DataStoreReadWrite> ProtocolManager<'a, O, P> {
  // Need three datastore, and they're all in the same write transaction.
  pub fn new(client: &'a Client, network: Network, ord_store: &'a O, brc20_store: &'a P) -> Self {
    Self {
      resolve_man: MsgResolveManager::new(client, network, ord_store, brc20_store),
      call_man: CallManager::new(ord_store, brc20_store),
    }
  }

  pub fn index_block(
    &self,
    block_height: u64,
    block: &BlockData,
    operation: Vec<InscriptionOperation>,
  ) -> Result<()> {
    let mut operations_peeker = operation.into_iter().peekable();
    for (tx, txid) in block.txdata.iter().skip(1) {
      let mut tx_operations: Vec<InscriptionOperation> = Vec::new();

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
