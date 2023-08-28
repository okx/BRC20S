use {
  super::{error::ApiError, types::ScriptPubkey, *},
  crate::{index::InscriptionEntry, okx::datastore::ScriptKey},
  axum::Json,
  utoipa::ToSchema,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = ord::OrdInscription)]
#[serde(rename_all = "camelCase")]
pub struct OrdInscription {
  /// The inscription id.
  pub id: String,
  /// The inscription number.
  pub number: i64,
  /// The inscription content type.
  pub content_type: Option<String>,
  /// The inscription content body.
  pub content: Option<String>,
  /// The inscription owner.
  pub owner: Option<ScriptPubkey>,
  /// The inscription genesis block height.
  #[schema(format = "uint64")]
  pub genesis_height: u64,
  /// The inscription location.
  pub location: String,
  /// The inscription sat index.  
  pub sat: Option<u64>,
}

// /ord/id/:id/inscription
#[utoipa::path(
  get,
  path = "/api/v1/ord/id/{id}/inscription",
  operation_id = "get inscription infomation by inscription ID",
  params(
      ("id" = String, Path, description = "inscription ID")
),
  responses(
    (status = 200, description = "Obtain inscription infomation.", body = OrdOrdInscription),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
pub(crate) async fn ord_inscription_id(
  Extension(index): Extension<Arc<Index>>,
  Path(id): Path<String>,
) -> ApiResult<OrdInscription> {
  log::debug!("rpc: get ord_inscription_id: {id}");
  let id = InscriptionId::from_str(&id).map_err(|e| ApiError::bad_request(e.to_string()))?;

  ord_get_inscription_by_id(index, id)
}

// /ord/number/:number/inscription
#[utoipa::path(
  get,
  path = "/api/v1/ord/number/{number}/inscription",
  operation_id = "get inscription infomation by inscription number",
  params(
      ("number" = i64, Path, description = "inscription number")
),
  responses(
    (status = 200, description = "Obtain inscription infomation.", body = OrdOrdInscription),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
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
    id: id.to_string(),
    number: inscription_data.entry.number,
    content_type: inscription_data
      .inscription
      .content_type()
      .map(String::from),
    content: inscription_data.inscription.body().map(hex::encode),
    owner,
    genesis_height: inscription_data.entry.height,
    location: inscription_data.sat_point.to_string(),
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
      }
      .to_string(),
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
      .unwrap()
      .to_string(),
      sat: None,
    };
    assert_eq!(
      serde_json::to_string_pretty(&ord_inscription).unwrap(),
      r#"{
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
}"#,
    );
    ord_inscription.owner = None;
    assert_eq!(
      serde_json::to_string_pretty(&ord_inscription).unwrap(),
      r#"{
  "id": "1111111111111111111111111111111111111111111111111111111111111111i4294967295",
  "number": -100,
  "contentType": "content_type",
  "content": "content",
  "owner": null,
  "genesisHeight": 1,
  "location": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "sat": null
}"#,
    );
  }
}
