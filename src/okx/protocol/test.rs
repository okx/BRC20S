use shadow_rs::new;
use std::str::FromStr;
pub(crate) use {
  super::*, crate::inscription_id::InscriptionId, crate::okx::datastore::ScriptKey,
  crate::okx::protocol::brc30::operation::BRC30Operation,
  crate::okx::protocol::brc30::BRC30ExecutionMessage, crate::SatPoint,
};

pub(crate) fn mock_create_brc30_message(
  inscription_id: InscriptionId,
  from: ScriptKey,
  to: ScriptKey,
  op: BRC30Operation,
) -> BRC30ExecutionMessage {
  let txid = inscription_id.txid.clone();
  let old_satpoint =
    SatPoint::from_str("1111111111111111111111111111111111111111111111111111111111111111:1:1")
      .unwrap();
  let new_satpoint =
    SatPoint::from_str("1111111111111111111111111111111111111111111111111111111111111111:2:1")
      .unwrap();
  let msg = BRC30ExecutionMessage::new(
    &txid,
    &inscription_id,
    0,
    &None,
    &old_satpoint,
    &new_satpoint,
    &Some(from.clone()),
    &from,
    &to,
    &op,
  );
  msg
}
