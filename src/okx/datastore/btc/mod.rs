mod errors;
mod events;

use crate::okx::datastore::ScriptKey;
use std::fmt::{Debug, Display};

pub use self::{errors::BTCError, events::*};

pub trait DataStoreReadOnly {
  type Error: Debug + Display;

  // BTC_BALANCE
  fn get_balance(&self, script_key: &ScriptKey) -> Result<Option<u128>, Self::Error>;
}

pub trait DataStoreReadWrite: DataStoreReadOnly {
  // BTC_BALANCE
  fn set_token_balance(&self, script_key: &ScriptKey, balance: u128) -> Result<(), Self::Error>;
}
