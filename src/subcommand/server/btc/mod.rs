use {super::*, crate::okx::datastore::btc::Balance, axum::Json};

// btc/debug/address/:address/balance
pub(crate) async fn btc_debug_balance(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<Balance> {
  log::debug!("rpc: get btc_debug_balance: address:{}", address);

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let balance = index
    .btc_balance(&address)?
    .ok_or_api_not_found("balance not found")?;

  log::debug!("rpc: get btc_debug_balance: {:?}", balance);

  Ok(Json(ApiResponse::ok(balance)))
}
