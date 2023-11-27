use super::{error::ApiError, types::ScriptPubkey, *};
use crate::{
  index::InscriptionEntry,
  okx::datastore::{
    ord::{Action, InscriptionOp},
    ScriptKey,
  },
};
use axum::Json;
use crate::okx::protocol::brc0::RpcParams;

#[derive(Debug, thiserror::Error)]
pub enum OrdError {
  #[error("operation not found")]
  OperationNotFound,
  #[error("block not found")]
  BlockNotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdInscription {
  pub id: InscriptionId,
  pub number: i64,
  pub content_type: Option<String>,
  pub content: Option<String>,
  pub owner: Option<ScriptPubkey>,
  pub genesis_height: u64,
  pub location: SatPoint,
  pub sat: Option<u64>,
}
#[derive(Debug, Clone)]
struct Flotsam {
  txid: Txid,
  inscription_id: InscriptionId,
  offset: u64,
  old_satpoint: SatPoint,
  origin: Origin,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutPointData {
  pub txid: String,
  pub script_pub_key: String,
  pub owner: ScriptPubkey,
  pub value: u64,
  pub inscription_digest: Vec<InscriptionDigest>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscriptionDigest {
  pub id: String,
  pub number: i64,
  pub location: String,
}

#[derive(Debug, Clone)]
enum Origin {
  New { cursed: bool, unbound: bool },
  Old,
}

fn ord_get_inscription_by_id(index: Arc<Index>, id: InscriptionId) -> ApiResult<OrdInscription> {
  let inscription_data = get_inscription_all_data_by_id(index.clone(), id)?
    .ok_or_api_not_found(format!("inscriptionId not found {id}"))?;
  let location_outpoint = inscription_data.sat_point.outpoint;
  let mut owner = None;
  if location_outpoint != unbound_outpoint() {
    owner = if inscription_data.tx.txid() != location_outpoint.txid {
      let location_raw_tx = index
        .get_transaction(location_outpoint.txid)?
        .ok_or_api_not_found(format!(
          "inscriptionId not found {}",
          location_outpoint.txid
        ))?;
      Some(
        ScriptKey::from_script(
          &location_raw_tx
            .output
            .get(usize::try_from(location_outpoint.vout).unwrap())
            .unwrap()
            .script_pubkey,
          index.get_chain_network(),
        )
        .into(),
      )
    } else {
      Some(
        ScriptKey::from_script(
          &inscription_data.tx.output[0].script_pubkey,
          index.get_chain_network(),
        )
        .into(),
      )
    };
  };

  Ok(Json(ApiResponse::ok(OrdInscription {
    id,
    number: inscription_data.entry.number,
    content_type: inscription_data
      .inscription
      .content_type()
      .map(String::from),
    content: inscription_data.inscription.body().map(hex::encode),
    owner,
    genesis_height: inscription_data.entry.height,
    location: inscription_data.sat_point,
    sat: inscription_data.entry.sat.map(|s| s.0),
  })))
}

pub(crate) async fn ord_inscription_id(
  Extension(index): Extension<Arc<Index>>,
  Path(id): Path<String>,
) -> ApiResult<OrdInscription> {
  log::debug!("rpc: get ord_inscription_id: {id}");
  let id = InscriptionId::from_str(&id).map_err(|e| ApiError::bad_request(e.to_string()))?;

  ord_get_inscription_by_id(index, id)
}

pub(crate) async fn ord_inscription_number(
  Extension(index): Extension<Arc<Index>>,
  Path(number): Path<i64>,
) -> ApiResult<OrdInscription> {
  log::debug!("rpc: get ord_inscription_number: {number}");

  let id = index
    .get_inscription_id_by_inscription_number(number)?
    .ok_or_api_not_found(format!("inscriptionId not found for number: {number}"))?;

  ord_get_inscription_by_id(index, id)
}

pub(super) async fn ord_outpoint(
  Extension(index): Extension<Arc<Index>>,
  Path(outpoint): Path<OutPoint>,
) -> ApiResult<OutPointData> {
  log::debug!("rpc: get ord_outpoint: {outpoint}");

  let inscription_ids = index.get_inscriptions_on_output(outpoint)?;
  if inscription_ids.is_empty() {
    return Err(ApiError::not_found("inscriptionIds not found"));
  }

  let tx = index
    .get_transaction(outpoint.txid)?
    .ok_or_api_not_found(format!("transaction not found {}", outpoint.txid))?;

  let vout = tx
    .output
    .get(outpoint.vout as usize)
    .ok_or_api_not_found(format!("vout not found for {outpoint}"))?;

  let mut inscription_digests = Vec::with_capacity(inscription_ids.len());
  for id in &inscription_ids {
    let ins_data = index
      .get_inscription_entry(*id)?
      .ok_or_api_not_found(format!("inscriptionId not found for {id}"))?;

    let satpoint = index
      .get_inscription_satpoint_by_id(*id)?
      .ok_or_api_not_found(format!("satpoint not found for {id}"))?;

    inscription_digests.push(InscriptionDigest {
      id: id.to_string(),
      number: ins_data.number,
      location: satpoint.to_string(),
    });
  }

  Ok(Json(ApiResponse::ok(OutPointData {
    txid: outpoint.txid.to_string(),
    script_pub_key: vout.script_pubkey.to_asm_string(),
    owner: ScriptKey::from_script(&vout.script_pubkey, index.get_chain_network()).into(),
    value: vout.value,
    inscription_digest: inscription_digests,
  })))
}

pub(super) fn get_ord_operations_by_txid(
  index: &Arc<Index>,
  txid: &bitcoin::Txid,
) -> Result<Vec<InscriptionOp>> {
  let tx = index
    .get_transaction_info(txid)?
    .ok_or(anyhow!("can't get transaction info: {txid}"))?;
  // If the transaction is not confirmed, simulate indexing the transaction. Otherwise, retrieve it from the database.
  match tx.confirmations {
    None => simulate_index_ord_transaction(index, &tx.transaction()?, tx.txid),
    // TODO: retrieve it from the database.
    Some(_) => Ok(Vec::new()),
  }
}

// Simulate the execution of a transaction and parse out the inscription operation.
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
    for (old_satpoint, inscription_id) in index
      .get_inscriptions_with_satpoint_on_output(tx_in.previous_output)?
      .into_iter()
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

pub(crate) struct InscriptionAllData {
  pub tx: Transaction,
  pub entry: InscriptionEntry,
  pub sat_point: SatPoint,
  pub inscription: Inscription,
}

pub(crate) fn get_inscription_all_data_by_id(
  index: Arc<Index>,
  inscription_id: InscriptionId,
) -> Result<Option<InscriptionAllData>> {
  let entry = match index.get_inscription_entry(inscription_id)? {
    Some(entry) => entry,
    None => return Ok(None),
  };
  let tx = match index.get_transaction(inscription_id.txid)? {
    Some(tx) => tx,
    None => return Ok(None),
  };
  let inscription =
    match Inscription::from_transaction(&tx).get(usize::try_from(inscription_id.index).unwrap()) {
      Some(transaction_inscription) => transaction_inscription.inscription.clone(),
      None => return Ok(None),
    };

  let sat_point = match index.get_inscription_satpoint_by_id(inscription_id)? {
    Some(sat_point) => sat_point,
    None => return Ok(None),
  };

  Ok(Some(InscriptionAllData {
    entry,
    tx,
    inscription,
    sat_point,
  }))
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TxInscription {
  pub action: InscriptionAction,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscription_number: Option<i64>,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub new_satpoint: Option<SatPoint>,
  pub from: ScriptPubkey,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub to: Option<ScriptPubkey>,
}

impl TxInscription {
  pub(super) fn new(op: InscriptionOp, index: Arc<Index>) -> Result<Self> {
    let from = index
      .get_outpoint_entry(&op.old_satpoint.outpoint)?
      .map(|txout| ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()))
      .ok_or(anyhow!(
        "failed to get tx out! error: outpoint {} not found",
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
              .get_outpoint_entry(&new_satpoint.outpoint)?
              .map(|txout| ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()))
              .ok_or(anyhow!(
                "failed to get tx out! error: outpoint {} not found",
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxInscriptions {
  pub inscriptions: Vec<TxInscription>,
  pub txid: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockInscriptions {
  pub block: Vec<TxInscriptions>,
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
            "failed to get transaction inscriptions for {txid}, error: {error}"
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
          "failed to get transaction inscriptions for {txid}, error: {error}"
        )));
      }
    }
  }

  Ok(Json(ApiResponse::ok(TxInscriptions {
    inscriptions: api_tx_inscriptions,
    txid: txid.to_string(),
  })))
}


// brc0/rpc_request/:height
pub(crate) async fn brc0_rpcrequest(
  Extension(index): Extension<Arc<Index>>,
  Path(height): Path<u64>,
) -> ApiResult<RpcParams> {
  log::debug!("rpc: brc0_rpcrequest: {}", height);

  let params = index.ord_brc0_rpcrequest(height)?;

  Ok(Json(ApiResponse::ok(params)))
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

  #[test]
  fn test_serialize_ord_inscription() {
    let mut ord_inscription = OrdInscription {
      id: InscriptionId {
        txid: txid(1),
        index: 0xFFFFFFFF,
      },
      number: -100,
      content_type: Some("content_type".to_string()),
      content: Some("content".to_string()),
      owner: Some(
        ScriptKey::from_script(
          &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
            .unwrap()
            .assume_checked()
            .script_pubkey(),
          Network::Bitcoin,
        )
        .into(),
      ),
      genesis_height: 1,
      location: SatPoint::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
      )
      .unwrap(),
      sat: None,
    };
    assert_eq!(
      serde_json::to_string_pretty(&ord_inscription).unwrap(),
      r###"{
  "id": "1111111111111111111111111111111111111111111111111111111111111111i4294967295",
  "number": -100,
  "contentType": "content_type",
  "content": "content",
  "owner": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "genesisHeight": 1,
  "location": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "sat": null
}"###,
    );
    ord_inscription.owner = None;
    assert_eq!(
      serde_json::to_string_pretty(&ord_inscription).unwrap(),
      r###"{
  "id": "1111111111111111111111111111111111111111111111111111111111111111i4294967295",
  "number": -100,
  "contentType": "content_type",
  "content": "content",
  "owner": null,
  "genesisHeight": 1,
  "location": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "sat": null
}"###,
    );
  }
}
