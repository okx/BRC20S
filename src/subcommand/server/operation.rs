use super::*;
use crate::brc20::{
  deserialize_brc20_operation, Action, Deploy as InsDeploy, Mint as InsMint, Operation,
  Transfer as InsTransfer,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScriptPubkey {
  Address(String),
  NonStandard(String),
}
impl Default for ScriptPubkey {
  fn default() -> Self {
    ScriptPubkey::NonStandard(String::new())
  }
}

impl ScriptPubkey {
  pub fn from_script(script: &Script, network: Network) -> Self {
    match Address::from_script(script, network) {
      Ok(address) => ScriptPubkey::Address(address.to_string()),
      Err(_) => ScriptPubkey::NonStandard(script.script_hash().to_string()),
    }
  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brc20Transaction {
  pub txid: String,
  pub isconfirmed: bool,
  pub operations: Vec<Brc20Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brc20Operation {
  // if the transaction not committed to the blockchain, the following fields are None
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscription_number: Option<u64>,
  pub inscription_id: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub old_satpoint: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  // if transfer to coinbase new_satpoint is None
  pub new_satpoint: Option<String>,
  pub operation: RawOperation,
}
pub(super) fn get_brc20_operations(
  Extension(index): Extension<Arc<Index>>,
  tx: &Transaction,
) -> Result<Vec<Brc20Operation>> {
  let mut operations: Vec<(u64, Brc20Operation)> = Vec::new();
  let mut input_value = 0;
  let mut script_cache = Script::default();

  for tx_in in &tx.input {
    // skipped coinbase transaction
    if tx_in.previous_output.is_null() {
      return Ok(Vec::new());
    }

    // requset previous transaction
    let prev_tx = index
      .get_transaction(tx_in.previous_output.txid)?
      .ok_or(anyhow!(format!(
        "failed to get pervious transaction {}",
        tx_in.previous_output.txid
      )))?;

    if tx_in == tx.input.get(0).unwrap() {
      script_cache = prev_tx.output.get(0).unwrap().script_pubkey.clone();
    }

    // collect the transfer operation if the previous is a inscribed transfer operation.
    if let Some(Operation::Transfer(transfer)) = Inscription::from_transaction(&prev_tx)
      .and_then(|v| deserialize_brc20_operation(v, true).ok())
    {
      let id = InscriptionId::from(tx_in.previous_output.txid);
      operations.push((
        input_value,
        Brc20Operation {
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
          operation: RawOperation::Transfer(transfer.into()),
        },
      ))
    }
    input_value += prev_tx
      .output
      .get(tx_in.previous_output.vout as usize)
      .unwrap()
      .value;
  }

  // new inscription
  if operations
    .iter()
    .all(|(offset, op)| op.new_satpoint.as_ref().map_or(true, |v| *offset != 0))
    && input_value > 0
  {
    if let Some(op) =
      Inscription::from_transaction(&tx).and_then(|v| deserialize_brc20_operation(v, false).ok())
    {
      let id = InscriptionId::from(tx.txid());
      operations.insert(
        0,
        (
          0,
          Brc20Operation {
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
            operation: match op {
              Operation::Deploy(deploy) => RawOperation::Deploy(deploy.into()),
              Operation::Mint(mint) => RawOperation::Mint(mint.into()),
              Operation::Transfer(transfer) => RawOperation::InscribeTransfer(transfer.into()),
            },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum RawOperation {
  Deploy(Deploy),
  Mint(Mint),
  InscribeTransfer(Transfer),
  Transfer(Transfer),
}

// action to raw operation
impl From<Action> for RawOperation {
  fn from(action: Action) -> Self {
    match action {
      Action::Inscribe(op) => match op {
        Operation::Deploy(deploy) => RawOperation::Deploy(deploy.into()),
        Operation::Mint(mint) => RawOperation::Mint(mint.into()),
        Operation::Transfer(transfer) => RawOperation::InscribeTransfer(transfer.into()),
      },
      Action::Transfer(transfer) => RawOperation::Transfer(transfer.into()),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deploy {
  pub tick: String,
  pub max: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lim: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dec: Option<String>,
}

impl From<InsDeploy> for Deploy {
  fn from(deploy: InsDeploy) -> Self {
    Deploy {
      tick: deploy.tick,
      max: deploy.max_supply,
      lim: deploy.mint_limit,
      dec: deploy.decimals,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mint {
  pub tick: String,
  pub amt: String,
}

impl From<InsMint> for Mint {
  fn from(mint: InsMint) -> Self {
    Mint {
      tick: mint.tick,
      amt: mint.amount,
    }
  }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
  pub tick: String,
  pub amt: String,
}

impl From<InsTransfer> for Transfer {
  fn from(transfer: InsTransfer) -> Self {
    Transfer {
      tick: transfer.tick,
      amt: transfer.amount,
    }
  }
}
#[cfg(test)]
mod tests {
  use bitcoin::hashes::hex::FromHex;

  use super::*;
  #[test]
  fn serialize_script_pubkey() {
    let script_pubkey = ScriptPubkey::from_script(
      &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
        .unwrap()
        .script_pubkey(),
      Network::Bitcoin,
    );
    assert_eq!(
      serde_json::to_string(&script_pubkey).unwrap(),
      r#"{"address":"bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"}"#
    );
    let script_pubkey = ScriptPubkey::from_script(
      &Script::from_hex(
        "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
      )
      .unwrap(),
      Network::Bitcoin,
    );

    assert_eq!(
      serde_json::to_string(&script_pubkey).unwrap(),
      r#"{"nonStandard":"df65c8a338dce7900824e7bd18c336656ca19e57"}"#
    );
  }

  #[test]
  fn serialize_deploy() {
    let deploy = Deploy {
      tick: "ordi".to_string(),
      max: "1000".to_string(),
      lim: Some("1000".to_string()),
      dec: Some("18".to_string()),
    };
    assert_eq!(
      serde_json::to_string(&deploy).unwrap(),
      r#"{"tick":"ordi","max":"1000","lim":"1000","dec":"18"}"#
    );
    let deploy = Deploy {
      tick: "ordi".to_string(),
      max: "1000".to_string(),
      lim: None,
      dec: None,
    };
    assert_eq!(
      serde_json::to_string(&deploy).unwrap(),
      r#"{"tick":"ordi","max":"1000"}"#
    );
  }

  #[test]
  fn serialize_mint() {
    let mint = Mint {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    };
    assert_eq!(
      serde_json::to_string(&mint).unwrap(),
      r#"{"tick":"ordi","amt":"1000"}"#
    );
  }

  #[test]
  fn serialize_transfer() {
    let transfer = Transfer {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    };
    assert_eq!(
      serde_json::to_string(&transfer).unwrap(),
      r#"{"tick":"ordi","amt":"1000"}"#
    );
  }

  #[test]
  fn serialize_raw_operation() {
    let deploy = RawOperation::Deploy(Deploy {
      tick: "ordi".to_string(),
      max: "1000".to_string(),
      lim: Some("1000".to_string()),
      dec: Some("18".to_string()),
    });
    assert_eq!(
      serde_json::to_string(&deploy).unwrap(),
      r#"{"type":"deploy","tick":"ordi","max":"1000","lim":"1000","dec":"18"}"#
    );
    let mint = RawOperation::Mint(Mint {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    });
    assert_eq!(
      serde_json::to_string(&mint).unwrap(),
      r#"{"type":"mint","tick":"ordi","amt":"1000"}"#
    );
    let inscribe_transfer = RawOperation::InscribeTransfer(Transfer {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    });
    assert_eq!(
      serde_json::to_string(&inscribe_transfer).unwrap(),
      r#"{"type":"inscribeTransfer","tick":"ordi","amt":"1000"}"#
    );
    let transfer = RawOperation::Transfer(Transfer {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    });
    assert_eq!(
      serde_json::to_string(&transfer).unwrap(),
      r#"{"type":"transfer","tick":"ordi","amt":"1000"}"#
    );
  }

  #[test]
  fn serialize_transaction() {
    let operation = Brc20Operation {
      from: ScriptPubkey::from_script(
        &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .script_pubkey(),
        Network::Bitcoin,
      ),
      to: ScriptPubkey::from_script(
        &Script::from_hex(
          "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
        )
        .unwrap(),
        Network::Bitcoin,
      ),
      old_satpoint: SatPoint::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
      )
      .unwrap()
      .to_string(),
      new_satpoint: None,
      inscription_id: InscriptionId::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
      )
      .unwrap()
      .to_string(),
      inscription_number: None,
      operation: RawOperation::Deploy(Deploy {
        tick: "ordi".to_string(),
        max: "1000".to_string(),
        lim: Some("1000".to_string()),
        dec: Some("18".to_string()),
      }),
    };
    assert_eq!(
      serde_json::to_string_pretty(&operation).unwrap(),
      r#"{
  "inscriptionId": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "nonStandard": "df65c8a338dce7900824e7bd18c336656ca19e57"
  },
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "operation": {
    "type": "deploy",
    "tick": "ordi",
    "max": "1000",
    "lim": "1000",
    "dec": "18"
  }
}"#
    );

    let operation = Brc20Operation {
      from: ScriptPubkey::from_script(
        &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .script_pubkey(),
        Network::Bitcoin,
      ),
      to: ScriptPubkey::from_script(
        &Script::from_hex(
          "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
        )
        .unwrap(),
        Network::Bitcoin,
      ),
      old_satpoint: SatPoint::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
      )
      .unwrap()
      .to_string(),
      new_satpoint: Some(
        SatPoint::from_str(
          "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
        )
        .unwrap()
        .to_string(),
      ),
      inscription_number: Some(100),
      inscription_id: InscriptionId::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
      )
      .unwrap()
      .to_string(),
      operation: RawOperation::Mint(Mint {
        tick: "ordi".to_string(),
        amt: "1000".to_string(),
      }),
    };
    assert_eq!(
      serde_json::to_string_pretty(&operation).unwrap(),
      r#"{
  "inscriptionNumber": 100,
  "inscriptionId": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "nonStandard": "df65c8a338dce7900824e7bd18c336656ca19e57"
  },
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "operation": {
    "type": "mint",
    "tick": "ordi",
    "amt": "1000"
  }
}"#
    )
  }
}
