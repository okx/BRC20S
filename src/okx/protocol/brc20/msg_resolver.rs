use super::BRC20Message;
use crate::{
  inscription::Inscription,
  okx::{
    datastore::{
      brc20::BRC20DataStoreReadOnly,
      ord::{Action, InscriptionOp},
    },
    protocol::brc20::deserialize_brc20_operation,
  },
  Index, Result,
};
use anyhow::anyhow;
use bitcoincore_rpc::Client;

pub(crate) fn resolve_message<'a, O: OrdDataStoreReadOnly>(
  client: &Client,
  brc20_store: &'a N,
  new_inscriptions: &Vec<Inscription>,
  op: &InscriptionOp,
) -> Result<Option<BRC20Message>> {
  let inscription = match op.action {
    Action::New {
      cursed: false,
      unbound: false,
    } => new_inscriptions
      .get(usize::try_from(op.inscription_id.index).unwrap())
      .unwrap()
      .clone(),
    Action::Transfer => match brc20_store.get_inscribe_transfer_inscription(op.inscription_id) {
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

  match deserialize_brc20_operation(&inscription, &op.action) {
    Ok(brc20_operation) => Ok(Some(BRC20Message {
      txid: op.txid,
      inscription_id: op.inscription_id,
      old_satpoint: op.old_satpoint,
      new_satpoint: op.new_satpoint,
      op: brc20_operation,
    })),
    Err(_) => Ok(None),
  }
}
