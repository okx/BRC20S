use super::BRC20Message;
use crate::{
  inscription::Inscription,
  okx::{
    datastore::{
      ord::{Action, InscriptionOp, OrdDataStoreReadOnly},
      ScriptKey,
    },
    protocol::brc20::deserialize_brc20_operation,
  },
  Index, Result,
};
use anyhow::anyhow;
use bitcoin::Network;
use bitcoincore_rpc::Client;

pub fn resolve_message<'a, O: OrdDataStoreReadOnly>(
  client: &Client,
  network: Network,
  ord_store: &'a O,
  block_height: Option<u64>,
  block_time: Option<u32>,
  new_inscriptions: &Vec<Inscription>,
  op: &InscriptionOp,
) -> Result<Option<BRC20Message>> {
  // Ignore cursed and unbound inscription.
  // There is a bug in ordinals that may cause unbound inscriptions to occupy normal inscription numbers. The code needs to be reviewed after this bug is fixed.
  // https://github.com/ordinals/ord/issues/2202

  let inscription = match op.action {
    Action::New { .. } => new_inscriptions
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

  if let Ok(brc20_operation) = deserialize_brc20_operation(&inscription, &op.action) {
    let from = ScriptKey::from_script(
      &ord_store
        .get_outpoint_to_txout(op.old_satpoint.outpoint)?
        .ok_or(anyhow!("outpoint {} not found", op.old_satpoint.outpoint))?
        .script_pubkey,
      network,
    );

    let to = match op.new_satpoint {
      Some(satpoint) => ScriptKey::from_script(
        &ord_store
          .get_outpoint_to_txout(satpoint.outpoint)?
          .ok_or(anyhow!("outpoint {} not found", satpoint.outpoint))?
          .script_pubkey,
        network,
      ),
      None => ScriptKey::UnKnown,
    };

    Ok(Some(BRC20Message {
      txid: op.txid,
      block_height,
      block_time,
      inscription_id: op.inscription_id,
      inscription_number: op.inscription_number,
      old_satpoint: op.old_satpoint,
      new_satpoint: op.new_satpoint,
      from,
      to,
      op: brc20_operation,
    }))
  } else {
    Ok(None)
  }
}
