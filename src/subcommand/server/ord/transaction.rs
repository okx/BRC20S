use {
  super::{error::ApiError, types::ScriptPubkey, *},
  crate::okx::datastore::{
    ord::{Action, InscriptionOp},
    ScriptKey,
  },
  axum::Json,
  utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Deserialize, ToSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InscriptionAction {
  New { cursed: bool, unbound: bool },
  Transfer,
}

impl From<Action> for InscriptionAction {
  fn from(action: Action) -> Self {
    match action {
      Action::New { cursed, unbound } => InscriptionAction::New { cursed, unbound },
      Action::Transfer => InscriptionAction::Transfer,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize, ToSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TxInscription {
  pub action: InscriptionAction,
  pub inscription_number: Option<i64>,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub from: ScriptPubkey,
  pub to: Option<ScriptPubkey>,
}

impl TxInscription {
  pub(super) fn new(op: InscriptionOp, index: Arc<Index>) -> Result<Self> {
    let from = index
      .get_outpoint_entry(op.old_satpoint.outpoint)?
      .map(|txout| ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()))
      .ok_or(anyhow!(
        "outpoint {} not found from database",
        op.old_satpoint.outpoint
      ))?
      .into();
    let to = match op.new_satpoint {
      Some(new_satpoint) => {
        if new_satpoint.outpoint == unbound_outpoint() {
          None
        } else {
          Some(
            index
              .get_outpoint_entry(new_satpoint.outpoint)?
              .map(|txout| ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()))
              .ok_or(anyhow!(
                "outpoint {} not found from database",
                new_satpoint.outpoint
              ))?
              .into(),
          )
        }
      }
      None => None,
    };
    Ok(TxInscription {
      from,
      to,
      action: op.action.into(),
      inscription_number: op.inscription_number,
      inscription_id: op.inscription_id,
      old_satpoint: op.old_satpoint,
      new_satpoint: op.new_satpoint,
    })
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TxInscriptions {
  pub inscriptions: Vec<TxInscription>,
  pub txid: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockInscriptions {
  pub block: Vec<TxInscriptions>,
}

// ord/tx/:txid/inscriptions
pub(crate) async fn ord_txid_inscriptions(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<TxInscriptions> {
  log::debug!("rpc: get ord_txid_inscriptions: {}", txid);
  let txid = Txid::from_str(&txid).unwrap();

  let ops = index
    .ord_txid_inscriptions(&txid)?
    .ok_or_api_not_found(OrdError::OperationNotFound)?;

  log::debug!("rpc: get ord_txid_inscriptions: {:?}", ops);

  let mut api_tx_inscriptions = Vec::new();
  for op in ops.into_iter() {
    match TxInscription::new(op, index.clone()) {
      Ok(tx_inscription) => {
        api_tx_inscriptions.push(tx_inscription);
      }
      Err(error) => {
        return Err(ApiError::internal(format!(
          "Failed to get transaction inscriptions for {txid}, error: {error}"
        )));
      }
    }
  }

  Ok(Json(ApiResponse::ok(TxInscriptions {
    inscriptions: api_tx_inscriptions,
    txid: txid.to_string(),
  })))
}

// ord/block/:blockhash/inscriptions
pub(crate) async fn ord_block_inscriptions(
  Extension(index): Extension<Arc<Index>>,
  Path(block_hash): Path<String>,
) -> ApiResult<BlockInscriptions> {
  log::debug!("rpc: get ord_block_inscriptions: {}", block_hash);

  let block_hash = bitcoin::BlockHash::from_str(&block_hash).map_err(ApiError::bad_request)?;
  let block_inscriptions = index
    .ord_block_inscriptions(&block_hash)?
    .ok_or_api_not_found(OrdError::BlockNotFound)?;

  log::debug!("rpc: get ord_block_inscriptions: {:?}", block_inscriptions);

  let mut api_block_inscriptions = Vec::new();
  for (txid, ops) in block_inscriptions {
    let mut api_tx_inscriptions = Vec::new();
    for op in ops.into_iter() {
      match TxInscription::new(op, index.clone()) {
        Ok(tx_inscription) => {
          api_tx_inscriptions.push(tx_inscription);
        }
        Err(error) => {
          return Err(ApiError::internal(format!(
            "Failed to get transaction inscriptions for {txid}, error: {error}"
          )));
        }
      }
    }
    api_block_inscriptions.push(TxInscriptions {
      inscriptions: api_tx_inscriptions,
      txid: txid.to_string(),
    });
  }

  Ok(Json(ApiResponse::ok(BlockInscriptions {
    block: api_block_inscriptions,
  })))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{txid, InscriptionId, SatPoint};
  use std::str::FromStr;

  #[test]
  fn serialize_ord_inscriptions() {
    let mut tx_inscription = TxInscription {
      from: ScriptKey::from_script(
        &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .assume_checked()
          .script_pubkey(),
        Network::Bitcoin,
      )
      .into(),
      to: Some(
        ScriptKey::from_script(
          &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
            .unwrap()
            .assume_checked()
            .script_pubkey(),
          Network::Bitcoin,
        )
        .into(),
      ),
      action: InscriptionAction::New {
        cursed: false,
        unbound: false,
      },
      inscription_number: Some(100),
      inscription_id: InscriptionId {
        txid: txid(1),
        index: 0xFFFFFFFF,
      },
      old_satpoint: SatPoint::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
      )
      .unwrap(),

      new_satpoint: Some(
        SatPoint::from_str(
          "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
        )
        .unwrap(),
      ),
    };
    assert_eq!(
      serde_json::to_string_pretty(&tx_inscription).unwrap(),
      r###"{
  "action": {
    "new": {
      "cursed": false,
      "unbound": false
    }
  },
  "inscriptionNumber": 100,
  "inscriptionId": "1111111111111111111111111111111111111111111111111111111111111111i4294967295",
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  }
}"###,
    );
    tx_inscription.action = InscriptionAction::Transfer;
    assert_eq!(
      serde_json::to_string_pretty(&tx_inscription).unwrap(),
      r###"{
  "action": "transfer",
  "inscriptionNumber": 100,
  "inscriptionId": "1111111111111111111111111111111111111111111111111111111111111111i4294967295",
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  }
}"###,
    );
  }
}
