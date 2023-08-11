pub mod redb;
mod balance;

use std::fmt::{Debug, Display};
pub use self::{
  balance::Balance
};
use super::ScriptKey;

pub trait DataStoreReadOnly {
  type Error: Debug + Display;

  fn get_balance(
    &self,
    script_key: &ScriptKey,
  ) -> Result<Option<Balance>, Self::Error>;
}


pub trait DataStoreReadWrite: DataStoreReadOnly {
  fn update_balance(
    &self,
    script_key: &ScriptKey,
    new_balance: Balance,
  ) -> Result<(), Self::Error>;
}
