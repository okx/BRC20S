use crate::index::{InscriptionEntryValue, InscriptionIdValue};
use crate::okx::datastore::BRC20::BRC20DataStoreReadWrite;
use crate::okx::protocol::protocol::Protocol;
use crate::okx::protocol::BRC20::{BRC20DataStore, BRC20Updater, InscriptionData};
use anyhow::anyhow;
use bitcoin::{Transaction, Txid};
use redb::Table;
use std::collections::VecDeque;

pub struct ProtocolManager<'a, 'db, 'tx, L: BRC20DataStoreReadWrite> {
  pub BRC20_data_store: &'a L,
  pub id_to_entry: &'a Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
  pub protocols: VecDeque<(Txid, Vec<InscriptionData>)>,
}

impl<'a, 'db, 'tx, L: BRC20DataStoreReadWrite> ProtocolManager<'a, 'db, 'tx, L> {
  pub fn new(
    BRC20_data_store: &'a L,
    id_to_entry: &'a Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
  ) -> Self {
    Self {
      BRC20_data_store,
      id_to_entry,
      protocols: VecDeque::new(),
    }
  }

  pub fn register(&mut self, tx_id: Txid, protocols: Vec<InscriptionData>) {
    self.protocols.push_back((tx_id, protocols));
  }

  pub fn execute_protocols(&mut self, height: u64, block_time: u32) -> Result<(), anyhow::Error> {
    let mut brc20_updater = BRC20Updater::new(self.BRC20_data_store, self.id_to_entry);
    loop {
      if let Some((tx_id, brc20_transaction)) = self.protocols.pop_front() {
        brc20_updater
          .index_transaction(height, block_time, tx_id, brc20_transaction)
          .map_err(|e| anyhow!("failed to parse brc20 protocol for {tx_id} reason {e}"))?;
      } else {
        break;
      }
    }
    Ok(())
  }
}
