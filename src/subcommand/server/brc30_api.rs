use super::brc30_types::*;
use super::error::ApiError;
use super::*;
use crate::okx::datastore::{ScriptKey, BRC30};
use axum::Json;

// 3.4.1 /brc30/tick
pub(crate) async fn brc30_all_tick_info(
  Extension(index): Extension<Arc<Index>>,
) -> ApiResult<AllBRC30TickInfo> {
  log::debug!("rpc: get brc30_all_tick_info");

  // TODO
  // Ok(Json(ApiResponse::ok(AllTickInfo {
  //   tokens: all_tick_info.iter().map(|t| t.into()).collect(),
  // })))

  return Err(ApiError::bad_request("".to_string()));
}

// 3.4.2 /brc30/tick/:tickId
pub(crate) async fn brc30_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tickId): Path<String>,
) -> ApiResult<BRC30TickInfo> {
  log::debug!("rpc: get brc30_tick_info: {}", tickId);

  // TODO
  // Ok(Json(ApiResponse::ok(tick_info.into())))

  log::debug!("rpc: get brc30_tick_info: {}", tickId);
  return Err(ApiError::bad_request("".to_string()));
}

// brc30/pool
pub(crate) async fn brc30_all_pool_info(
  Extension(index): Extension<Arc<Index>>,
) -> ApiResult<AllBRC30PoolInfo> {
  log::debug!("rpc: get brc30_all_pool_info");

  // TODO
  // Ok(Json(ApiResponse::ok(AllTickInfo {
  //   tokens: all_tick_info.iter().map(|t| t.into()).collect(),
  // })))

  return Err(ApiError::bad_request("".to_string()));
}

// 3.4.4 /brc30/pool/:pid
pub(crate) async fn brc30_pool_info(
  Extension(index): Extension<Arc<Index>>,
  Path(pid): Path<String>,
) -> ApiResult<BRC30Pool> {
  log::debug!("rpc: get brc30_pool_info: {}", pid);

  // TODO
  // Ok(Json(ApiResponse::ok(tick_info.into())))

  log::debug!("rpc: get brc30_pool_info: {}", pid);
  return Err(ApiError::bad_request("".to_string()));
}

// 3.4.5 /brc30/pool/:pid/address/:address/userinfo
pub(crate) async fn brc30_userinfo(
  Extension(index): Extension<Arc<Index>>,
  Path((pid, address)): Path<(String, String)>,
) -> ApiResult<UserInfo> {
  log::debug!("rpc: get brc30_userinfo: {}, {}", pid, address);

  // TODO
  // Ok(Json(ApiResponse::ok(tick_info.into())))

  log::debug!("rpc: get brc30_userinfo: {}, {}", pid, address);
  return Err(ApiError::bad_request("".to_string()));
}

// 3.4.6 /brc30/tick/:tickId/address/:address/balance
pub(crate) async fn brc30_balance(
  Extension(index): Extension<Arc<Index>>,
  Path((tickId, address)): Path<(String, String)>,
) -> ApiResult<BRC30Balance> {
  log::debug!(
    "rpc: get brc30_balance: tickId:{}, address:{}",
    tickId,
    address
  );

  // TODO
  // Ok(Json(ApiResponse::ok(tick_info.into())))

  log::debug!(
    "rpc: get brc30_balance: tickId:{}, address:{}",
    tickId,
    address
  );
  return Err(ApiError::bad_request("".to_string()));
}

// 3.4.7 /brc30/address/:address/balance
pub(crate) async fn brc30_all_balance(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<AllBRC30Balance> {
  log::debug!("rpc: get brc30_all_balance: {}", address);

  // TODO
  // Ok(Json(ApiResponse::ok(tick_info.into())))

  log::debug!("rpc: get brc30_all_balance: {}", address);
  return Err(ApiError::bad_request("".to_string()));
}

// 3.4.8 /brc30/tick/:tickId/address/:address/transferable
pub(crate) async fn brc30_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path((tickId, address)): Path<(String, String)>,
) -> ApiResult<Transferable> {
  log::debug!("rpc: get brc30_transferable: {},{}", tickId, address);

  // TODO
  // Ok(Json(ApiResponse::ok(tick_info.into())))

  log::debug!("rpc: get brc30_transferable: {},{}", tickId, address);
  return Err(ApiError::bad_request("".to_string()));
}

// 3.4.9 /brc30/address/:address/transferable
pub(crate) async fn brc30_all_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<Transferable> {
  log::debug!("rpc: get brc30_all_transferable: {}", address);

  // TODO
  // Ok(Json(ApiResponse::ok(tick_info.into())))

  log::debug!("rpc: get brc30_all_transferable: {}", address);
  return Err(ApiError::bad_request("".to_string()));
}

// 3.4.10 /brc30/tx/:txid/events
pub(crate) async fn brc30_txid_events(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<Events> {
  log::debug!("rpc: get brc30_txid_events: {}", txid);

  // TODO
  // Ok(Json(ApiResponse::ok(tick_info.into())))

  log::debug!("rpc: get brc30_txid_events: {}", txid);
  return Err(ApiError::bad_request("".to_string()));
}

// 3.4.11 /brc30/block/:blockhash/events
pub(crate) async fn brc30_block_events(
  Extension(index): Extension<Arc<Index>>,
  Path(blockhash): Path<String>,
) -> ApiResult<Events> {
  log::debug!("rpc: get brc30_block_events: {}", blockhash);

  // TODO
  // Ok(Json(ApiResponse::ok(tick_info.into())))

  log::debug!("rpc: get brc30_block_events: {}", blockhash);
  return Err(ApiError::bad_request("".to_string()));
}
