use super::*;
use crate::{
  index::OUTPOINT_TO_ENTRY,
  okx::datastore::ord::{OrdDataStoreReadOnly, OrdDataStoreReadWrite},
  InscriptionId, Result,
};
use bitcoin::{consensus::Encodable, OutPoint, TxOut};
use redb::WriteTransaction;

pub struct OrdDbReadWriter<'db, 'a> {
  wtx: &'a WriteTransaction<'db>,
}

impl<'db, 'a> OrdDbReadWriter<'db, 'a>
where
  'db: 'a,
{
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self { wtx }
  }
}

impl<'db, 'a> OrdDataStoreReadOnly for OrdDbReadWriter<'db, 'a> {
  type Error = redb::Error;
  fn get_number_by_inscription_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<i64>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_number_by_inscription_id(inscription_id)
  }

  fn get_outpoint_to_txout(&self, outpoint: OutPoint) -> Result<Option<TxOut>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_outpoint_to_txout(outpoint)
  }
}

impl<'db, 'a> OrdDataStoreReadWrite for OrdDbReadWriter<'db, 'a> {
  // OUTPOINT_TO_SCRIPT

  fn set_outpoint_to_txout(&self, outpoint: OutPoint, tx_out: &TxOut) -> Result<(), Self::Error> {
    let mut value = [0; 36];
    outpoint
      .consensus_encode(&mut value.as_mut_slice())
      .unwrap();

    let mut entry = Vec::new();
    tx_out.consensus_encode(&mut entry)?;
    self
      .wtx
      .open_table(OUTPOINT_TO_ENTRY)?
      .insert(&value, entry.as_slice())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::unbound_outpoint;
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;

  #[test]
  fn test_outpoint_to_script() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let brc30db = OrdDbReadWriter::new(&wtx);

    let outpoint1 = unbound_outpoint();
    let tx_out = TxOut {
      value: 100,
      script_pubkey: bitcoin::Address::from_str("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")
        .unwrap()
        .script_pubkey(),
    };

    brc30db.set_outpoint_to_txout(outpoint1, &tx_out).unwrap();

    assert_eq!(
      brc30db.get_outpoint_to_txout(outpoint1).unwrap().unwrap(),
      tx_out
    );
  }
}
