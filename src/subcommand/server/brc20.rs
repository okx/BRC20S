use super::{ord::get_ord_operations_by_txid, *};
use crate::okx::{datastore::ord::OrdDbReader, protocol::brc20};

pub(super) fn get_operations_by_txid(
  index: &Arc<Index>,
  txid: &bitcoin::Txid,
) -> Result<TxInscriptionInfo> {
  let mut brc20_operation_infos = Vec::new();

  let tx_result = index
    .get_transaction_info(txid)?
    .ok_or(anyhow!("can't get transaction info: {txid}"))?;

  let block_height = match tx_result.blockhash {
    Some(blockhash) => Some(
      u64::try_from(
        index
          .block_header_info(blockhash)?
          .ok_or(anyhow!("can't get block info: {blockhash}"))?
          .height,
      )
      .unwrap(),
    ),
    None => None,
  };

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
    match brc20::resolve_message(
      index.client(),
      index.get_chain_network(),
      &ord_store,
      block_height,
      tx_result.blocktime.map(|v| u32::try_from(v).unwrap()),
      &new_inscriptions,
      &operation,
    )? {
      None => continue,
      Some(msg) => brc20_operation_infos.push(InscriptionInfo {
        action: match msg.op {
          brc20::BRC20Operation::Transfer(_) => ActionType::Transfer,
          _ => ActionType::Inscribe,
        },
        inscription_number: msg.inscription_number,
        inscription_id: msg.inscription_id.to_string(),
        from: msg.from.into(),
        to: msg.to.into(),
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
