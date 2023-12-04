pub(crate) mod balance;
pub mod brc0;
pub mod brc20;
pub mod brc20s;
pub mod ord;
mod redb;
mod script_key;

pub use self::{
  redb::{StateReadOnly, StateReadWrite},
  script_key::ScriptKey,
};

/// StateReader is a collection of multiple readonly storages.
///
/// There are multiple categories in the storage, and they can be obtained separately.
pub trait StateReader {
  type OrdReader: ord::DataStoreReadOnly;
  type BRC20Reader: brc20::DataStoreReadOnly;
  type BRC20SReader: brc20s::DataStoreReadOnly;

  // Returns a reference to the readonly Ord store.
  fn ord(&self) -> &Self::OrdReader;

  // Returns a reference to the readonly BRC20 store.
  fn brc20(&self) -> &Self::BRC20Reader;

  // Returns a reference to the readonly BRC20S store.
  fn brc20s(&self) -> &Self::BRC20SReader;
}

/// StateRWriter is a collection of multiple read-write storages.
///
/// There are multiple categories in the storage, and they can be obtained separately.
pub trait StateRWriter {
  type OrdRWriter: ord::DataStoreReadWrite;
  type BRC20RWriter: brc20::DataStoreReadWrite;
  type BRC20SRWriter: brc20s::DataStoreReadWrite;

  // Returns a reference to the read-write ord store.
  fn ord(&self) -> &Self::OrdRWriter;

  // Returns a reference to the read-write BRC20 store.
  fn brc20(&self) -> &Self::BRC20RWriter;

  // Returns a reference to the read-write BRC20S store.
  fn brc20s(&self) -> &Self::BRC20SRWriter;
}
