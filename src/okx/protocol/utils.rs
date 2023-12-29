use crate::index::{InscriptionEntryValue, InscriptionIdValue, OutPointValue};
use crate::okx::datastore::ord::redb::table::{
  get_number_by_inscription_id, get_txout_by_outpoint,
};
use crate::{okx::datastore::ScriptKey, InscriptionId, SatPoint};
use anyhow::anyhow;
use bitcoin::{Network, OutPoint, TxOut};
use redb::ReadableTable;
use std::collections::HashMap;

pub(super) fn get_script_key_on_satpoint<T>(
  table: &T,
  tx_out_cache: &HashMap<OutPoint, TxOut>,
  satpoint: &SatPoint,
  network: Network,
) -> crate::Result<ScriptKey>
where
  T: ReadableTable<&'static OutPointValue, &'static [u8]>,
{
  if let Some(tx_out) = tx_out_cache.get(&satpoint.outpoint) {
    Ok(ScriptKey::from_script(&tx_out.script_pubkey, network))
  } else if let Some(tx_out) = get_txout_by_outpoint(table, &satpoint.outpoint)? {
    Ok(ScriptKey::from_script(&tx_out.script_pubkey, network))
  } else {
    Err(anyhow!(
      "failed to get tx out! error: outpoint {} not found",
      &satpoint.outpoint
    ))
  }
}

pub(super) fn get_inscription_number_by_id<T>(
  table: &T,
  inscription_id: &InscriptionId,
) -> crate::Result<i64>
where
  T: ReadableTable<&'static InscriptionIdValue, InscriptionEntryValue>,
{
  get_number_by_inscription_id(table, inscription_id)
    .map_err(|e| anyhow!("failed to get inscription number from state! error: {e}"))?
    .ok_or(anyhow!(
      "failed to get inscription number! error: inscription id {} not found",
      inscription_id
    ))
}
