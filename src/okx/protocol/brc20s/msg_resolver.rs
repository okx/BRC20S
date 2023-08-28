use super::*;
use crate::{
  inscription::Inscription,
  okx::{
    datastore::{
      brc20s::DataStoreReadOnly,
      ord::{Action, InscriptionOp, OrdDataStoreReadOnly},
    },
    protocol::brc20s::{deserialize_brc20s_operation, operation::Transfer},
  },
  Index, Result,
};
use anyhow::anyhow;
use bitcoin::{OutPoint, TxOut};
use bitcoincore_rpc::Client;
use std::collections::HashMap;

impl Message {
  pub(crate) fn resolve<'a, O: OrdDataStoreReadOnly, M: DataStoreReadOnly>(
    client: &Client,
    ord_store: &'a O,
    brc30_store: &'a M,
    new_inscriptions: &[Inscription],
    op: &InscriptionOp,
    outpoint_to_txout_cache: &mut HashMap<OutPoint, TxOut>,
  ) -> Result<Option<Self>> {
    log::debug!("BRC20S resolving the message from {:?}", op);
    let sat_in_outputs = op
      .new_satpoint
      .map(|satpoint| satpoint.outpoint.txid == op.txid)
      .unwrap_or(false);
    let brc20s_operation = match op.action {
      // New inscription is not `cursed` or `unbound`.
      Action::New {
        cursed: false,
        unbound: false,
      } if sat_in_outputs => {
        match deserialize_brc20s_operation(
          new_inscriptions
            .get(usize::try_from(op.inscription_id.index).unwrap())
            .unwrap(),
          &op.action,
        ) {
          Ok(brc20s_operation) => brc20s_operation,
          _ => return Ok(None),
        }
      }
      // Transfered inscription operation.
      // Attempt to retrieve the `InscribeTransfer` Inscription information from the data store of BRC20S.
      Action::Transfer => match brc30_store.get_inscribe_transfer_inscription(op.inscription_id) {
        // Ignore non-first transfer operations.
        Ok(Some(transfer_info)) if op.inscription_id.txid == op.old_satpoint.outpoint.txid => {
          Operation::Transfer(Transfer {
            tick_id: transfer_info.tick_id.hex(),
            tick: transfer_info.tick_name.as_str().to_string(),
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
    Ok(Some(Self {
      txid: op.txid,
      inscription_id: op.inscription_id,
      old_satpoint: op.old_satpoint,
      new_satpoint: op.new_satpoint,
      commit_input_satpoint: match op.action {
        Action::New { .. } => Some(get_commit_input_satpoint(
          client,
          ord_store,
          op.old_satpoint,
          outpoint_to_txout_cache,
        )?),
        Action::Transfer => None,
      },
      op: brc20s_operation,
      sat_in_outputs,
    }))
  }
}

fn get_commit_input_satpoint<O: OrdDataStoreReadOnly>(
  client: &Client,
  ord_store: &O,
  satpoint: SatPoint,
  outpoint_to_txout_cache: &mut HashMap<OutPoint, TxOut>,
) -> Result<SatPoint> {
  let commit_transaction =
    &Index::get_transaction_retries(client, satpoint.outpoint.txid)?.ok_or(anyhow!(
      "failed to BRC20S message commit transaction! error: {} not found",
      satpoint.outpoint.txid
    ))?;

  // get satoshi offset
  let mut offset = 0;
  for (vout, output) in commit_transaction.output.iter().enumerate() {
    if vout < usize::try_from(satpoint.outpoint.vout).unwrap() {
      offset += output.value;
      continue;
    }
    offset += satpoint.offset;
    break;
  }

  let mut input_value = 0;
  for input in &commit_transaction.input {
    let value = if let Some(tx_out) = ord_store
      .get_outpoint_to_txout(input.previous_output)
      .map_err(|e| anyhow!("failed to get tx out from state! error: {e}"))?
    {
      tx_out.value
    } else if let Some(tx_out) = Index::get_transaction_retries(client, input.previous_output.txid)?
      .map(|tx| {
        tx.output
          .get(usize::try_from(input.previous_output.vout).unwrap())
          .unwrap()
          .clone()
      })
    {
      outpoint_to_txout_cache.insert(input.previous_output, tx_out.clone());
      tx_out.value
    } else {
      return Err(anyhow!(
        "failed to get tx out! error: {} not found",
        input.previous_output
      ));
    };

    input_value += value;
    if input_value >= offset {
      return Ok(SatPoint {
        outpoint: input.previous_output,
        offset: value - input_value + offset,
      });
    }
  }
  Err(anyhow!("no match found for the commit offset!"))
}
#[cfg(test)]
mod tests {
  use super::*;
  use crate::okx::datastore::{
    brc20s::{redb::DataStore, DataStoreReadWrite, Tick, TickId, TransferInfo},
    ord::OrdDbReadWriter,
  };
  use bitcoin::OutPoint;
  use bitcoincore_rpc::{Auth, Client};
  use redb::Database;
  use std::{str::FromStr, vec};
  use tempfile::NamedTempFile;
  fn create_inscription(str: &str) -> Inscription {
    Inscription::new(
      Some("text/plain;charset=utf-8".as_bytes().to_vec()),
      Some(str.as_bytes().to_vec()),
    )
  }

  fn create_inscribe_operation(str: &str) -> (Vec<Inscription>, InscriptionOp) {
    let inscriptions = vec![create_inscription(str)];
    let txid =
      Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();
    let op = InscriptionOp {
      txid,
      action: Action::New {
        cursed: false,
        unbound: false,
      },
      inscription_number: Some(1),
      inscription_id: txid.into(),
      old_satpoint: SatPoint {
        outpoint: OutPoint {
          txid: Txid::from_str("2111111111111111111111111111111111111111111111111111111111111111")
            .unwrap(),
          vout: 0,
        },
        offset: 0,
      },
      new_satpoint: Some(SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      }),
    };
    (inscriptions, op)
  }

  fn create_transfer_operation() -> InscriptionOp {
    let txid =
      Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();

    let inscription_id =
      Txid::from_str("2111111111111111111111111111111111111111111111111111111111111111")
        .unwrap()
        .into();

    InscriptionOp {
      txid,
      action: Action::Transfer,
      inscription_number: Some(1),
      inscription_id,
      old_satpoint: SatPoint {
        outpoint: OutPoint {
          txid: inscription_id.txid,
          vout: 0,
        },
        offset: 0,
      },
      new_satpoint: Some(SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      }),
    }
  }

  #[test]
  fn test_invalid_protocol() {
    let client = Client::new("http://localhost/", Auth::None).unwrap();
    let db_file = NamedTempFile::new().unwrap();
    let db = Database::create(db_file.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let ord_store = OrdDbReadWriter::new(&wtx);
    let brc30_store = DataStore::new(&wtx);

    let mut outpoint_to_txout_cache = HashMap::new();

    let (inscriptions, op) = create_inscribe_operation(
      r#"{"p":"brc30","op":"deploy","t":"pool","pid":"a3668daeaa#1f","stake":"btc","earn":"ordi","erate":"10","dmax":"12000000","dec":"18","total":"21000000","only":"1"}"#,
    );
    assert_matches!(
      Message::resolve(
        &client,
        &ord_store,
        &brc30_store,
        &inscriptions,
        &op,
        &mut outpoint_to_txout_cache,
      ),
      Ok(None)
    );
  }

  #[test]
  fn test_cursed_or_unbound_inscription() {
    let client = Client::new("http://localhost/", Auth::None).unwrap();
    let db_file = NamedTempFile::new().unwrap();
    let db = Database::create(db_file.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let ord_store = OrdDbReadWriter::new(&wtx);
    let brc30_store = DataStore::new(&wtx);

    let mut outpoint_to_txout_cache = HashMap::new();

    let (inscriptions, op) = create_inscribe_operation(
      r#"{"p":"brc20-s","op":"deploy","t":"pool","pid":"a3668daeaa#1f","stake":"btc","earn":"ordi","erate":"10","dmax":"12000000","dec":"18","total":"21000000","only":"1"}"#,
    );
    let op = InscriptionOp {
      action: Action::New {
        cursed: true,
        unbound: false,
      },
      ..op
    };
    assert_matches!(
      Message::resolve(
        &client,
        &ord_store,
        &brc30_store,
        &inscriptions,
        &op,
        &mut outpoint_to_txout_cache,
      ),
      Ok(None)
    );

    let op2 = InscriptionOp {
      action: Action::New {
        cursed: false,
        unbound: true,
      },
      ..op.clone()
    };
    assert_matches!(
      Message::resolve(
        &client,
        &ord_store,
        &brc30_store,
        &inscriptions,
        &op2,
        &mut outpoint_to_txout_cache,
      ),
      Ok(None)
    );
    let op3 = InscriptionOp {
      action: Action::New {
        cursed: true,
        unbound: true,
      },
      ..op.clone()
    };
    assert_matches!(
      Message::resolve(
        &client,
        &ord_store,
        &brc30_store,
        &inscriptions,
        &op3,
        &mut outpoint_to_txout_cache,
      ),
      Ok(None)
    );
  }

  #[test]
  fn test_invalid_transfer() {
    let client = Client::new("http://localhost/", Auth::None).unwrap();
    let db_file = NamedTempFile::new().unwrap();
    let db = Database::create(db_file.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let ord_store = OrdDbReadWriter::new(&wtx);
    let brc30_store = DataStore::new(&wtx);

    let mut outpoint_to_txout_cache = HashMap::new();
    // inscribe transfer not found
    let op = create_transfer_operation();
    assert_matches!(
      Message::resolve(
        &client,
        &ord_store,
        &brc30_store,
        &[],
        &op,
        &mut outpoint_to_txout_cache,
      ),
      Ok(None)
    );

    // non-first transfer operations.
    let op1 = InscriptionOp {
      old_satpoint: SatPoint {
        outpoint: OutPoint {
          txid: Txid::from_str("3111111111111111111111111111111111111111111111111111111111111111")
            .unwrap(),
          vout: 0,
        },
        offset: 0,
      },
      ..op.clone()
    };
    assert_matches!(
      Message::resolve(
        &client,
        &ord_store,
        &brc30_store,
        &[],
        &op1,
        &mut outpoint_to_txout_cache,
      ),
      Ok(None)
    );
  }

  #[test]
  fn test_valid_transfer() {
    let client = Client::new("http://localhost/", Auth::None).unwrap();
    let db_file = NamedTempFile::new().unwrap();
    let db = Database::create(db_file.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let ord_store = OrdDbReadWriter::new(&wtx);
    let brc30_store = DataStore::new(&wtx);

    let mut outpoint_to_txout_cache = HashMap::new();

    // inscribe transfer not found
    let op = create_transfer_operation();

    brc30_store
      .insert_inscribe_transfer_inscription(
        op.inscription_id,
        TransferInfo {
          tick_id: TickId::from_str("a3668daeaa").unwrap(),
          tick_name: Tick::from_str("ordi").unwrap(),
          amt: 100,
        },
      )
      .unwrap();
    let _msg = Message {
      txid: op.txid,
      inscription_id: op.inscription_id,
      old_satpoint: op.old_satpoint,
      new_satpoint: op.new_satpoint,
      commit_input_satpoint: None,
      op: Operation::Transfer(Transfer {
        tick_id: "a3668daeaa".to_string(),
        tick: "ordi".to_string(),
        amount: "100".to_string(),
      }),
      sat_in_outputs: true,
    };
    assert_matches!(
      Message::resolve(
        &client,
        &ord_store,
        &brc30_store,
        &[],
        &op,
        &mut outpoint_to_txout_cache,
      ),
      Ok(Some(_msg))
    );
  }
}
