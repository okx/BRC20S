use crate::index::{InscriptionEntryValue, InscriptionIdValue, OutPointValue};
use crate::inscription_id::InscriptionId;
use crate::okx::datastore::ord::collections::CollectionKind;
use crate::okx::datastore::ord::InscriptionOp;
use crate::Hash;
use bitcoin::consensus::{Decodable, Encodable};
use bitcoin::{OutPoint, TxOut, Txid};
use redb::{ReadableTable, Table};
use std::io;

// COLLECTIONS_INSCRIPTION_ID_TO_KINDS
pub fn get_collections_of_inscription<T>(
  table: &T,
  inscription_id: &InscriptionId,
) -> crate::Result<Option<Vec<CollectionKind>>>
where
  T: ReadableTable<&'static [u8; 36], &'static [u8]>,
{
  let mut key = [0; 36];
  let (txid, index) = key.split_at_mut(32);
  txid.copy_from_slice(inscription_id.txid.as_ref());
  index.copy_from_slice(&inscription_id.index.to_be_bytes());

  Ok(
    table
      .get(&key)?
      .map(|v| bincode::deserialize::<Vec<CollectionKind>>(v.value()).unwrap()),
  )
}

// COLLECTIONS_KEY_TO_INSCRIPTION_ID
pub fn get_collection_inscription_id<T>(
  table: &T,
  key: &str,
) -> crate::Result<Option<InscriptionId>>
where
  T: ReadableTable<&'static str, &'static [u8; 36]>,
{
  Ok(table.get(key)?.map(|v| {
    let (txid, index) = v.value().split_at(32);
    InscriptionId {
      txid: Txid::from_raw_hash(Hash::from_slice(txid).unwrap()),
      index: u32::from_be_bytes(index.try_into().unwrap()),
    }
  }))
}

// INSCRIPTION_ID_TO_INSCRIPTION_ENTRY
pub fn get_number_by_inscription_id<T>(
  table: &T,
  inscription_id: &InscriptionId,
) -> crate::Result<Option<i64>>
where
  T: ReadableTable<&'static InscriptionIdValue, InscriptionEntryValue>,
{
  let mut key = [0; 36];
  let (txid, index) = key.split_at_mut(32);
  txid.copy_from_slice(inscription_id.txid.as_ref());
  index.copy_from_slice(&inscription_id.index.to_be_bytes());
  Ok(table.get(&key)?.map(|value| value.value().2))
}

// OUTPOINT_TO_ENTRY
pub fn get_txout_by_outpoint<T>(table: &T, outpoint: &OutPoint) -> crate::Result<Option<TxOut>>
where
  T: ReadableTable<&'static OutPointValue, &'static [u8]>,
{
  let mut value = [0; 36];
  outpoint
    .consensus_encode(&mut value.as_mut_slice())
    .unwrap();
  Ok(
    table
      .get(&value)?
      .map(|x| Decodable::consensus_decode(&mut io::Cursor::new(x.value())).unwrap()),
  )
}

// ORD_TX_TO_OPERATIONS
pub fn get_transaction_operations<T>(table: &T, txid: &Txid) -> crate::Result<Vec<InscriptionOp>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
        .get(txid.to_string().as_str())? // TODO:optimize key and value
        .map_or(Vec::new(), |v| {
            bincode::deserialize::<Vec<InscriptionOp>>(v.value()).unwrap()
        }),
  )
}

// ORD_TX_TO_OPERATIONS
pub fn save_transaction_operations<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  txid: &Txid,
  operations: &[InscriptionOp],
) -> crate::Result<()> {
  table.insert(
    // TODO:optimize key and value
    txid.to_string().as_str(),
    bincode::serialize(operations).unwrap().as_slice(),
  )?;
  Ok(())
}

// COLLECTIONS_KEY_TO_INSCRIPTION_ID
pub fn set_inscription_by_collection_key<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8; 36]>,
  key: &str,
  inscription_id: &InscriptionId,
) -> crate::Result<()> {
  let mut value = [0; 36];
  let (txid, index) = value.split_at_mut(32);
  txid.copy_from_slice(inscription_id.txid.as_ref());
  index.copy_from_slice(&inscription_id.index.to_be_bytes());
  table.insert(key, &value)?;
  Ok(())
}

// COLLECTIONS_INSCRIPTION_ID_TO_KINDS
pub fn set_inscription_attributes<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static [u8; 36], &'static [u8]>,
  inscription_id: &InscriptionId,
  kind: &[CollectionKind],
) -> crate::Result<()> {
  let mut key = [0; 36];
  let (txid, index) = key.split_at_mut(32);
  txid.copy_from_slice(inscription_id.txid.as_ref());
  index.copy_from_slice(&inscription_id.index.to_be_bytes());
  table.insert(&key, bincode::serialize(&kind).unwrap().as_slice())?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::ord::redb::table::{
    get_transaction_operations, save_transaction_operations,
  };
  use crate::okx::datastore::ord::InscriptionOp;
  use crate::{inscription, okx::datastore::ord::Action, SatPoint};
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;

  #[test]
  fn test_transaction_to_operations() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let mut table = wtx.open_table(ORD_TX_TO_OPERATIONS)?;
    let txid =
      Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();
    let operation = InscriptionOp {
      txid,
      action: Action::New {
        cursed: false,
        unbound: false,
        inscription: inscription("text/plain;charset=utf-8", "foobar"),
      },
      inscription_number: Some(100),
      inscription_id: InscriptionId { txid, index: 0 },
      old_satpoint: SatPoint::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111:1:1",
      )
      .unwrap(),
      new_satpoint: Some(SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 1,
      }),
    };

    save_transaction_operations(&mut table, &txid, &[operation.clone()]).unwrap();

    assert_eq!(
      get_transaction_operations(&table, &txid).unwrap(),
      vec![operation]
    );
  }
}
