use super::*;
use crate::{
  index::OUTPOINT_TO_ENTRY,
  okx::datastore::ord::{InscriptionOp, OrdDataStoreReadOnly, OrdDataStoreReadWrite},
  InscriptionId, Result,
};
use bitcoin::{consensus::Encodable, OutPoint, TxOut, Txid};
use redb::WriteTransaction;
use crate::okx::protocol::brc0::RpcParams;

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

  fn get_transaction_operations(
    &self,
    txid: &bitcoin::Txid,
  ) -> Result<Vec<InscriptionOp>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_transaction_operations(txid)
  }

  fn get_brczero_rpcparams(
    &self,
    height: u64,
  ) -> Result<RpcParams, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_brczero_rpcparams(height)
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

  fn save_transaction_operations(
    &self,
    txid: &Txid,
    operations: &[InscriptionOp],
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(ORD_TX_TO_OPERATIONS)?.insert(
      txid.to_string().as_str(),
      bincode::serialize(operations).unwrap().as_slice(),
    )?;
    Ok(())
  }

  fn save_brczero_to_rpcparams(
    &self,
    height: u64,
    params: &RpcParams,
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(ORD_BRCZERO_TO_RPCPARAMS)?.insert(
      height,
      bincode::serialize(params).unwrap().as_slice(),
    )?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{okx::datastore::ord::Action, unbound_outpoint, SatPoint};
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;

  #[test]
  fn test_outpoint_to_script() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let ord_db = OrdDbReadWriter::new(&wtx);

    let outpoint1 = unbound_outpoint();
    let tx_out = TxOut {
      value: 100,
      script_pubkey: bitcoin::Address::from_str("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")
        .unwrap()
        .assume_checked()
        .script_pubkey(),
    };

    ord_db.set_outpoint_to_txout(outpoint1, &tx_out).unwrap();

    assert_eq!(
      ord_db.get_outpoint_to_txout(outpoint1).unwrap().unwrap(),
      tx_out
    );
  }

  #[test]
  fn test_transaction_to_operations() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let ord_db = OrdDbReadWriter::new(&wtx);
    let txid =
      Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();
    let operation = InscriptionOp {
      txid,
      action: Action::New {
        cursed: false,
        unbound: false,
      },
      inscription_number: Some(100),
      inscription_id: txid.into(),
      old_satpoint: SatPoint::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111:1:1",
      )
      .unwrap(),
      new_satpoint: Some(SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 1,
      }),
    };

    ord_db
      .save_transaction_operations(&txid, &[operation.clone()])
      .unwrap();

    assert_eq!(
      ord_db.get_transaction_operations(&txid).unwrap(),
      vec![operation]
    );
  }
}
