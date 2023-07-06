use super::*;
use crate::brc20::{deserialize_brc20_operation, Operation};

fn index_brc20_operations(
  Extension(index): Extension<Arc<Index>>,
  tx: &Transaction,
) -> Result<Vec<InscriptionInfo>> {
  let mut operations: Vec<(u64, InscriptionInfo)> = Vec::new();
  let mut input_value = 0;
  let mut script_cache = Script::default();

  for tx_in in &tx.input {
    // skipped coinbase transaction
    if tx_in.previous_output.is_null() {
      return Ok(Vec::new());
    }

    // request previous transaction
    let prev_tx = index
      .get_transaction(tx_in.previous_output.txid)?
      .ok_or(anyhow!(format!(
        "failed to get pervious transaction {}",
        tx_in.previous_output.txid
      )))?;

    if tx_in == tx.input.get(0).unwrap() {
      script_cache = prev_tx.output.get(0).unwrap().script_pubkey.clone();
    }

    if tx_in.previous_output.vout == 0 {
      // collect the transfer operation if the previous is a inscribed transfer operation.
      if let Some(Operation::Transfer(transfer)) = Inscription::from_transaction(&prev_tx)
        .and_then(|v| deserialize_brc20_operation(v, true).ok())
      {
        let id = InscriptionId::from(tx_in.previous_output.txid);
        operations.push((
          input_value,
          InscriptionInfo {
            action: ActionType::Transfer,
            inscription_number: index.get_inscription_entry(id)?.map(|v| v.number),
            inscription_id: id.to_string(),
            from: ScriptPubkey::from_script(
              &prev_tx.output.get(0).unwrap().script_pubkey,
              index.get_chain_network(),
            ),
            // set default and fill back later
            to: ScriptPubkey::default(),
            old_satpoint: SatPoint {
              outpoint: tx_in.previous_output,
              offset: 0,
            }
            .to_string(),
            new_satpoint: None,
            operation: Some(RawOperation::Brc20Operation(Brc20RawOperation::Transfer(
              transfer.into(),
            ))),
          },
        ))
      }
    }
    input_value += prev_tx
      .output
      .get(tx_in.previous_output.vout as usize)
      .unwrap()
      .value;
  }

  // new inscription
  if operations.iter().all(|(offset, _)| *offset != 0) && input_value > 0 {
    if let Some(op) =
      Inscription::from_transaction(&tx).and_then(|v| deserialize_brc20_operation(v, false).ok())
    {
      let id = InscriptionId::from(tx.txid());
      operations.insert(
        0,
        (
          0,
          InscriptionInfo {
            action: ActionType::Inscribe,
            inscription_number: index.get_inscription_entry(id)?.map(|v| v.number),
            inscription_id: id.to_string(),
            from: ScriptPubkey::from_script(&script_cache, index.get_chain_network()),
            to: ScriptPubkey::default(),
            old_satpoint: SatPoint {
              outpoint: tx.input.get(0).unwrap().previous_output,
              offset: 0,
            }
            .to_string(),
            new_satpoint: None,
            operation: Some(RawOperation::Brc20Operation(match op {
              Operation::Deploy(deploy) => Brc20RawOperation::Deploy(deploy.into()),
              Operation::Mint(mint) => Brc20RawOperation::Mint(mint.into()),
              Operation::Transfer(transfer) => Brc20RawOperation::InscribeTransfer(transfer.into()),
            })),
          },
        ),
      );
    }
  }

  // fill new_satpoint and to field
  let mut peeker = operations.into_iter().peekable();
  let mut operations = Vec::new();
  let mut output_value = 0;
  for (vout, tx_out) in tx.output.iter().enumerate() {
    let end = output_value + tx_out.value;

    while let Some((offset, op)) = peeker.peek_mut() {
      if *offset >= end {
        break;
      }
      op.new_satpoint = Some(
        SatPoint {
          outpoint: OutPoint {
            txid: tx.txid(),
            vout: vout.try_into().unwrap(),
          },
          offset: *offset - output_value,
        }
        .to_string(),
      );
      op.to = ScriptPubkey::from_script(&tx_out.script_pubkey, index.get_chain_network());
      operations.push(peeker.next().unwrap().1.clone());
    }
    output_value = end;
  }

  // fill 'to' field with 'from' if the inscription is transferd to coinbase.
  while let Some((_, op)) = peeker.peek_mut() {
    op.to = op.from.clone();
    operations.push(peeker.next().unwrap().1.clone());
  }
  Ok(operations)
}

pub(super) fn get_operations_by_txid(
  Extension(index): Extension<Arc<Index>>,
  txid: &bitcoin::Txid,
) -> Result<TxInscriptionInfo> {
  // TODO: search in database.
  let tx = index
    .get_transaction_info(txid)?
    .ok_or(anyhow!("can't get transaction info: {txid}"))?;

  // if the transaction is not confirmed, try to parsing protocol
  Ok(TxInscriptionInfo {
    txid: txid.to_string(),
    blockhash: tx.blockhash.map(|v| v.to_string()),
    confirmed: tx.blockhash.is_some(),
    inscriptions: index_brc20_operations(Extension(index), &tx.transaction()?)?,
  })
}
