use super::*;
use crate::{
  inscription::Inscription,
  okx::{
    datastore::ord::{Action, InscriptionOp},
    protocol::brc0::deserialize_brc0_operation,
  },
  Result,
};

impl Message {
  pub(crate) fn resolve(
    new_inscriptions: &[Inscription],
    op: &InscriptionOp,
    btc_fee: u128,
  ) -> Result<Option<Message>> {
    log::debug!("BRC0 resolving the message from {:?}", op);
    let sat_in_outputs = op
      .new_satpoint
      .map(|satpoint| satpoint.outpoint.txid == op.txid)
      .unwrap_or(false);

    let brc0_operation = match op.action {
      // New inscription is not `cursed` or `unbound`.
      Action::New { .. } if sat_in_outputs => {
        match deserialize_brc0_operation(
          new_inscriptions
            .get(usize::try_from(op.inscription_id.index).unwrap())
            .unwrap(),
          &op.action,
        ) {
          Ok(brc0_operation) => brc0_operation,
          _ => return Ok(None),
        }
      }
      _ => return Ok(None),
    };
    Ok(Some(Self {
      txid: op.txid,
      inscription_id: op.inscription_id,
      old_satpoint: op.old_satpoint,
      new_satpoint: op.new_satpoint,
      op: brc0_operation,
      sat_in_outputs,
      btc_fee,
    }))
  }
}
