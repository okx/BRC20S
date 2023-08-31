mod balance;
mod events;
pub mod redb;

pub use self::{balance::Balance, events::*, redb::*};
use {
  super::ScriptKey,
  std::fmt::{Debug, Display},
};

pub trait DataStoreReadOnly {
  type Error: Debug + Display;

  fn get_balance(&self, script_key: &ScriptKey) -> Result<Option<Balance>, Self::Error>;
}

pub trait DataStoreReadWrite: DataStoreReadOnly {
  fn update_balance(&self, script_key: &ScriptKey, new_balance: Balance)
    -> Result<(), Self::Error>;
}
