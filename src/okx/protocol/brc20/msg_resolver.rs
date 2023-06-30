use super::BRC20Message;
use crate::{
  inscription::Inscription,
  okx::{
    datastore::ord::{Action, InscriptionOp, OrdDataStoreReadOnly},
    protocol::brc20::deserialize_brc20_operation,
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
) -> Result<Option<BRC20Message>> {
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
              &Index::get_transaction_retries(client, op.inscription_id.txid)?.ok_or(anyhow!(
                "failed to fetch transaction! {} not found",
                op.inscription_id.txid
              ))?,
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

  Ok(
    deserialize_brc20_operation(&inscription, &op.action)
      .map(|brc20_operation| BRC20Message {
        txid: op.txid,
        inscription_id: op.inscription_id,
        old_satpoint: op.old_satpoint,
        new_satpoint: op.new_satpoint,
        op: brc20_operation,
      })
      .ok(),
  )
}
