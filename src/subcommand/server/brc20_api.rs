use super::{ord::get_ord_operations_by_txid, *};
use crate::okx::{
  datastore::{ord::OrdDbReader, ScriptKey},
  protocol::brc20,
};

pub(super) fn get_operations_by_txid(
  index: &Arc<Index>,
  txid: &bitcoin::Txid,
) -> Result<TxInscriptionInfo> {
  let mut brc20_operation_infos = Vec::new();

  let tx_result = index
    .get_transaction_info(txid)?
    .ok_or(anyhow!("can't get transaction info: {txid}"))?;

  // get inscription operations
  let operations = get_ord_operations_by_txid(index, txid)?;

  // get new inscriptions
  let new_inscriptions = Inscription::from_transaction(&tx_result.transaction()?)
    .into_iter()
    .map(|i| i.inscription)
    .collect();

  let rtx = index.begin_read()?.0;
  let ord_store = OrdDbReader::new(&rtx);
  for operation in operations {
    match brc20::resolve_message(index.client(), &ord_store, &new_inscriptions, &operation)? {
      None => continue,
      Some(msg) => brc20_operation_infos.push(InscriptionInfo {
        action: match msg.op {
          brc20::BRC20Operation::Transfer(_) => ActionType::Transfer,
          _ => ActionType::Inscribe,
        },
        inscription_number: index
          .get_inscription_entry(msg.inscription_id)?
          .map(|entry| entry.number),
        inscription_id: msg.inscription_id.to_string(),
        from: index
          .get_outpoint_entry(&msg.old_satpoint.outpoint)?
          .map(|txout| {
            ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()).into()
          })
          .ok_or(anyhow!("outpoint not found {}", msg.old_satpoint.outpoint))?,
        to: match msg.new_satpoint {
          Some(satpoint) => match index.get_outpoint_entry(&satpoint.outpoint) {
            Ok(Some(txout)) => {
              Some(ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()).into())
            }
            Ok(None) => return Err(anyhow!("outpoint not found {}", satpoint.outpoint)),
            Err(e) => return Err(e),
          },
          None => None,
        },
        old_satpoint: msg.old_satpoint.to_string(),
        new_satpoint: msg.new_satpoint.map(|v| v.to_string()),
        operation: Some(RawOperation::Brc20Operation(msg.op.into())),
      }),
    };
  }
  // if the transaction is not confirmed, try to parsing protocol
  Ok(TxInscriptionInfo {
    txid: txid.to_string(),
    blockhash: tx_result.blockhash.map(|v| v.to_string()),
    confirmed: tx_result.blockhash.is_some(),
    inscriptions: brc20_operation_infos,
  })
}
