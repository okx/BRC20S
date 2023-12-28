use bitcoin::{OutPoint, TxOut};
use std::collections::HashMap;
use {
  super::{
    brc20::redb::{DataStore as BRC20StateRW, DataStoreReader as BRC20StateReader},
    brc20s::redb::{DataStore as BRC20SStateRW, DataStoreReader as BRC20SStateReader},
    ord::redb::{OrdDbReadWriter as OrdStateRW, OrdDbReader as OrdStateReader},
    StateRWriter, StateReader,
  },
  redb::{ReadTransaction, WriteTransaction},
};

/// StateReadOnly, based on `redb`, is an implementation of the StateRWriter trait.
pub struct StateReadOnly<'db, 'a> {
  ord: OrdStateReader<'db, 'a>,
  brc20: BRC20StateReader<'db, 'a>,
  brc20s: BRC20SStateReader<'db, 'a>,
}

impl<'db, 'a> StateReadOnly<'db, 'a> {
  #[allow(dead_code)]
  pub fn new(rtx: &'a ReadTransaction<'db>) -> Self {
    Self {
      ord: OrdStateReader::new(rtx),
      brc20: BRC20StateReader::new(rtx),
      brc20s: BRC20SStateReader::new(rtx),
    }
  }
}

impl<'db, 'a> StateReader for StateReadOnly<'db, 'a> {
  type OrdReader = OrdStateReader<'db, 'a>;
  type BRC20Reader = BRC20StateReader<'db, 'a>;
  type BRC20SReader = BRC20SStateReader<'db, 'a>;

  fn ord(&self) -> &Self::OrdReader {
    &self.ord
  }

  fn brc20(&self) -> &Self::BRC20Reader {
    &self.brc20
  }

  fn brc20s(&self) -> &Self::BRC20SReader {
    &self.brc20s
  }
}

/// StateReadWrite, based on `redb`, is an implementation of the StateRWriter trait.
pub struct StateReadWrite<'db, 'a> {
  ord: OrdStateRW<'db, 'a>,
  brc20: BRC20StateRW<'db, 'a>,
  brc20s: BRC20SStateRW<'db, 'a>,
}

impl<'db, 'a> StateReadWrite<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>, tx_out_cache: &'a HashMap<OutPoint, TxOut>) -> Self {
    Self {
      ord: OrdStateRW::new(wtx, tx_out_cache),
      brc20: BRC20StateRW::new(wtx),
      brc20s: BRC20SStateRW::new(wtx),
    }
  }
}

impl<'db, 'a> StateRWriter for StateReadWrite<'db, 'a> {
  type OrdRWriter = OrdStateRW<'db, 'a>;
  type BRC20RWriter = BRC20StateRW<'db, 'a>;
  type BRC20SRWriter = BRC20SStateRW<'db, 'a>;

  fn ord(&self) -> &Self::OrdRWriter {
    &self.ord
  }

  fn brc20(&self) -> &Self::BRC20RWriter {
    &self.brc20
  }

  fn brc20s(&self) -> &Self::BRC20SRWriter {
    &self.brc20s
  }
}
