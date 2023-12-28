use {
  super::{
    brc20::redb::{DataStore as BRC20StateRW, DataStoreReader as BRC20StateReader},
    ord::redb::{OrdDbReadWriter as OrdStateRW, OrdDbReader as OrdStateReader},
    StateRWriter, StateReader,
  },
  redb::{ReadTransaction, WriteTransaction},
};

/// StateReadOnly, based on `redb`, is an implementation of the StateRWriter trait.
pub struct StateReadOnly<'db, 'a> {
  ord: OrdStateReader<'db, 'a>,
  brc20: BRC20StateReader<'db, 'a>,
}

impl<'db, 'a> StateReadOnly<'db, 'a> {
  #[allow(dead_code)]
  pub fn new(rtx: &'a ReadTransaction<'db>) -> Self {
    Self {
      ord: OrdStateReader::new(rtx),
      brc20: BRC20StateReader::new(rtx),
    }
  }
}

impl<'db, 'a> StateReader for StateReadOnly<'db, 'a> {
  type OrdReader = OrdStateReader<'db, 'a>;
  type BRC20Reader = BRC20StateReader<'db, 'a>;

  fn ord(&self) -> &Self::OrdReader {
    &self.ord
  }

  fn brc20(&self) -> &Self::BRC20Reader {
    &self.brc20
  }
}

/// StateReadWrite, based on `redb`, is an implementation of the StateRWriter trait.
pub struct StateReadWrite<'db, 'a> {
  ord: OrdStateRW<'db, 'a>,
  brc20: BRC20StateRW<'db, 'a>,
}

impl<'db, 'a> StateReadWrite<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self {
      ord: OrdStateRW::new(wtx),
      brc20: BRC20StateRW::new(wtx),
    }
  }
}

impl<'db, 'a> StateRWriter for StateReadWrite<'db, 'a> {
  type OrdRWriter = OrdStateRW<'db, 'a>;
  type BRC20RWriter = BRC20StateRW<'db, 'a>;

  fn ord(&self) -> &Self::OrdRWriter {
    &self.ord
  }

  fn brc20(&self) -> &Self::BRC20RWriter {
    &self.brc20
  }
}
