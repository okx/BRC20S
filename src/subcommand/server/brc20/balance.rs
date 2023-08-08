use {super::*, crate::okx::datastore::brc20::Tick, axum::Json, utoipa::ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
  /// Name of the ticker.
  pub tick: String,
  /// Available balance.
  pub available_balance: String,
  /// Transferable balance.
  pub transferable_balance: String,
  /// Overall balance.
  pub overall_balance: String,
}

#[utoipa::path(
    get,
    path = "/brc20/tick/{ticker}/address/{address}/balance",
    operation_id = "get the ticker balance of the address",
    params(
        ("ticker" = String, Path, description = "Token ticker", min_length = 4, max_length = 4),
        ("address" = String, Path, description = "Address")
  ),
    responses(
      (status = 200, description = "Obtain account balance by query ticker.", body = BRC20BalanceResponse),
      (status = 400, description = "Bad query.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::bad_request(BRC20Error::IncorrectTickFormat)))),
      (status = 404, description = "Ticker not found.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::not_found(BRC20Error::TickNotFound)))),
      (status = 500, description = "Internal server error.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::internal("internal error")))),
    )
  )]

pub(crate) async fn brc20_balance(
  Extension(index): Extension<Arc<Index>>,
  Path((tick, address)): Path<(String, String)>,
) -> ApiResult<Balance> {
  log::debug!("rpc: get brc20_balance: {} {}", tick, address);

  let tick =
    Tick::from_str(&tick).map_err(|_| ApiError::bad_request(BRC20Error::IncorrectTickFormat))?;

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let balance = index
    .brc20_get_balance_by_address(&tick, &address)?
    .ok_or_api_not_found(BRC20Error::BalanceNotFound)?;

  let available_balance = balance.overall_balance - balance.transferable_balance;

  log::debug!("rpc: get brc20_balance: {} {} {:?}", tick, address, balance);

  Ok(Json(ApiResponse::ok(Balance {
    tick: balance.tick.to_string(),
    available_balance: available_balance.to_string(),
    transferable_balance: balance.transferable_balance.to_string(),
    overall_balance: balance.overall_balance.to_string(),
  })))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AllBalance {
  pub balance: Vec<Balance>,
}

#[utoipa::path(
    get,
    path = "/brc20/address/{address}/balance",
    operation_id = "get all ticker balances of the address",
    params(
        ("address" = String, Path, description = "Address")
  ),
    responses(
      (status = 200, description = "Obtain account balances by query address.", body = BRC20AllBalanceResponse),
      (status = 400, description = "Bad query.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::bad_request(BRC20Error::IncorrectTickFormat)))),
      (status = 404, description = "Ticker not found.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::not_found(BRC20Error::TickNotFound)))),
      (status = 500, description = "Internal server error.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::internal("internal error")))),
    )
  )]
pub(crate) async fn brc20_all_balance(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<AllBalance> {
  log::debug!("rpc: get brc20_all_balance: {}", address);

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let all_balance = index.brc20_get_all_balance_by_address(&address)?;

  log::debug!("rpc: get brc20_all_balance: {} {:?}", address, all_balance);

  Ok(Json(ApiResponse::ok(AllBalance {
    balance: all_balance
      .iter()
      .map(|bal| Balance {
        tick: bal.tick.to_string(),
        available_balance: (bal.overall_balance - bal.transferable_balance).to_string(),
        transferable_balance: bal.transferable_balance.to_string(),
        overall_balance: bal.overall_balance.to_string(),
      })
      .collect(),
  })))
}
