use {
  super::{error::ApiError, types::ScriptPubkey, *},
  crate::{index::InscriptionEntry, okx::datastore::ScriptKey},
  axum::Json,
  utoipa::ToSchema,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

struct InscriptionAllData {
  pub tx: Transaction,
  pub entry: InscriptionEntry,
  pub sat_point: SatPoint,
  pub inscription: Inscription,
}

fn get_inscription_all_data_by_id(
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

#[cfg(test)]
mod tests {
  use super::*;
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
