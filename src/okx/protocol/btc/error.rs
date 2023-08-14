use crate::okx::datastore::btc::{BTCError, DataStoreReadOnly};

#[derive(Debug, thiserror::Error)]
pub enum Error<L: DataStoreReadOnly> {
  #[error("btc error: {0}")]
  BTCError(BTCError),

  #[error("ledger error: {0}")]
  LedgerError(<L>::Error),
}
