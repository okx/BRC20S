use crate::{
  okx::datastore::{ord::DataStoreReadOnly, ScriptKey},
  InscriptionId, Result, SatPoint,
};
use anyhow::anyhow;
use bitcoin::{Network, OutPoint};

pub(super) fn get_script_key_on_satpoint<O: DataStoreReadOnly>(
  satpoint: SatPoint,
  ord_store: &O,
  network: Network,
) -> Result<ScriptKey> {
  get_script_key_on_out_point(satpoint.outpoint, ord_store, network)
}

pub(super) fn get_script_key_on_out_point<O: DataStoreReadOnly>(
  out_point: OutPoint,
  ord_store: &O,
  network: Network,
) -> Result<ScriptKey> {
  Ok(ScriptKey::from_script(
    &ord_store
      .get_outpoint_to_txout(out_point)
      .map_err(|e| anyhow!("failed to get tx out from state! error: {e}",))?
      .ok_or(anyhow!(
        "failed to get tx out! error: outpoint {} not found",
        out_point
      ))?
      .script_pubkey,
    network,
  ))
}

pub(super) fn get_inscription_number_by_id<O: DataStoreReadOnly>(
  inscription_id: InscriptionId,
  ord_store: &O,
) -> Result<i64> {
  ord_store
    .get_number_by_inscription_id(inscription_id)
    .map_err(|e| anyhow!("failed to get inscription number from state! error: {e}"))?
    .ok_or(anyhow!(
      "failed to get inscription number! error: inscription id {} not found",
      inscription_id
    ))
}
