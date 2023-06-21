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

  let all_tick_info = index.brc30_all_tick_info()?;
  log::debug!("rpc: get brc30_all_tick_info: {:?}", all_tick_info);

  Ok(Json(ApiResponse::ok(AllBRC30TickInfo {
    tokens: all_tick_info.iter().map(|t| t.into()).collect(),
  })))
}

// 3.4.2 /brc30/tick/:tickId
pub(crate) async fn brc30_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tickId): Path<String>,
) -> ApiResult<BRC30TickInfo> {
  log::debug!("rpc: get brc30_tick_info: {}", tickId.to_string());
  if tickId.as_bytes().len() != 5 {
    return Err(ApiError::bad_request("tick id length must 5."));
  }
  let tickId = tickId.to_lowercase();

  let tick_info = &index
    .brc30_tick_info(&tickId)?
    .ok_or_api_not_found("tick not found")?;

  log::debug!("rpc: get brc30_tick_info: {:?} {:?}", tickId, tick_info);

  if tick_info.tick_id != BRC30::TickId::from_str(&tickId).unwrap() {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(tick_info.into())))
}

// brc30/pool
pub(crate) async fn brc30_all_pool_info(
  Extension(index): Extension<Arc<Index>>,
) -> ApiResult<AllBRC30PoolInfo> {
  log::debug!("rpc: get brc30_all_pool_info");
  let all_pool_info = index.brc30_all_pool_info()?;
  log::debug!("rpc: get brc30_all_pool_info: {:?}", all_pool_info);
  Ok(Json(ApiResponse::ok(AllBRC30PoolInfo {
    tokens: all_pool_info.iter().map(|(pool)| pool.into()).collect(),
  })))
}

// 3.4.4 /brc30/pool/:pid
pub(crate) async fn brc30_pool_info(
  Extension(index): Extension<Arc<Index>>,
  Path(pid): Path<String>,
) -> ApiResult<BRC30Pool> {
  log::debug!("rpc: get brc30_pool_info: {}", pid);

  if pid.as_bytes().len() != 13 {
    return Err(ApiError::bad_request("pid length must 13."));
  }
  let pid = pid.to_lowercase();

  let pool_info = &index
    .brc30_pool_info(&pid)?
    .ok_or_api_not_found("pid not found")?;

  log::debug!(
    "rpc: get brc30_pool_info: {:?} {:?}",
    pid.as_str(),
    pool_info
  );

  if pool_info.pid != BRC30::Pid::from_str(&pid).unwrap() {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(pool_info.into())))
}

// 3.4.5 /brc30/pool/:pid/address/:address/userinfo
pub(crate) async fn brc30_userinfo(
  Extension(index): Extension<Arc<Index>>,
  Path((pid, address)): Path<(String, String)>,
) -> ApiResult<UserInfo> {
  log::debug!("rpc: get brc30_userinfo: {}, {}", pid, address);

  if pid.as_bytes().len() != 13 {
    return Err(ApiError::bad_request("pid length must 13."));
  }
  let pid = pid.to_lowercase();
  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;
  let user_info = &index
    .brc30_user_info(&pid, &address)?
    .ok_or_api_not_found("pid not found")?;

  log::debug!(
    "rpc: get brc30_userinfo: {:?} {:?}",
    pid.as_str(),
    user_info
  );

  if user_info.pid != BRC30::Pid::from_str(&pid).unwrap() {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(user_info.into())))
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

  if tickId.as_bytes().len() != 5 {
    return Err(ApiError::bad_request("tick id length must 5."));
  }
  let tickId = tickId.to_lowercase();
  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;
  let balance = &index
    .brc30_balance(&tickId, &address)?
    .ok_or_api_not_found("pid not found")?;

  log::debug!(
    "rpc: get brc30_userinfo: {:?} {:?}",
    tickId.as_str(),
    balance
  );

  Ok(Json(ApiResponse::ok(balance.into())))
}

// 3.4.7 /brc30/address/:address/balance
pub(crate) async fn brc30_all_balance(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<AllBRC30Balance> {
  log::debug!("rpc: get brc30_all_balance: {}", address);

  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;

  let all_balance = index.brc30_all_balance(&address)?;

  log::debug!("rpc: get brc30_all_balance: {} {:?}", address, all_balance);

  Ok(Json(ApiResponse::ok(AllBRC30Balance {
    balance: all_balance.iter().map(|(tickid, bal)| bal.into()).collect(),
  })))
}

// 3.4.8 /brc30/tick/:tickId/address/:address/transferable
pub(crate) async fn brc30_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path((tick_id, address)): Path<(String, String)>,
) -> ApiResult<Transferable> {
  log::debug!("rpc: get brc30_transferable: {},{}", tick_id, address);

  if tick_id.as_bytes().len() != 5 {
    return Err(ApiError::bad_request("tick id length must 5."));
  }
  let tick_id = tick_id.to_lowercase();
  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;
  let all_transfer = &index.brc30_tickid_transferable(&tick_id, &address)?;

  log::debug!(
    "rpc: get brc30_transferable: {:?} {:?}",
    tick_id.as_str(),
    all_transfer
  );

  Ok(Json(ApiResponse::ok(Transferable {
    inscriptions: all_transfer
      .iter()
      .map(|(transfer)| transfer.into())
      .collect(),
  })))
}

// 3.4.9 /brc30/address/:address/transferable
pub(crate) async fn brc30_all_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<Transferable> {
  log::debug!("rpc: get brc30_all_transferable: {}", address);

  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;

  let all = index.brc30_all_transferable(&address)?;

  log::debug!("rpc: get brc30_all_transferable: {} {:?}", address, all);

  Ok(Json(ApiResponse::ok(Transferable {
    inscriptions: all.iter().map(|(transer)| transer.into()).collect(),
  })))
}

// 3.4.10 /brc30/tx/:txid/events
pub(crate) async fn brc30_txid_events(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<Events> {
  log::debug!("rpc: get brc30_txid_events: {}", txid);
  let txid = Txid::from_str(&txid).unwrap();

  let all_receipt = index.brc30_txid_events(&txid)?;

  log::debug!("rpc: get brc30_txid_events: {:?}", all_receipt);

  Ok(Json(ApiResponse::ok(Events {
    events: all_receipt.iter().map(|(receipt)| receipt.into()).collect(),
    txid: txid.to_string(),
  })))
}

// 3.4.11 /brc30/block/:blockhash/events
pub(crate) async fn brc30_block_events(
  Extension(index): Extension<Arc<Index>>,
  Path(blockhash): Path<String>,
) -> ApiResult<BRC30BlockEvents> {
  log::debug!("rpc: get brc30_block_events: {}", blockhash);

  let all_receipt = index.brc30_block_events()?;

  log::debug!("rpc: get brc30_block_events: {:?}", all_receipt);

  Ok(Json(ApiResponse::ok(BRC30BlockEvents {
    block: all_receipt.iter().map(|(receipt)| receipt.into()).collect(),
  })))
}
