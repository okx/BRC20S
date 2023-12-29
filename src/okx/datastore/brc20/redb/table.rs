use crate::inscription_id::InscriptionId;
use crate::okx::datastore::brc20::redb::{
  max_script_tick_key, min_script_tick_key, script_tick_key,
};
use crate::okx::datastore::brc20::{
  Balance, Receipt, Tick, TokenInfo, TransferInfo, TransferableLog,
};
use crate::okx::datastore::ScriptKey;
use bitcoin::Txid;
use redb::{ReadableTable, Table};

// BRC20_BALANCES
pub fn get_balances<T>(table: &T, script_key: &ScriptKey) -> crate::Result<Vec<Balance>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .range(min_script_tick_key(script_key).as_str()..max_script_tick_key(script_key).as_str())?
      .flat_map(|result| {
        result.map(|(_, data)| bincode::deserialize::<Balance>(data.value()).unwrap())
      })
      .collect(),
  )
}

// BRC20_BALANCES
pub fn get_balance<T>(
  table: &T,
  script_key: &ScriptKey,
  tick: &Tick,
) -> crate::Result<Option<Balance>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .get(script_tick_key(script_key, tick).as_str())?
      .map(|v| bincode::deserialize::<Balance>(v.value()).unwrap()),
  )
}

// BRC20_TOKEN
pub fn get_token_info<T>(table: &T, tick: &Tick) -> crate::Result<Option<TokenInfo>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .get(tick.to_lowercase().hex().as_str())?
      .map(|v| bincode::deserialize::<TokenInfo>(v.value()).unwrap()),
  )
}

// BRC20_TOKEN
pub fn get_tokens_info<T>(table: &T) -> crate::Result<Vec<TokenInfo>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .range::<&str>(..)?
      .flat_map(|result| {
        result.map(|(_, data)| bincode::deserialize::<TokenInfo>(data.value()).unwrap())
      })
      .collect(),
  )
}

// BRC20_EVENTS
pub fn get_transaction_receipts<T>(table: &T, txid: &Txid) -> crate::Result<Vec<Receipt>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .get(txid.to_string().as_str())?
      .map_or(Vec::new(), |v| {
        bincode::deserialize::<Vec<Receipt>>(v.value()).unwrap()
      }),
  )
}

// BRC20_TRANSFERABLELOG
pub fn get_transferable<T>(table: &T, script: &ScriptKey) -> crate::Result<Vec<TransferableLog>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .range(min_script_tick_key(script).as_str()..max_script_tick_key(script).as_str())?
      .flat_map(|result| {
        result.map(|(_, v)| bincode::deserialize::<Vec<TransferableLog>>(v.value()).unwrap())
      })
      .flatten()
      .collect(),
  )
}

// BRC20_TRANSFERABLELOG
pub fn get_transferable_by_tick<T>(
  table: &T,
  script: &ScriptKey,
  tick: &Tick,
) -> crate::Result<Vec<TransferableLog>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .get(script_tick_key(script, tick).as_str())?
      .map_or(Vec::new(), |v| {
        bincode::deserialize::<Vec<TransferableLog>>(v.value()).unwrap()
      }),
  )
}

// BRC20_TRANSFERABLELOG
pub fn get_transferable_by_id<T>(
  table: &T,
  script: &ScriptKey,
  inscription_id: &InscriptionId,
) -> crate::Result<Option<TransferableLog>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    get_transferable(table, script)?
      .iter()
      .find(|log| log.inscription_id == *inscription_id)
      .cloned(),
  )
}

// BRC20_INSCRIBE_TRANSFER
pub fn get_inscribe_transfer_inscription<T>(
  table: &T,
  inscription_id: &InscriptionId,
) -> crate::Result<Option<TransferInfo>>
where
  T: ReadableTable<&'static [u8; 36], &'static [u8]>,
{
  let mut value = [0; 36];
  let (txid, index) = value.split_at_mut(32);
  txid.copy_from_slice(inscription_id.txid.as_ref());
  index.copy_from_slice(&inscription_id.index.to_be_bytes());
  Ok(
    table
      .get(&value)?
      .map(|v| bincode::deserialize::<TransferInfo>(v.value()).unwrap()),
  )
}

// BRC20_BALANCES
pub fn update_token_balance<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  script_key: &ScriptKey,
  new_balance: Balance,
) -> crate::Result<()> {
  table.insert(
    script_tick_key(script_key, &new_balance.tick).as_str(),
    bincode::serialize(&new_balance).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_TOKEN
pub fn insert_token_info<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  tick: &Tick,
  new_info: &TokenInfo,
) -> crate::Result<()> {
  table.insert(
    tick.to_lowercase().hex().as_str(),
    bincode::serialize(new_info).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_TOKEN
pub fn update_mint_token_info<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  tick: &Tick,
  minted_amt: u128,
  minted_block_number: u64,
) -> crate::Result<()> {
  let mut info =
    get_token_info(table, tick)?.unwrap_or_else(|| panic!("token {} not exist", tick.as_str()));

  info.minted = minted_amt;
  info.latest_mint_number = minted_block_number;

  table.insert(
    tick.to_lowercase().hex().as_str(),
    bincode::serialize(&info).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_EVENTS
pub fn save_transaction_receipts<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  txid: &Txid,
  receipts: &[Receipt],
) -> crate::Result<()> {
  table.insert(
    txid.to_string().as_str(),
    bincode::serialize(receipts).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_EVENTS
pub fn add_transaction_receipt<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  txid: &Txid,
  receipt: &Receipt,
) -> crate::Result<()> {
  let mut receipts = get_transaction_receipts(table, txid)?;
  receipts.push(receipt.clone());
  save_transaction_receipts(table, txid, &receipts)
}

// BRC20_TRANSFERABLELOG
pub fn insert_transferable<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  script: &ScriptKey,
  tick: &Tick,
  inscription: TransferableLog,
) -> crate::Result<()> {
  let mut logs = get_transferable_by_tick(table, script, tick)?;
  if logs
    .iter()
    .any(|log| log.inscription_id == inscription.inscription_id)
  {
    return Ok(());
  }

  logs.push(inscription);

  table.insert(
    script_tick_key(script, tick).as_str(),
    bincode::serialize(&logs).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_TRANSFERABLELOG
pub fn remove_transferable<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  script: &ScriptKey,
  tick: &Tick,
  inscription_id: &InscriptionId,
) -> crate::Result<()> {
  let mut logs = get_transferable_by_tick(table, script, tick)?;
  let old_len = logs.len();

  logs.retain(|log| &log.inscription_id != inscription_id);

  if logs.len() != old_len {
    table.insert(
      script_tick_key(script, tick).as_str(),
      bincode::serialize(&logs).unwrap().as_slice(),
    )?;
  }
  Ok(())
}

// BRC20_INSCRIBE_TRANSFER
pub fn insert_inscribe_transfer_inscription<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static [u8; 36], &'static [u8]>,
  inscription_id: &InscriptionId,
  transfer_info: TransferInfo,
) -> crate::Result<()> {
  let mut value = [0; 36];
  let (txid, index) = value.split_at_mut(32);
  txid.copy_from_slice(inscription_id.txid.as_ref());
  index.copy_from_slice(&inscription_id.index.to_be_bytes());

  table.insert(
    &value,
    bincode::serialize(&transfer_info).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_INSCRIBE_TRANSFER
pub fn remove_inscribe_transfer_inscription<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static [u8; 36], &'static [u8]>,
  inscription_id: &InscriptionId,
) -> crate::Result<()> {
  let mut value = [0; 36];
  let (txid, index) = value.split_at_mut(32);
  txid.copy_from_slice(inscription_id.txid.as_ref());
  index.copy_from_slice(&inscription_id.index.to_be_bytes());

  table.remove(&value)?;
  Ok(())
}
