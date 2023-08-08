use {
  super::{error::ApiError, types::ScriptPubkey, *},
  crate::okx::datastore::ScriptKey,
  axum::Json,
  utoipa::ToSchema,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InscriptionDigest {
  pub id: String,
  pub number: i64,
  pub location: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OutPointData {
  pub txid: String,
  pub script_pub_key: String,
  pub owner: ScriptPubkey,
  pub value: u64,
  pub inscription_digest: Vec<InscriptionDigest>,
}

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
