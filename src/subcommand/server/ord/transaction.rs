use serde_json::Value;
use {
  super::{error::ApiError, types::ScriptPubkey, *},
  crate::okx::datastore::{
    ord::{Action, InscriptionOp},
    ScriptKey,
  },
  axum::Json,
  utoipa::ToSchema,
};
use crate::okx::protocol::brc0::{BRCZeroTx, JSONError, RpcParams};
use crate::okx::protocol::message::MsgInscription;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = ord::InscriptionAction)]
#[serde(rename_all = "camelCase")]
pub enum InscriptionAction {
  /// New inscription
  New { cursed: bool, unbound: bool },
  /// Transfer inscription
  Transfer,
}

impl From<Action> for InscriptionAction {
  fn from(action: Action) -> Self {
    match action {
      Action::New {
        cursed, unbound, ..
      } => InscriptionAction::New { cursed, unbound },
      Action::Transfer => InscriptionAction::Transfer,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = ord::TxInscription)]
#[serde(rename_all = "camelCase")]
pub struct TxInscription {
  /// The action of the inscription.
  #[schema(value_type = ord::InscriptionAction)]
  pub action: InscriptionAction,
  /// The inscription number.
  pub inscription_number: Option<i64>,
  /// The inscription id.
  pub inscription_id: String,
  /// The inscription satpoint of the transaction input.
  pub old_satpoint: String,
  /// The inscription satpoint of the transaction output.
  pub new_satpoint: Option<String>,
  /// The message sender which is an address or script pubkey hash.
  pub from: ScriptPubkey,
  /// The message receiver which is an address or script pubkey hash.
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
      inscription_id: op.inscription_id.to_string(),
      old_satpoint: op.old_satpoint.to_string(),
      new_satpoint: op.new_satpoint.map(|v| v.to_string()),
    })
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = ord::TxInscriptions)]
#[serde(rename_all = "camelCase")]
pub struct TxInscriptions {
  #[schema(value_type = Vec<ord::TxInscription>)]
  pub inscriptions: Vec<TxInscription>,
  pub txid: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = ord::BlockInscriptions)]
#[serde(rename_all = "camelCase")]
pub struct BlockInscriptions {
  #[schema(value_type = Vec<ord::TxInscriptions>)]
  pub block: Vec<TxInscriptions>,
}

// ord/tx/:txid/inscriptions
/// Retrieve the inscription actions from the given transaction.
#[utoipa::path(
  get,
  path = "/api/v1/ord/tx/{txid}/inscriptions",
  params(
      ("txid" = String, Path, description = "transaction ID")
),
  responses(
    (status = 200, description = "Obtain inscription actions by txid", body = OrdTxInscriptions),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
pub(crate) async fn ord_txid_inscriptions(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<TxInscriptions> {
  log::debug!("rpc: get ord_txid_inscriptions: {}", txid);
  let txid = Txid::from_str(&txid).map_err(ApiError::bad_request)?;

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
/// Retrieve the inscription actions from the given block.
#[utoipa::path(
  get,
  path = "/api/v1/ord/block/{blockhash}/inscriptions",
  params(
      ("blockhash" = String, Path, description = "block hash")
),
  responses(
    (status = 200, description = "Obtain inscription actions by blockhash", body = OrdBlockInscriptions),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
pub(crate) async fn ord_block_inscriptions(
  Extension(index): Extension<Arc<Index>>,
  Path(blockhash): Path<String>,
) -> ApiResult<BlockInscriptions> {
  log::debug!("rpc: get ord_block_inscriptions: {}", blockhash);

  let blockhash = bitcoin::BlockHash::from_str(&blockhash).map_err(ApiError::bad_request)?;
  // get block from btc client.
  let blockinfo = index
    .get_block_info_by_hash(blockhash)
    .map_err(ApiError::internal)?
    .ok_or_api_not_found(OrdError::BlockNotFound)?;

  // get blockhash from redb.
  let blockhash = index
    .block_hash(Some(u64::try_from(blockinfo.height).unwrap()))
    .map_err(ApiError::internal)?
    .ok_or_api_not_found(OrdError::BlockNotFound)?;

  // check of conflicting block.
  if blockinfo.hash != blockhash {
    return Err(ApiError::NotFound(OrdError::BlockNotFound.to_string()));
  }

  let block_inscriptions = index
    .ord_get_txs_inscriptions(&blockinfo.tx)
    .map_err(ApiError::internal)?;

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

// ord/block/:blockhash/inscriptions
/// Retrieve the inscription actions from the given block.
#[utoipa::path(
get,
path = "/api/v1/brc0/rpc_request/:height",
params(
("height" = u64, Path, description = "block height")
),
responses(
(status = 200, description = "Obtain inscription actions by blockhash", body = OrdBlockInscriptions),
(status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
(status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
(status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
)
)]
pub(crate) async fn brc0_rpcrequest(
  Extension(index): Extension<Arc<Index>>,
  Path(height): Path<u64>,
) -> ApiResult<RpcParams> {
  log::debug!("rpc: brc0_rpcrequest: {}", height);

  let params = index.ord_brc0_rpcrequest(height)?;

  Ok(Json(ApiResponse::ok(params)))
}

#[derive(Debug, PartialEq, Clone,Deserialize, Serialize)]
pub struct ZeroData {
  pub block_height: u64,
  pub block_hash: String,
  pub prev_block_hash: String,
  pub block_time: u32,
  pub txs: Vec<ZeroIndexerTx>,
}
#[derive(Debug, PartialEq, Clone,Serialize,Deserialize)]
pub struct ZeroIndexerTx {
  pub protocol_name: String,
  pub inscription: String,
  pub inscription_context: String,
  pub btc_txid: String,
  pub btc_fee: String,
}

fn convert_to_zerodata(params: &RpcParams) -> Option<ZeroData> {
  let mut txs: Vec<ZeroIndexerTx> = Vec::new();
  for brc0_tx in params.txs.iter() {
    let tx = convert_to_zerotx(&brc0_tx);
    match tx {
      None => {}
      Some(tx) => {
        txs.push(tx);
      }
    }
  }


  match params.height.parse::<u64>() {
    Ok(num) => {
      Some(ZeroData {
        block_height: num,
        block_hash: params.block_hash.clone(),
        prev_block_hash: "".to_string(),
        block_time: 0,
        txs,
      })
    }
    Err(err) => {
      None
    }
  }
}

fn convert_to_zerotx(tx:&BRCZeroTx) -> Option<ZeroIndexerTx> {
  let msg = match deserialize_msg_inscription(tx.hex_rlp_encode_tx.as_str()) {
    Ok(msg) => {msg}
    Err(_) => {return None;}
  };
  let protocol_name = match get_protocol_name(msg.inscription.as_str()) {
    Ok(name) => {name}
    Err(_) => {return None}
  };
  Some(ZeroIndexerTx{
    protocol_name: protocol_name.to_string(),
    inscription: msg.inscription,
    inscription_context: serde_json::to_string(&msg.inscription_context).unwrap(),
    btc_txid: msg.inscription_context.txid,
    btc_fee: tx.btc_fee.to_string(),
  })
}

fn get_protocol_name(s: &str) -> Result<String, JSONError> {
  let value:Value = serde_json::from_str(s).map_err(|_| JSONError::InvalidJson)?;

  let protocol_name =  match value.get("p") {
    None => {return Err(JSONError::NotBRC0Json)}
    Some(v) => {
      v.to_string().replace("\"", "")
    }
  };
  Ok(protocol_name)
}

fn deserialize_msg_inscription(s: &str) -> Result<MsgInscription, JSONError> {
  let value = serde_json::from_str(s).map_err(|_| JSONError::InvalidJson)?;
  serde_json::from_value(value).map_err(|e| JSONError::ParseOperationJsonError(e.to_string()))
}
// ord/block/:blockhash/inscriptions
/// Retrieve the inscription actions from the given block.
#[utoipa::path(
get,
path = "/api/v1/crawler/zeroindexer/:height",
params(
("height" = u64, Path, description = "block height")
),
responses(
(status = 200, description = "Obtain inscription actions by blockhash", body = OrdBlockInscriptions),
(status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
(status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
(status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
)
)]
pub(crate) async fn crawler_zeroindexer(
  Extension(index): Extension<Arc<Index>>,
  Path(height): Path<u64>,
) -> ApiResult<ZeroData> {
  log::debug!("rpc: brc0_rpcrequest: {}", height);

  let params = index.ord_brc0_rpcrequest(height)?;
  let zero_data = convert_to_zerodata(&params);

  match zero_data {
    None => {Err(ApiError::internal("height parse failed"))}
    Some(zd) => {Ok(Json(ApiResponse::ok(zd)))}
  }
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
      }
      .to_string(),
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
    };
    assert_eq!(
      serde_json::to_string_pretty(&tx_inscription).unwrap(),
      r#"{
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
}"#,
    );
    tx_inscription.action = InscriptionAction::Transfer;
    assert_eq!(
      serde_json::to_string_pretty(&tx_inscription).unwrap(),
      r#"{
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
}"#,
    );
  }
}
