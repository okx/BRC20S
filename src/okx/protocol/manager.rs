use crate::index::{InscriptionEntryValue, InscriptionIdValue};
use crate::okx::datastore::BRC20::redb::BRC20DataStore;
use crate::okx::datastore::BRC20::BRC20DataStoreReadWrite;
use crate::okx::datastore::BRC30::redb::BRC30DataStore;
use crate::okx::datastore::BRC30::BRC30DataStoreReadWrite;
use crate::okx::datastore::ORD::{OrdDataStoreReadOnly, OrdDbReader};
use crate::okx::protocol::protocol::Protocol;
use crate::okx::protocol::BRC20::{BRC20Updater, InscriptionData};
use crate::okx::protocol::BRC30::updater::BRC30Updater;
use anyhow::anyhow;
use bitcoin::{Transaction, Txid};
use redb::Table;
use std::collections::VecDeque;

pub struct ProtocolManager<
  'a,
  'db,
  'tx,
  rw2: BRC20DataStoreReadWrite,
  rw3: BRC30DataStoreReadWrite,
  or: OrdDataStoreReadOnly,
> {
  pub brc20_data_store: &'a rw2,
  pub brc30_data_store: &'a rw3,
  pub ord_reader: &'a or,
  pub id_to_entry: &'a Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
  pub protocols: VecDeque<Protocol>,
}

impl<
    'a,
    'db,
    'tx,
    rw2: BRC20DataStoreReadWrite,
    rw3: BRC30DataStoreReadWrite,
    or: OrdDataStoreReadOnly,
  > ProtocolManager<'a, 'db, 'tx, rw2, rw3, or>
{
  pub fn new(
    brc20_data_store: &'a rw2,
    brc30_data_store: &'a rw3,
    ord_reader: &'a or,
    id_to_entry: &'a Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
  ) -> Self {
    Self {
      brc20_data_store,
      brc30_data_store,
      ord_reader,
      id_to_entry,
      protocols: VecDeque::new(),
    }
  }

  pub fn register(&mut self, p: Protocol) {
    self.protocols.push_back(p);
  }

  pub fn execute_protocols(&mut self, height: u64, block_time: u32) -> Result<(), anyhow::Error> {
    let mut brc20_updater = BRC20Updater::new(self.brc20_data_store, self.id_to_entry);

    let mut brc30_updater = BRC30Updater::new(self.brc30_data_store,self.brc20_data_store, self.id_to_entry);
    loop {
      if let Some(p) = self.protocols.pop_front() {
        match p {
          Protocol::BRC20((tx_id, brc20_transactions)) => {
            brc20_updater
              .index_transaction(height, block_time, tx_id, brc20_transactions)
              .map_err(|e| anyhow!("failed to parse brc20 protocol for {tx_id} reason {e}"))?;
          }
          Protocol::BRC30((tx_id, brc30_transactions)) => {
            brc30_updater
              .index_transaction(height, block_time, tx_id, brc30_transactions)
              .map_err(|e| anyhow!("failed to parse brc20 protocol for {tx_id} reason {e}"))?;
          }
        }
      } else {
        break;
      }
    }
    Ok(())
  }
}
