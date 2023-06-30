use super::*;
use crate::{
  inscription::Inscription,
  okx::{
    datastore::{
      brc30::BRC30DataStoreReadOnly,
      ord::{Action, InscriptionOp, OrdDataStoreReadOnly},
    },
    protocol::brc30::deserialize_brc30_operation,
  },
  Index, Result,
};
use anyhow::anyhow;
use bitcoin::{OutPoint, TxOut};
use bitcoincore_rpc::Client;
use std::collections::HashMap;

pub(crate) fn resolve_message<'a, O: OrdDataStoreReadWrite>(
  client: &Client,
  ord_store: &'a O,
  brc30_store: &'a M,
  new_inscriptions: &Vec<Inscription>,
  op: &InscriptionOp,
  outpoint_to_txout_cache: &mut HashMap<OutPoint, TxOut>,
) -> Result<Option<BRC30Message>> {
  let inscription = match op.action {
    Action::New {
      cursed: false,
      unbound: false,
    } => new_inscriptions
      .get(usize::try_from(op.inscription_id.index).unwrap())
      .unwrap()
      .clone(),
    Action::Transfer => match brc30_store.get_inscribe_transfer_inscription(op.inscription_id) {
      Ok(Some(_)) if op.inscription_id.txid == op.old_satpoint.outpoint.txid => {
        Inscription::from_transaction(
          &Index::get_transaction_retries(client, op.inscription_id.txid)?.ok_or(anyhow!(
            "failed to fetch transaction! {} not found",
            op.inscription_id.txid
          ))?,
        )
        .get(usize::try_from(op.inscription_id.index).unwrap())
        .unwrap()
        .inscription
        .clone()
      }
      Err(e) => {
        return Err(anyhow!(
          "failed to get inscribe transfer inscription for {}! error: {e}",
          op.inscription_id,
        ))
      }
      _ => return Ok(None),
    },
    _ => return Ok(None),
  };

  match deserialize_brc30_operation(&inscription, &op.action) {
    Ok(brc20s_operation) => {
      let commit_input_satpoint = match op.action {
        Action::New { .. } => Some(get_commit_input_satpoint(
          client,
          ord_store,
          op.old_satpoint,
          outpoint_to_txout_cache,
        )?),
        Action::Transfer => None,
      };

      Ok(Some(BRC30Message {
        txid: op.txid,
        inscription_id: op.inscription_id,
        old_satpoint: op.old_satpoint,
        new_satpoint: op.new_satpoint,
        commit_input_satpoint,
        op: brc20s_operation,
      }))
    }
    Err(_) => Ok(None),
  }
}

fn get_commit_input_satpoint<'a, O: OrdDataStoreReadOnly>(
  client: &Client,
  ord_store: &'a O,
  satpoint: SatPoint,
  outpoint_to_txout_cache: &mut HashMap<OutPoint, TxOut>,
) -> Result<SatPoint> {
  let commit_transaction =
    &Index::get_transaction_retries(client, satpoint.outpoint.txid)?.ok_or(anyhow!(
      "failed to BRC20S message commit transaction! error: {} not found",
      satpoint.outpoint.txid
    ))?;

  // get satoshi offset
  let mut offset = 0;
  for (vout, output) in commit_transaction.output.iter().enumerate() {
    if vout < usize::try_from(satpoint.outpoint.vout).unwrap() {
      offset += output.value;
      continue;
    }
    offset += satpoint.offset;
    break;
  }

  let mut input_value = 0;
  for input in &commit_transaction.input {
    let value = if let Some(tx_out) = ord_store
      .get_outpoint_to_txout(input.previous_output)
      .map_err(|e| anyhow!("failed to get tx out from state! error: {e}"))?
    {
      tx_out.value
    } else if let Some(tx_out) = Index::get_transaction_retries(client, input.previous_output.txid)?
      .map(|tx| {
        tx.output
          .get(usize::try_from(input.previous_output.vout).unwrap())
          .unwrap()
          .clone()
      })
    {
      outpoint_to_txout_cache.insert(input.previous_output, tx_out.clone());
      tx_out.value
    } else {
      return Err(anyhow!(
        "failed to get tx out! error: {} not found",
        input.previous_output
      ));
    };

    input_value += value;
    if input_value >= offset {
      return Ok(SatPoint {
        outpoint: input.previous_output,
        offset: value - input_value + offset,
      });
    }
  }
  return Err(anyhow!("no match found for the commit offset!"));
}
