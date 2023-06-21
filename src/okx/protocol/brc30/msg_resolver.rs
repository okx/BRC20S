use super::*;
use crate::{
  inscription::Inscription,
  okx::{
    datastore::{
      ord::{Action, InscriptionOp, OrdDataStoreReadOnly},
      ScriptKey,
    },
    protocol::brc30::deserialize_brc30_operation,
  },
  unbound_outpoint, Index, Result,
};
use anyhow::anyhow;
use bitcoin::Network;
use bitcoincore_rpc::Client;

pub fn resolve_message<'a, O: OrdDataStoreReadOnly>(
  client: &Client,
  network: Network,
  ord_store: &'a O,
  block_height: u64,
  block_time: u32,
  new_inscriptions: &Vec<Inscription>,
  op: &InscriptionOp,
) -> Result<Option<BRC30Message>> {
  let inscription_number = ord_store.get_number_by_inscription_id(op.inscription_id)?;

  // Ignore cursed and unbound inscription.
  // There is a bug in ordinals that may cause unbound inscriptions to occupy normal inscription numbers. The code needs to be reviewed after this bug is fixed.
  // https://github.com/ordinals/ord/issues/2202
  if inscription_number < 0 || op.new_satpoint.outpoint == unbound_outpoint() {
    return Ok(None);
  }

  let inscription = match op.action {
    Action::New => new_inscriptions
      .get(usize::try_from(op.inscription_id.index).unwrap())
      .unwrap()
      .clone(),
    Action::Transfer => {
      // Ignored if the inscription is not the first transfer.
      if op.inscription_id.txid != op.old_satpoint.outpoint.txid {
        return Ok(None);
      }
      Inscription::from_transaction(
        &Index::get_transaction_with_retries(client, op.inscription_id.txid)?
          .ok_or(anyhow!("transaction not found {}", op.inscription_id.txid))?,
      )
      .get(usize::try_from(op.inscription_id.index).unwrap())
      .unwrap()
      .inscription
      .clone()
    }
  };

  if let Ok(brc20_operation) = deserialize_brc30_operation(&inscription, &op.action) {
    let from = Index::get_outpoint_script_key(ord_store, op.old_satpoint.outpoint, network)?
      .ok_or(anyhow!("outpoint {} not found", op.old_satpoint.outpoint))?;

    let to = Index::get_outpoint_script_key(ord_store, op.new_satpoint.outpoint, network)?
      .ok_or(anyhow!("outpoint {} not found", op.new_satpoint.outpoint))?;

    let commit_from = match op.action {
      Action::New => Some(get_origin_scriptkey(
        client,
        network,
        ord_store,
        op.old_satpoint,
      )?),
      Action::Transfer => None,
    };

    Ok(Some(BRC30Message {
      txid: op.txid,
      block_height,
      block_time,
      inscription_id: op.inscription_id,
      inscription_number,
      old_satpoint: op.old_satpoint,
      new_satpoint: op.new_satpoint,
      commit_from,
      from,
      to,
      op: brc20_operation,
    }))
  } else {
    Ok(None)
  }
}

fn get_origin_scriptkey<'a, O: OrdDataStoreReadOnly>(
  client: &Client,
  network: Network,
  ord_store: &'a O,
  satpoint: SatPoint,
) -> Result<ScriptKey> {
  let transaction = &Index::get_transaction_with_retries(client, satpoint.outpoint.txid)?
    .ok_or(anyhow!("transaction not found {}", satpoint.outpoint.txid))?;
  let mut offset = 0;
  for (vout, output) in transaction.output.iter().enumerate() {
    if vout < usize::try_from(satpoint.outpoint.vout).unwrap() {
      offset += output.value;
      continue;
    }
    offset += satpoint.offset;
    break;
  }

  let mut input_value = 0;
  for (_, input) in transaction.input.iter().enumerate() {
    let prev_outpoint = ord_store
      .get_outpoint_to_txout(input.previous_output)?
      .ok_or(anyhow!("outpoint {} not found", input.previous_output))?;
    input_value += prev_outpoint.value;
    if input_value >= offset {
      return Ok(ScriptKey::from_script(
        &prev_outpoint.script_pubkey,
        network,
      ));
    }
  }
  return Err(anyhow!("origin satpoint not found"));
}
