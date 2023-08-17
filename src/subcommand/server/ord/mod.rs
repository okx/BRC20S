use {
  super::*,
  crate::okx::datastore::ord::{Action, InscriptionOp},
};

mod inscription;
mod outpoint;
mod transaction;

pub(super) use {inscription::*, outpoint::*, transaction::*};

#[derive(Debug, thiserror::Error)]
pub enum OrdError {
  #[error("operation not found")]
  OperationNotFound,
  #[error("block not found")]
  BlockNotFound,
}

#[derive(Debug, Clone)]
enum Origin {
  New { cursed: bool, unbound: bool },
  Old,
}

#[derive(Debug, Clone)]
struct Flotsam {
  txid: Txid,
  inscription_id: InscriptionId,
  offset: u64,
  old_satpoint: SatPoint,
  origin: Origin,
}

pub(super) fn get_ord_operations_by_txid(
  index: &Arc<Index>,
  txid: &bitcoin::Txid,
  with_unconfirmed: bool,
) -> Result<Vec<InscriptionOp>> {
  let tx = index
    .get_transaction_info(txid)?
    .ok_or(anyhow!("can't get transaction info: {txid}"))?;

  match tx.confirmations {
    None => {
      if with_unconfirmed {
        // If the transaction is not confirmed, simulate indexing the transaction. Otherwise, retrieve it from the database.
        simulate_index_ord_transaction(index, &tx.transaction()?, tx.txid)
      } else {
        Err(anyhow!("transaction not confirmed: {txid}"))
      }
    }
    // TODO: retrieve it from the database.
    Some(_) => Err(anyhow!("not implemented")),
  }
}

/// Simulate the execution of a transaction and parse out the inscription operation.
fn simulate_index_ord_transaction(
  index: &Arc<Index>,
  tx: &Transaction,
  txid: Txid,
) -> Result<Vec<InscriptionOp>> {
  let mut new_inscriptions = Inscription::from_transaction(tx).into_iter().peekable();
  let mut operations = Vec::new();
  let mut floating_inscriptions = Vec::new();
  let mut inscribed_offsets = BTreeMap::new();
  let mut input_value = 0;
  let mut id_counter = 0;

  for (input_index, tx_in) in tx.input.iter().enumerate() {
    // skip coinbase transaction.
    if tx_in.previous_output.is_null() {
      return Ok(operations);
    }

    // find existing inscriptions on input aka transfers of
    for (old_satpoint, inscription_id) in
      index.get_inscriptions_with_satpoint_on_output(tx_in.previous_output)?
    {
      let offset = input_value + old_satpoint.offset;
      floating_inscriptions.push(Flotsam {
        txid,
        offset,
        old_satpoint,
        inscription_id,
        origin: Origin::Old,
      });

      inscribed_offsets.insert(offset, inscription_id);
    }

    let offset = input_value;

    input_value +=
      if let Some(tx_out) = index.get_transaction_output_by_outpoint(tx_in.previous_output)? {
        tx_out.value
      } else if let Some(tx) = index.get_transaction_with_retries(tx_in.previous_output.txid)? {
        tx.output
          .get(usize::try_from(tx_in.previous_output.vout).unwrap())
          .unwrap()
          .value
      } else {
        return Err(anyhow!(
          "can't get transaction output by outpoint: {}",
          tx_in.previous_output
        ));
      };

    // go through all inscriptions in this input
    while let Some(inscription) = new_inscriptions.peek() {
      if inscription.tx_in_index != u32::try_from(input_index).unwrap() {
        break;
      }

      let initial_inscription_is_cursed = inscribed_offsets
        .get(&offset)
        .and_then(
          |inscription_id| match index.get_inscription_entry(*inscription_id) {
            Ok(option) => option.map(|entry| entry.number < 0),
            Err(_) => None,
          },
        )
        .unwrap_or(false);

      let cursed = !initial_inscription_is_cursed
        && (inscription.tx_in_index != 0
          || inscription.tx_in_offset != 0
          || inscribed_offsets.contains_key(&offset));

      // In this first part of the cursed inscriptions implementation we ignore reinscriptions.
      // This will change once we implement reinscriptions.
      let unbound = inscribed_offsets.contains_key(&offset)
        || inscription.tx_in_offset != 0
        || input_value == 0;

      let inscription_id = InscriptionId {
        txid,
        index: id_counter,
      };

      floating_inscriptions.push(Flotsam {
        txid,
        old_satpoint: SatPoint {
          outpoint: tx_in.previous_output,
          offset: 0,
        },
        inscription_id,
        offset,
        origin: Origin::New { cursed, unbound },
      });

      new_inscriptions.next();
      id_counter += 1;
    }
  }

  floating_inscriptions.sort_by_key(|flotsam| flotsam.offset);
  let mut inscriptions = floating_inscriptions.into_iter().peekable();

  let mut output_value = 0;
  for (vout, tx_out) in tx.output.iter().enumerate() {
    let end = output_value + tx_out.value;

    while let Some(flotsam) = inscriptions.peek() {
      if flotsam.offset >= end {
        break;
      }

      let new_satpoint = SatPoint {
        outpoint: OutPoint {
          txid,
          vout: vout.try_into().unwrap(),
        },
        offset: flotsam.offset - output_value,
      };

      let flotsam = inscriptions.next().unwrap();

      // Find the inscription with the output position and add it to the vector.
      operations.push(InscriptionOp {
        txid: flotsam.txid,
        action: match flotsam.origin {
          Origin::New { cursed, unbound } => Action::New { cursed, unbound },
          Origin::Old => Action::Transfer,
        },
        // Unknown number, replaced with zero.
        inscription_number: None,
        inscription_id: flotsam.inscription_id,
        old_satpoint: flotsam.old_satpoint,
        new_satpoint: Some(new_satpoint),
      });
    }

    output_value = end;
  }

  // Inscription not found with matching output position.
  operations.extend(inscriptions.map(|flotsam| InscriptionOp {
    txid: flotsam.txid,
    action: match flotsam.origin {
      Origin::New { cursed, unbound } => Action::New { cursed, unbound },
      Origin::Old => Action::Transfer,
    },
    inscription_number: None,
    inscription_id: flotsam.inscription_id,
    old_satpoint: flotsam.old_satpoint,
    // We use a zero satpoint to represent the default position.
    new_satpoint: None,
  }));

  Ok(operations)
}
