use super::*;
use crate::{
  inscription::Inscription,
  okx::{
    datastore::{
      brc20::BRC20DataStoreReadOnly,
      ord::{Action, InscriptionOp},
    },
    protocol::brc20::{deserialize_brc20_operation, Operation},
  },
  Result,
};
use anyhow::anyhow;

pub(crate) fn resolve_message<'a, N: BRC20DataStoreReadOnly>(
  brc20_store: &'a N,
  new_inscriptions: &Vec<Inscription>,
  op: &InscriptionOp,
) -> Result<Option<BRC20Message>> {
  log::debug!("BRC20 resolving the message from {:?}", op);
  let brc20_operation = match op.action {
    Action::New {
      cursed: false,
      unbound: false,
    } => {
      if op.new_satpoint.is_none() || op.new_satpoint.unwrap().outpoint.txid != op.txid {
        log::debug!("BRC20 resolving filtered new inscription on coinbase tx");
        return Ok(None);
      }
      match deserialize_brc20_operation(
        new_inscriptions
          .get(usize::try_from(op.inscription_id.index).unwrap())
          .unwrap(),
        &op.action,
      ) {
        Ok(brc20_operation) => brc20_operation,
        _ => return Ok(None),
      }
    }
    Action::Transfer => match brc20_store.get_inscribe_transfer_inscription(op.inscription_id) {
      Ok(Some(transfer_info)) if op.inscription_id.txid == op.old_satpoint.outpoint.txid => {
        Operation::Transfer(BRC20Transfer {
          tick: transfer_info.tick.as_str().to_string(),
          amount: transfer_info.amt.to_string(),
        })
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
  let msg = BRC20Message {
    txid: op.txid,
    inscription_id: op.inscription_id,
    old_satpoint: op.old_satpoint,
    new_satpoint: op.new_satpoint,
    op: brc20_operation,
  };
  log::debug!("BRC20 resolved the message from {:?}, msg {:?}", op, msg);
  Ok(Some(msg))
}
