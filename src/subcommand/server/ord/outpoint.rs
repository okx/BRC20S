use {
  super::{error::ApiError, types::ScriptPubkey, *},
  crate::okx::datastore::ScriptKey,
  axum::Json,
  utoipa::ToSchema,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = ord::InscriptionDigest)]
#[serde(rename_all = "camelCase")]
pub struct InscriptionDigest {
  /// The inscription id.
  pub id: String,
  /// The inscription number.
  pub number: i64,
  /// The inscription location.
  pub location: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = ord::OutPointData)]
#[serde(rename_all = "camelCase")]
pub struct OutPointData {
  /// The transaction id.
  pub txid: String,
  /// The script pubkey.
  pub script_pub_key: String,
  /// The owner of the script pubkey.
  pub owner: ScriptPubkey,
  /// The value of the transaction output.
  #[schema(format = "uint64")]
  pub value: u64,
  #[schema(value_type = Vec<ord::InscriptionDigest>)]
  /// The inscriptions on the transaction output.
  pub inscription_digest: Vec<InscriptionDigest>,
}

// /ord/outpoint/:outpoint/info
#[utoipa::path(
  get,
  path = "/api/v1/ord/outpoint/{outpoint}/info",
  operation_id = "get outpoint infomation",
  params(
      ("outpoint" = String, Path, description = "Outpoint")
),
  responses(
    (status = 200, description = "Obtain outpoint infomation", body = OrdOutPointData),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
pub(crate) async fn ord_outpoint(
  Extension(index): Extension<Arc<Index>>,
  Path(outpoint): Path<OutPoint>,
) -> ApiResult<OutPointData> {
  log::debug!("rpc: get ord_outpoint: {outpoint}");

  let inscription_ids = index.get_inscriptions_on_output(outpoint)?;
  if inscription_ids.is_empty() {
    return Err(ApiError::not_found(format!(
      "Inscription not found from outpoint: {outpoint}"
    )));
  }

  // Get the txout from the database store or from an RPC request.
  let vout = index
    .get_transaction_output_by_outpoint(outpoint)
    .and_then(|v| {
      v.ok_or(anyhow!(format!(
        "Outpoint not found from db store: {outpoint}"
      )))
    })
    .or_else(|_| {
      index
        .get_transaction_with_retries(outpoint.txid)
        .and_then(|v| {
          v.map(|tx| {
            tx.output
              .get(usize::try_from(outpoint.vout).unwrap())
              .unwrap()
              .to_owned()
          })
          .ok_or(anyhow!(format!("Can't get transaction: {}", outpoint.txid)))
        })
    })?;

  let mut inscription_digests = Vec::with_capacity(inscription_ids.len());
  for id in inscription_ids {
    inscription_digests.push(InscriptionDigest {
      id: id.to_string(),
      number: index
        .get_inscription_entry(id)?
        .map(|entry| entry.number)
        .ok_or(anyhow!(
          "Failed to get the inscription number by ID, there may be an error in the database."
        ))?,
      location: index
        .get_inscription_satpoint_by_id(id)?
        .ok_or(anyhow!(
          "Failed to get the inscription location, there may be an error in the database."
        ))?
        .to_string(),
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
