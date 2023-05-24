use super::*;

fn get_unconfirmed_inscriptions(
  Extension(index): Extension<Arc<Index>>,
  tx: &Transaction,
) -> Result<Vec<InscriptionInfo>> {
  let mut infos: Vec<(u64, InscriptionInfo)> = Vec::new();
  let mut input_value = 0;

  for tx_in in &tx.input {
    // skipped coinbase transaction
    if tx_in.previous_output.is_null() {
      return Ok(Vec::new());
    }

    let inscriptions_location =
      index.get_inscriptions_with_satpoint_on_output(tx_in.previous_output)?;

    // TODO: Network bottleneck here, consider using caching in the future.
    let prevtransaction = index
      .get_transaction(tx_in.previous_output.txid)?
      .ok_or(anyhow!(format!(
        "failed to get pervious transaction {}",
        tx_in.previous_output.txid
      )))?;

    // get the inscription for the prevout.
    for (old_satpoint, inscription_id) in inscriptions_location {
      infos.push((
        input_value + old_satpoint.offset,
        InscriptionInfo {
          action: ActionType::Transfer,
          inscription_number: index
            .get_inscription_entry(inscription_id)?
            .map(|v| v.number),
          inscription_id: inscription_id.to_string(),
          from: ScriptPubkey::from_script(
            &prevtransaction
              .output
              .get(tx_in.previous_output.vout as usize)
              .unwrap()
              .script_pubkey,
            index.get_chain_network(),
          ),
          // set default and fill back later
          to: ScriptPubkey::default(),
          old_satpoint: old_satpoint.to_string(),
          new_satpoint: None,
          operation: None,
        },
      ))
    }
    input_value += prevtransaction
      .output
      .get(tx_in.previous_output.vout as usize)
      .unwrap()
      .value;
  }

  // new inscription
  if infos.iter().all(|(offset, _)| *offset != 0) && Inscription::from_transaction(&tx).is_some() {
    let prevtransaction = index
      .get_transaction(tx.input.get(0).unwrap().previous_output.txid)?
      .ok_or(anyhow!(format!(
        "failed to get pervious transaction {}",
        tx.input.get(0).unwrap().previous_output.txid
      )))?;

    let id = InscriptionId::from(tx.txid());
    infos.insert(
      0,
      (
        0,
        InscriptionInfo {
          action: ActionType::Inscribe,
          inscription_number: index.get_inscription_entry(id)?.map(|v| v.number),
          inscription_id: id.to_string(),
          from: ScriptPubkey::from_script(
            &prevtransaction.output.get(0).unwrap().script_pubkey,
            index.get_chain_network(),
          ),
          to: ScriptPubkey::default(),
          old_satpoint: SatPoint {
            outpoint: tx.input.get(0).unwrap().previous_output,
            offset: 0,
          }
          .to_string(),
          new_satpoint: None,
          operation: None,
        },
      ),
    );
  }

  // fill new_satpoint and to field
  let mut peeker = infos.into_iter().peekable();
  let mut infos = Vec::new();
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
      infos.push(peeker.next().unwrap().1.clone());
    }
    output_value = end;
  }

  // fill 'to' field with 'from' if the inscription is transferd to coinbase.
  while let Some((_, op)) = peeker.peek_mut() {
    op.to = op.from.clone();
    infos.push(peeker.next().unwrap().1.clone());
  }
  Ok(infos)
}

pub(super) fn get_inscription_by_txid(
  Extension(index): Extension<Arc<Index>>,
  txid: &bitcoin::Txid,
) -> Result<TxInscriptionInfo> {
  // TODO: search in database.
  let tx = index
    .get_transaction_info(txid)?
    .ok_or(anyhow!("can't get transaction info: {txid}"))?;

  // if the transaction is not confirmed, try to parsing protocol
  let infos = if let None = tx.blockhash {
    get_unconfirmed_inscriptions(Extension(index), &tx.transaction()?)
  } else {
    return Err(anyhow!("can't get inscription by txid: {txid}"));
  };
  Ok(TxInscriptionInfo {
    txid: txid.to_string(),
    blockhash: tx.blockhash.map(|v| v.to_string()),
    confirmed: tx.blockhash.is_some(),
    inscriptions: infos?,
  })
}
