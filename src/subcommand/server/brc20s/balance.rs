use {super::*, axum::Json, utoipa::ToSchema};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = brc20s::Balance)]
pub(crate) struct Balance {
  /// Ticker.
  #[schema(value_type = brc20s::Tick)]
  pub tick: Tick,
  /// Transferable balance.
  #[schema(format = "uint64")]
  pub transferable: String,
  /// Overall balance.
  #[schema(format = "uint64")]
  pub overall: String,
}

impl Balance {
  pub fn set_tick_name(&mut self, name: String) {
    self.tick.name = name;
  }
}

impl From<&brc20s::Balance> for Balance {
  fn from(balance: &brc20s::Balance) -> Self {
    let tick = Tick {
      id: balance.tick_id.hex(),
      name: "".to_string(),
    };

    Self {
      tick,
      transferable: balance.transferable_balance.to_string(),
      overall: balance.overall_balance.to_string(),
    }
  }
}

// brc20s/tick/:tickId/address/:address/balance

/// Get the ticker balance of the address.
///
/// The balance is the sum of the transferable balance and the available balance.
#[utoipa::path(
  get,
  path = "/api/v1/brc20s/tick/{tick_id}/address/{address}/balance",
  params(
      ("tick_id" = String, Path, description = "Token ticker ID", min_length = 10, max_length = 10),
      ("address" = String, Path, description = "Address")
),
  responses(
    (status = 200, description = "Obtain account balance by query ticker.", body = BRC20SBalance),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
pub(crate) async fn brc20s_balance(
  Extension(index): Extension<Arc<Index>>,
  Path((tick_id, address)): Path<(String, String)>,
) -> ApiResult<Balance> {
  log::debug!(
    "rpc: get brc20s_balance: tickId:{}, address:{}",
    tick_id,
    address
  );

  let tick_id = brc20s::TickId::from_str(&tick_id)
    .map_err(|_| ApiError::bad_request(BRC20SError::IncorrectTickIdFormat))?;

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;
  let balance = &index
    .brc20s_balance(&tick_id, &address)?
    .ok_or_api_not_found(BRC20SError::BalanceNotFound)?;

  let mut balance_result = Balance::from(balance);

  let tick_info = &index
    .brc20s_tick_info(&tick_id)?
    .ok_or_api_not_found(BRC20SError::TickIdNotFound)?;

  balance_result.set_tick_name(tick_info.name.as_str().to_string());
  log::debug!(
    "rpc: get brc20s_balance: {:?} {:?}",
    tick_id.hex(),
    balance_result
  );

  Ok(Json(ApiResponse::ok(balance_result)))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = brc20s::AllBalance)]
pub(crate) struct AllBalance {
  #[schema(value_type = Vec<brc20s::Balance>)]
  pub balance: Vec<Balance>,
}
// brc20s/address/:address/balance
/// Get all ticker balances of the address.
///
/// Retrieve all asset balances of the address.
#[utoipa::path(
  get,
  path = "/api/v1/brc20s/address/{address}/balance",
  params(
      ("address" = String, Path, description = "Address")
),
  responses(
    (status = 200, description = "Obtain account balances by query address.", body = BRC20SAllBalance),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
pub(crate) async fn brc20s_all_balance(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<AllBalance> {
  log::debug!("rpc: get brc20s_all_balance: {}", address);

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let all_balance = index.brc20s_all_balance(&address)?;

  log::debug!("rpc: get brc20s_all_balance: {} {:?}", address, all_balance);

  Ok(Json(ApiResponse::ok(AllBalance {
    balance: all_balance
      .iter()
      .map(|(tick_id, balance)| {
        let mut balance_result = Balance::from(balance);

        let tick_info = &index.brc20s_tick_info(tick_id).unwrap().unwrap();

        balance_result.set_tick_name(tick_info.name.as_str().to_string());
        log::debug!(
          "rpc: get brc20s_userinfo: {:?} {:?}",
          tick_id,
          balance_result
        );
        balance_result
      })
      .collect(),
  })))
}
// brc20s/debug/tick/:tickId/address/:address/balance
pub(crate) async fn brc20s_debug_balance(
  Extension(index): Extension<Arc<Index>>,
  Path((tick_id, address)): Path<(String, String)>,
) -> ApiResult<brc20s::Balance> {
  log::debug!(
    "rpc: get brc20s_debug_balance: tickId:{}, address:{}",
    tick_id,
    address
  );

  let tick_id = brc20s::TickId::from_str(&tick_id)
    .map_err(|_| ApiError::bad_request(BRC20SError::IncorrectTickIdFormat))?;
  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;
  let balance = index
    .brc20s_balance(&tick_id, &address)?
    .ok_or_api_not_found(BRC20SError::BalanceNotFound)?;

  log::debug!(
    "rpc: get brc20s_debug_balance: {:?} {:?}",
    tick_id.hex(),
    balance
  );

  Ok(Json(ApiResponse::ok(balance)))
}
