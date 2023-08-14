mod balance;
mod errors;
mod events;
pub mod redb;

pub use self::{balance::Balance, errors::BTCError, events::*};
use super::ScriptKey;
use std::fmt::{Debug, Display};

pub trait DataStoreReadOnly {
  type Error: Debug + Display;

  fn get_balance(&self, script_key: &ScriptKey) -> Result<Option<Balance>, Self::Error>;
}

pub trait DataStoreReadWrite: DataStoreReadOnly {
  fn update_balance(&self, script_key: &ScriptKey, new_balance: Balance)
    -> Result<(), Self::Error>;
}
