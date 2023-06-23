use super::brc30_types::*;
use super::error::ApiError;
use super::*;
use crate::okx::datastore::brc30;
use crate::okx::datastore::brc30::PledgedTick;
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
  Path(tick_id): Path<String>,
) -> ApiResult<BRC30TickInfo> {
  log::debug!("rpc: get brc30_tick_info: {}", tick_id.to_string());
  let tick_id = tick_id.to_lowercase();

  let tick_info = &index
    .brc30_tick_info(&tick_id)?
    .ok_or_api_not_found("tick not found")?;

  log::debug!("rpc: get brc30_tick_info: {:?} {:?}", tick_id, tick_info);

  if tick_info.tick_id != brc30::TickId::from_str(&tick_id).unwrap() {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(tick_info.into())))
}

// 3.4.2 /brc30/tick/:tickId
pub(crate) async fn brc30_debug_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tick_id): Path<String>,
) -> ApiResult<brc30::TickInfo> {
  log::debug!("rpc: get brc30_tick_info: {}", tick_id.to_string());
  let tick_id = tick_id.to_lowercase();

  let tick_info = index
    .brc30_tick_info(&tick_id)?
    .ok_or_api_not_found("tick not found")?;

  log::debug!("rpc: get brc30_tick_info: {:?} {:?}", tick_id, tick_info);

  if tick_info.tick_id != brc30::TickId::from_str(&tick_id).unwrap() {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(tick_info)))
}

// brc30/pool
pub(crate) async fn brc30_all_pool_info(
  Extension(index): Extension<Arc<Index>>,
) -> ApiResult<AllBRC30PoolInfo> {
  log::debug!("rpc: get brc30_all_pool_info");
  let all_pool_info = index.brc30_all_pool_info()?;
  log::debug!("rpc: get brc30_all_pool_info: {:?}", all_pool_info);
  Ok(Json(ApiResponse::ok(AllBRC30PoolInfo {
    tokens: all_pool_info.iter().map(|pool| pool.into()).collect(),
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

  if pool_info.pid != brc30::Pid::from_str(&pid).unwrap() {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(pool_info.into())))
}

pub(crate) async fn brc30_debug_pool_info(
  Extension(index): Extension<Arc<Index>>,
  Path(pid): Path<String>,
) -> ApiResult<brc30::PoolInfo> {
  log::debug!("rpc: get brc30_pool_info: {}", pid);

  if pid.as_bytes().len() != 13 {
    return Err(ApiError::bad_request("pid length must 13."));
  }
  let pid = pid.to_lowercase();

  let pool_info = index
    .brc30_pool_info(&pid)?
    .ok_or_api_not_found("pid not found")?;

  log::debug!(
    "rpc: get brc30_pool_info: {:?} {:?}",
    pid.as_str(),
    pool_info
  );

  Ok(Json(ApiResponse::ok(pool_info)))
}

pub(crate) async fn brc30_debug_stake_info(
  Extension(index): Extension<Arc<Index>>,
  Path((address, tick)): Path<(String, String)>,
) -> ApiResult<brc30::StakeInfo> {
  log::debug!("rpc: get brc30_pool_info: {},{}", address, tick);

  let address: bitcoin::Address = address
    .parse()
    .map_err(|err: bitcoin::util::address::Error| ApiError::bad_request(err.to_string()))?;

  let pool_info = index
    .brc30_stake_info(&address, &PledgedTick::from_str(tick.as_str()))?
    .ok_or_api_not_found("pid not found")?;

  log::debug!("rpc: get brc30_pool_info: {:?}", pool_info);

  Ok(Json(ApiResponse::ok(pool_info)))
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

  if user_info.pid != brc30::Pid::from_str(&pid).unwrap() {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(user_info.into())))
}

// 3.4.5 /brc30/debug/pool/:pid/address/:address/userinfo
pub(crate) async fn brc30_debug_userinfo(
  Extension(index): Extension<Arc<Index>>,
  Path((pid, address)): Path<(String, String)>,
) -> ApiResult<brc30::UserInfo> {
  log::debug!("rpc: get brc30_userinfo: {}, {}", pid, address);

  if pid.as_bytes().len() != 13 {
    return Err(ApiError::bad_request("pid length must 13."));
  }
  let pid = pid.to_lowercase();
  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;
  let user_info = index
    .brc30_user_info(&pid, &address)?
    .ok_or_api_not_found("pid not found")?;

  log::debug!(
    "rpc: get brc30_userinfo: {:?} {:?}",
    pid.as_str(),
    user_info
  );

  if user_info.pid != brc30::Pid::from_str(&pid).unwrap() {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(user_info)))
}

// 3.4.6 /brc30/debug/tick/:tickId/address/:address/balance
pub(crate) async fn brc30_debug_balance(
  Extension(index): Extension<Arc<Index>>,
  Path((tick_id, address)): Path<(String, String)>,
) -> ApiResult<brc30::Balance> {
  log::debug!(
    "rpc: get brc30_balance: tickId:{}, address:{}",
    tick_id,
    address
  );

  if tick_id.as_bytes().len() != 5 {
    return Err(ApiError::bad_request("tick id length must 5."));
  }
  let tick_id = tick_id.to_lowercase();
  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;
  let balance = index
    .brc30_balance(&tick_id, &address)?
    .ok_or_api_not_found("pid not found")?;

  log::debug!(
    "rpc: get brc30_userinfo: {:?} {:?}",
    tick_id.as_str(),
    balance
  );

  Ok(Json(ApiResponse::ok(balance)))
}

// 3.4.6 /brc30/tick/:tickId/address/:address/balance
pub(crate) async fn brc30_balance(
  Extension(index): Extension<Arc<Index>>,
  Path((tick_id, address)): Path<(String, String)>,
) -> ApiResult<BRC30Balance> {
  log::debug!(
    "rpc: get brc30_balance: tickId:{}, address:{}",
    tick_id,
    address
  );

  if tick_id.as_bytes().len() != 5 {
    return Err(ApiError::bad_request("tick id length must 5."));
  }
  let tick_id = tick_id.to_lowercase();
  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;
  let balance = &index
    .brc30_balance(&tick_id, &address)?
    .ok_or_api_not_found("pid not found")?;

  log::debug!(
    "rpc: get brc30_userinfo: {:?} {:?}",
    tick_id.as_str(),
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
    balance: all_balance
      .iter()
      .map(|(_tick_id, bal)| bal.into())
      .collect(),
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
      .map(|transfer| transfer.into())
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
    inscriptions: all.iter().map(|transer| transer.into()).collect(),
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
    events: all_receipt.iter().map(|receipt| receipt.into()).collect(),
    txid: txid.to_string(),
  })))
}

// 3.4.11 /brc30/block/:blockhash/events
pub(crate) async fn brc30_block_events(
  Extension(index): Extension<Arc<Index>>,
  Path(block_hash): Path<String>,
) -> ApiResult<BRC30BlockEvents> {
  log::debug!("rpc: get brc30_block_events: {}", block_hash);

  let hash =
    bitcoin::BlockHash::from_str(&block_hash).map_err(|e| ApiError::bad_request(e.to_string()))?;
  let block_event = index
    .brc30_block_events(&hash)?
    .ok_or_api_not_found("block not found")?;

  log::debug!("rpc: get brc30_block_events: {:?}", block_event);
  Ok(Json(ApiResponse::ok(BRC30BlockEvents {
    block: block_event
      .iter()
      .map(|(tx_id, receipt)| Events {
        events: receipt.iter().map(|e| e.into()).collect(),
        txid: tx_id.to_string(),
      })
      .collect(),
  })))
}
