use crate::{
  inscription::Inscription,
  okx::{
    datastore::{
      ord::{Action, InscriptionOp, OrdDataStoreReadOnly},
      ScriptKey,
    },
    protocol::brc20::deserialize_brc20_operation,
  },
  unbound_outpoint, Result,
};
use anyhow::anyhow;
use bitcoin::{Network, Transaction, Txid};
use bitcoincore_rpc::{Client, RpcApi};
use std::{thread, time::Duration};

use super::BRC20Message;

pub fn resolve_message<'a, O: OrdDataStoreReadOnly>(
  client: &Client,
  network: Network,
  ord_store: &'a O,
  txid: &Txid,
  block_height: u64,
  block_time: u32,
  tx: &Transaction,
  inscription_op: &InscriptionOp,
) -> Result<Option<BRC20Message>> {
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

  if let Ok(brc20_operation) = deserialize_brc20_operation(&inscription, &inscription_op.action) {
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
    Ok(Some(BRC20Message {
      txid: txid.clone(),
      block_height,
      block_time,
      inscription_id: inscription_op.inscription_id,
      inscription_number: number,
      old_satpoint: inscription_op.old_satpoint,
      new_satpoint: inscription_op.new_satpoint,
      from,
      to,
      op: brc20_operation,
    }))
  } else {
    Ok(None)
  }
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
