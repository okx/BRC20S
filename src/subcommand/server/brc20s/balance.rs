use {super::*, axum::Json};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Balance {
  pub tick: Tick,
  pub transferable: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AllBalance {
  pub balance: Vec<Balance>,
}

// brc20s/tick/:tickId/address/:address/balance
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

// brc20s/address/:address/balance
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
