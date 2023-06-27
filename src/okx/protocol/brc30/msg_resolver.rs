use super::*;
use crate::{
  inscription::Inscription,
  okx::{
    datastore::ord::{Action, InscriptionOp, OrdDataStoreReadOnly},
    protocol::brc30::deserialize_brc30_operation,
  },
  Index, Result,
};
use anyhow::anyhow;
use bitcoincore_rpc::Client;

pub fn resolve_message<'a, O: OrdDataStoreReadOnly>(
  client: &Client,
  ord_store: &'a O,
  new_inscriptions: &Vec<Inscription>,
  op: &InscriptionOp,
) -> Result<Option<BRC30Message>> {
  // Ignore cursed and unbound inscriptions.
  if op.inscription_id.index > 0 {
    return Ok(None);
  }

  let inscription = match op.action {
    Action::New {
      cursed: false,
      unbound: false,
    } => new_inscriptions
      .get(usize::try_from(op.inscription_id.index).unwrap())
      .unwrap()
      .clone(),
    Action::Transfer => {
      // Ignored if the inscription is not the first transfer.
      if op.inscription_id.txid != op.old_satpoint.outpoint.txid {
        return Ok(None);
      }
      match ord_store.get_number_by_inscription_id(op.inscription_id) {
        Ok(Some(inscription_number)) => {
          // Ignore negative number inscriptions.
          if inscription_number >= 0 {
            Inscription::from_transaction(
              &Index::get_transaction_with_retries(client, op.inscription_id.txid)?.ok_or(
                anyhow!(
                  "failed to fetch transaction! {} not found",
                  op.inscription_id.txid
                ),
              )?,
            )
            .get(usize::try_from(op.inscription_id.index).unwrap())
            .unwrap()
            .inscription
            .clone()
          } else {
            return Ok(None);
          }
        }
        _ => return Ok(None),
      }
    }
    _ => return Ok(None),
  };

  if let Ok(brc30_operation) = deserialize_brc30_operation(&inscription, &op.action) {
    let commit_input_satpoint = match op.action {
      Action::New { .. } => Some(get_commit_input_satpoint(
        client,
        ord_store,
        op.old_satpoint,
      )?),
      Action::Transfer => None,
    };

    Ok(Some(BRC30Message {
      txid: op.txid,
      inscription_id: op.inscription_id,
      old_satpoint: op.old_satpoint,
      new_satpoint: op.new_satpoint,
      commit_input_satpoint,
      op: brc30_operation,
    }))
  } else {
    Ok(None)
  }
}

fn get_commit_input_satpoint<'a, O: OrdDataStoreReadOnly>(
  client: &Client,
  ord_store: &'a O,
  satpoint: SatPoint,
) -> Result<SatPoint> {
  let transaction =
    &Index::get_transaction_with_retries(client, satpoint.outpoint.txid)?.ok_or(anyhow!(
      "failed to fetch transaction! {} not found",
      satpoint.outpoint.txid
    ))?;

  // get satoshi offset
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
  for input in &transaction.input {
    let prevout = ord_store
      .get_outpoint_to_txout(input.previous_output)
      .map_err(|e| anyhow!("failed to get txout from state! error: {e}"))?
      .ok_or(anyhow!(
        "failed to get txout! {} not found",
        input.previous_output
      ))?;
    input_value += prevout.value;
    if input_value >= offset {
      return Ok(SatPoint {
        outpoint: input.previous_output,
        offset: prevout.value - input_value + offset,
      });
    }
  }
  return Err(anyhow!("no match found for the commit offset!"));
}
