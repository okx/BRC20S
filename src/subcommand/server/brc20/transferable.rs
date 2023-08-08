use {super::*, crate::okx::datastore::brc20 as brc20_store, axum::Json, utoipa::ToSchema};

#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransferableInscription {
  pub inscription_id: String,
  pub inscription_number: i64,
  pub amount: String,
  pub tick: String,
  pub owner: String,
}

impl From<&brc20_store::TransferableLog> for TransferableInscription {
  fn from(trans: &brc20_store::TransferableLog) -> Self {
    Self {
      inscription_id: trans.inscription_id.to_string(),
      inscription_number: trans.inscription_number,
      amount: trans.amount.to_string(),
      tick: trans.tick.as_str().to_string(),
      owner: trans.owner.to_string(),
    }
  }
}

#[utoipa::path(
  get,
  path = "/brc20/tick/{ticker}/address/{address}/transferable",
  operation_id = "get transferable inscriptions",
  params(
      ("ticker" = String, Path, description = "Token ticker", min_length = 4, max_length = 4),
      ("address" = String, Path, description = "Address")
),
  responses(
    (status = 200, description = "Obtain account transferable inscriptions of ticker.", body = BRC20TransferableResponse),
    (status = 400, description = "Bad query.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::bad_request(BRC20Error::IncorrectTickFormat)))),
    (status = 404, description = "Ticker not found.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::not_found(BRC20Error::TickNotFound)))),
    (status = 500, description = "Internal server error.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::internal("internal error")))),
  )
)]
pub(crate) async fn brc20_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path((tick, address)): Path<(String, String)>,
) -> ApiResult<TransferableInscriptions> {
  log::debug!("rpc: get brc20_transferable: {tick} {address}");

  let tick = brc20_store::Tick::from_str(&tick)
    .map_err(|_| ApiError::bad_request(BRC20Error::IncorrectTickFormat))?;

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let transferable = index.brc20_get_tick_transferable_by_address(&tick, &address)?;

  log::debug!(
    "rpc: get brc20_transferable: {tick} {address} {:?}",
    transferable
  );

  Ok(Json(ApiResponse::ok(TransferableInscriptions {
    inscriptions: transferable.iter().map(|trans| trans.into()).collect(),
  })))
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransferableInscriptions {
  pub inscriptions: Vec<TransferableInscription>,
}

#[utoipa::path(
  get,
  path = "/brc20/address/{address}/transferable",
  operation_id = "get address ticker balance",
  params(
      ("address" = String, Path, description = "Address")
),
  responses(
    (status = 200, description = "Obtain account all transferable inscriptions.", body = BRC20AllTransferableResponse),
    (status = 400, description = "Bad query.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::bad_request(BRC20Error::IncorrectTickFormat)))),
    (status = 404, description = "Ticker not found.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::not_found(BRC20Error::TickNotFound)))),
    (status = 500, description = "Internal server error.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::internal("internal error")))),
  )
)]
pub(crate) async fn brc20_all_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<TransferableInscriptions> {
  log::debug!("rpc: get brc20_all_transferable: {address}");

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let transferable = index.brc20_get_all_transferable_by_address(&address)?;
  log::debug!(
    "rpc: get brc20_all_transferable: {address} {:?}",
    transferable
  );

  Ok(Json(ApiResponse::ok(TransferableInscriptions {
    inscriptions: transferable.iter().map(|trans| trans.into()).collect(),
  })))
}
