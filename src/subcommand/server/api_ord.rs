use self::api::*;
use super::error::ApiError;
use super::*;
use axum::Json;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutPointData {
  pub txid: String,
  pub script_pub_key: String,
  pub address: Option<String>,
  pub value: u64,
  pub inscription_digest: Vec<InscriptionDigest>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscriptionDigest {
  pub id: String,
  pub number: u64,
  pub location: String,
}

fn ord_get_inscription_by_id(index: Arc<Index>, id: InscriptionId) -> ApiResult<OrdInscription> {
  let inscription_data = index
    .get_inscription_all_data_by_id(id)?
    .ok_or_api_notfound("inscription not found")?;

  Ok(Json(ApiResponse::ok(OrdInscription {
    id: id.to_string(),
    number: inscription_data.entry.number.to_string(),
    content_type: inscription_data
      .inscription
      .content_type()
      .map(|c| String::from(c)),
    content: inscription_data.inscription.body().map(|c| hex::encode(c)),
    owner: brc20::ScriptKey::from_script(
      &inscription_data.tx.output[0].script_pubkey,
      index.get_chain_network(),
    )
    .to_string(),
    genesis_height: inscription_data.entry.height,
    location: inscription_data.sat_point.to_string(),
    sat: inscription_data.entry.sat.map(|s| s.0),
  })))
}

pub(crate) async fn ord_inscription_id(
  Extension(index): Extension<Arc<Index>>,
  Path(id): Path<String>,
) -> ApiResult<OrdInscription> {
  log::debug!("rpc: get ord_inscription_id: {}", id);
  let id = InscriptionId::from_str(&id).map_err(|e| ApiError::bad_request(e.to_string()))?;

  ord_get_inscription_by_id(index, id)
}

pub(crate) async fn ord_inscription_number(
  Extension(index): Extension<Arc<Index>>,
  Path(number): Path<u64>,
) -> ApiResult<OrdInscription> {
  log::debug!("rpc: get ord_inscription_number: {}", number);

  let id = index
    .get_inscription_id_by_inscription_number(number)?
    .ok_or_api_notfound("inscription not found")?;

  ord_get_inscription_by_id(index, id)
}

pub(crate) async fn ord_outpoint(
  Extension(index): Extension<Arc<Index>>,
  Path(outpoint): Path<OutPoint>,
) -> ApiResult<OutPointData> {
  log::debug!("rpc: get ord_outpoint: {}", outpoint);

  let inscription_ids = index.get_inscriptions_on_output(outpoint)?;
  if inscription_ids.is_empty() {
    return Err(ApiError::not_found("inscription not found"));
  }

  let tx = index
    .get_transaction(outpoint.txid)?
    .ok_or_api_notfound("transaction not found")?;

  let vout = tx
    .output
    .get(outpoint.vout as usize)
    .ok_or_api_notfound("vout not found")?;

  let mut inscription_digests = Vec::with_capacity(inscription_ids.len());
  for id in &inscription_ids {
    let ins_data = index
      .get_inscription_entry(id.clone())?
      .ok_or_api_notfound("inscription not found")?;

    let satpoint = index
      .get_inscription_satpoint_by_id(id.clone())?
      .ok_or_api_notfound("inscription not found")?;

    inscription_digests.push(InscriptionDigest {
      id: id.to_string(),
      number: ins_data.number,
      location: satpoint.to_string(),
    });
  }

  Ok(Json(ApiResponse::ok(OutPointData {
    txid: outpoint.txid.to_string(),
    script_pub_key: vout.script_pubkey.asm(),
    address: match brc20::ScriptKey::from_script(&vout.script_pubkey, index.get_chain_network()) {
      brc20::ScriptKey::Address(address) => Some(address.to_string()),
      _ => None,
    },
    value: vout.value,
    inscription_digest: inscription_digests,
  })))
}
