use super::*;
use crate::{
  inscription::Inscription,
  okx::{
    datastore::{
      ScriptKey,
      ORD::{Action, InscriptionOp, OrdDataStoreReadOnly},
    },
    protocol::BRC30::deserialize_brc30_operation,
  },
  unbound_outpoint, Result,
};
use anyhow::anyhow;
use bitcoin::{Network, Transaction, Txid};
use bitcoincore_rpc::{Client, RpcApi};
use std::{thread, time::Duration};

pub fn resolve_message<'a, O: OrdDataStoreReadOnly>(
  client: &Client,
  network: Network,
  ord_store: &'a O,
  txid: &Txid,
  block_height: u64,
  block_time: u32,
  tx: &Transaction,
  inscription_op: &InscriptionOp,
) -> Result<Option<BRC30Message>> {
  // ignore cursed and unbound
  let number = ord_store.get_number_by_inscription_id(inscription_op.inscription_id)?;
  if number < 0 || inscription_op.new_satpoint.outpoint == unbound_outpoint() {
    return Ok(None);
  }

  let inscription = match inscription_op.action {
    Action::New => Inscription::from_transaction(tx)
      .get(usize::try_from(inscription_op.inscription_id.index).unwrap())
      .unwrap()
      .inscription
      .clone(),
    Action::Transfer => {
      // ignored if the inscription is not the first transfer.
      if inscription_op.inscription_id.txid != inscription_op.old_satpoint.outpoint.txid {
        return Ok(None);
      }
      Inscription::from_transaction(&get_transaction_with_retries(
        client,
        inscription_op.inscription_id.txid,
      )?)
      .get(usize::try_from(inscription_op.inscription_id.index).unwrap())
      .unwrap()
      .inscription
      .clone()
    }
  };

  if let Ok(brc20_operation) = deserialize_brc30_operation(&inscription, &inscription_op.action) {
    let from = ScriptKey::from_script(
      &ord_store
        .get_outpoint_to_txout(inscription_op.old_satpoint.outpoint)?
        .ok_or(anyhow!(format!(
          "outpoint {} not found",
          inscription_op.old_satpoint.outpoint
        )))?
        .script_pubkey,
      network,
    );

    let to = ScriptKey::from_script(
      &ord_store
        .get_outpoint_to_txout(inscription_op.new_satpoint.outpoint)?
        .ok_or(anyhow!(format!(
          "outpoint {} not found",
          inscription_op.new_satpoint.outpoint
        )))?
        .script_pubkey,
      network,
    );

    let commit_from = match inscription_op.action {
      Action::New => Some(get_origin_scriptkey(
        client,
        network,
        ord_store,
        inscription_op.old_satpoint,
      )?),
      Action::Transfer => None,
    };

    Ok(Some(BRC30Message {
      txid: txid.clone(),
      block_height,
      block_time,
      inscription_id: inscription_op.inscription_id,
      inscription_number: number,
      old_satpoint: inscription_op.old_satpoint,
      new_satpoint: inscription_op.new_satpoint,
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
  let transaction = get_transaction_with_retries(client, satpoint.outpoint.txid)?;
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
      .ok_or(anyhow!(format!(
        "outpoint {} not found",
        input.previous_output
      )))?;
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

fn get_transaction_with_retries(client: &Client, txid: Txid) -> Result<Transaction> {
  let mut errors = 0;
  loop {
    match client.get_raw_transaction(&txid, None) {
      Err(err) => {
        errors += 1;
        let seconds = 1 << errors;
        log::warn!("failed to fetch transaction {txid}, retrying in {seconds}s: {err}");

        if seconds > 120 {
          log::error!("would sleep for more than 120s, giving up");
          return Err(anyhow!("failed to fetch transaction {txid}"));
        }

        thread::sleep(Duration::from_secs(seconds));
      }
      Ok(result) => return Ok(result),
    }
  }
}
