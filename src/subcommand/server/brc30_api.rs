use super::brc30_types::*;
use super::error::ApiError;
use super::*;
use crate::okx::datastore::brc30;
use crate::okx::datastore::brc30::{BRC30Receipt, PledgedTick, TickId};
use axum::Json;

// 3.4.1 /brc30/tick
pub(crate) async fn brc30_all_tick_info(
  Extension(index): Extension<Arc<Index>>,
) -> ApiResult<AllBRC30TickInfo> {
  log::debug!("rpc: get brc30_all_tick_info");

  let all_tick_info = index.brc30_all_tick_info()?;
  log::debug!("rpc: get brc30_all_tick_info: {:?}", all_tick_info);

  Ok(Json(ApiResponse::ok(AllBRC30TickInfo {
    tokens: all_tick_info
      .iter()
      .map(|tick_info| {
        let inscription_number = &index
          .get_inscription_entry(tick_info.inscription_id)
          .unwrap()
          .unwrap();

        let block = &index
          .get_block_by_height(tick_info.deploy_block)
          .unwrap()
          .unwrap();

        let mut brc30_tick = BRC30TickInfo::from(tick_info);
        brc30_tick.set_deploy_blocktime(block.header.time as u64);
        brc30_tick.set_inscription_number(inscription_number.number as u64);
        brc30_tick
      })
      .collect(),
  })))
}

// 3.4.2 /brc30/tick/:tickId
pub(crate) async fn brc30_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tick_id): Path<String>,
) -> ApiResult<BRC30TickInfo> {
  log::debug!("rpc: get brc30_tick_info: {}", tick_id.to_string());

  let tick_id = tick_id.to_lowercase();
  match TickId::from_str(tick_id.as_str()) {
    Ok(_) => {}
    Err(error) => {
      return Err(ApiError::Internal(error.to_string()));
    }
  }

  let tick_info = &index
    .brc30_tick_info(&tick_id)?
    .ok_or_api_not_found("tick not found")?;

  log::debug!("rpc: get brc30_tick_info: {:?} {:?}", tick_id, tick_info);

  if tick_info.tick_id != brc30::TickId::from_str(&tick_id).unwrap() {
    return Err(ApiError::internal("db: not match"));
  }

  let inscription_number = &index
    .get_inscription_entry(tick_info.inscription_id)
    .unwrap()
    .unwrap();

  let block = &index
    .get_block_by_height(tick_info.deploy_block)
    .unwrap()
    .unwrap();

  let mut brc30_tick = BRC30TickInfo::from(tick_info);
  brc30_tick.set_deploy_blocktime(block.header.time as u64);
  brc30_tick.set_inscription_number(inscription_number.number as u64);

  Ok(Json(ApiResponse::ok(brc30_tick)))
}

// 3.4.2 /brc30/tick/:tickId
pub(crate) async fn brc30_debug_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tick_id): Path<String>,
) -> ApiResult<brc30::TickInfo> {
  log::debug!("rpc: get brc30_tick_info: {}", tick_id.to_string());
  let tick_id = tick_id.to_lowercase();
  match TickId::from_str(tick_id.as_str()) {
    Ok(_) => {}
    Err(error) => {
      return Err(ApiError::Internal(error.to_string()));
    }
  }

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
    tokens: all_pool_info
      .iter()
      .map(|pool| {
        let split_string: Vec<&str> = pool.pid.as_str().split("#").collect();
        let tick_id = split_string[0];

        let tick_info = &index
          .brc30_tick_info(&tick_id.to_string())
          .unwrap()
          .unwrap();
        let block = &index
          .get_block_by_height(tick_info.deploy_block)
          .unwrap()
          .unwrap();

        let inscription_number = &index
          .get_inscription_entry(pool.inscription_id)
          .unwrap()
          .unwrap();

        let mut pool_result = BRC30Pool::from(pool);
        pool_result.set_earn(
          tick_info.tick_id.hex().to_string(),
          tick_info.name.as_str().to_string(),
        );
        pool_result.set_inscription_num(inscription_number.number as u64);
        pool_result.set_deploy(
          tick_info.deployer.to_string(),
          tick_info.deploy_block,
          block.header.time as u64,
        );
        pool_result
      })
      .collect(),
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

  let split_string: Vec<&str> = pid.split("#").collect();
  let tick_id = split_string[0];

  let tick_info = &index
    .brc30_tick_info(&tick_id.to_string())?
    .ok_or_api_not_found("tick not found")?;
  let block = &index
    .get_block_by_height(tick_info.deploy_block)
    .unwrap()
    .unwrap();

  let inscription_number = &index
    .get_inscription_entry(pool_info.inscription_id)
    .unwrap()
    .unwrap();

  let mut pool = BRC30Pool::from(pool_info);
  pool.set_earn(
    tick_info.tick_id.hex().to_string(),
    tick_info.name.as_str().to_string(),
  );
  pool.set_inscription_num(inscription_number.number as u64);
  pool.set_deploy(
    tick_info.deployer.to_string(),
    tick_info.deploy_block,
    block.header.time as u64,
  );

  Ok(Json(ApiResponse::ok(pool)))
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

  let stake_info = index
    .brc30_stake_info(&address, &PledgedTick::from_str(tick.as_str()))?
    .ok_or_api_not_found("pid not found")?;

  log::debug!("rpc: get brc30_pool_info: {:?}", stake_info);

  Ok(Json(ApiResponse::ok(stake_info)))
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

pub(crate) async fn brc30_user_reward(
  Extension(index): Extension<Arc<Index>>,
  Path((pid, address)): Path<(String, String)>,
) -> ApiResult<UserReward> {
  log::debug!("rpc: get brc30_user_reward: {}, {}", pid, address);

  if pid.as_bytes().len() != 13 {
    return Err(ApiError::bad_request("pid length must 13."));
  }
  let pid = pid.to_lowercase();
  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;
  let (user_reward, block) = &index.brc30_user_reward(&pid, &address)?;

  log::debug!(
    "rpc: get brc30_user_reward: {:?}, {:?}, {:?}",
    pid.as_str(),
    user_reward,
    block,
  );

  Ok(Json(ApiResponse::ok(UserReward {
    user_reward: user_reward.clone().unwrap(),
    block_num: block.clone().unwrap(),
  })))
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

  match TickId::from_str(tick_id.as_str()) {
    Ok(_) => {}
    Err(error) => {
      return Err(ApiError::Internal(error.to_string()));
    }
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

  let tick_id = tick_id.to_lowercase();
  match TickId::from_str(tick_id.as_str()) {
    Ok(_) => {}
    Err(error) => {
      return Err(ApiError::Internal(error.to_string()));
    }
  }

  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;
  let balance = &index
    .brc30_balance(&tick_id, &address)?
    .ok_or_api_not_found("pid not found")?;

  let mut balance_result = BRC30Balance::from(balance);

  let tick_info = &index
    .brc30_tick_info(&tick_id.to_string())?
    .ok_or_api_not_found("tick not found")?;

  balance_result.set_tick_name(tick_info.name.as_str().to_string());
  log::debug!(
    "rpc: get brc30_userinfo: {:?} {:?}",
    tick_id.as_str(),
    balance_result
  );

  Ok(Json(ApiResponse::ok(balance_result)))
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
      .map(|(tick_id, balance)| {
        let mut balance_result = BRC30Balance::from(balance);

        let tick_info = &index
          .brc30_tick_info(&tick_id.hex().to_string())
          .unwrap()
          .unwrap();

        balance_result.set_tick_name(tick_info.name.as_str().to_string());
        log::debug!(
          "rpc: get brc30_userinfo: {:?} {:?}",
          tick_id,
          balance_result
        );
        balance_result
      })
      .collect(),
  })))
}

// 3.4.8 /brc30/tick/:tickId/address/:address/transferable
pub(crate) async fn brc30_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path((tick_id, address)): Path<(String, String)>,
) -> ApiResult<Transferable> {
  log::debug!("rpc: get brc30_transferable: {},{}", tick_id, address);

  let tick_id = tick_id.to_lowercase();
  match TickId::from_str(tick_id.as_str()) {
    Ok(_) => {}
    Err(error) => {
      return Err(ApiError::Internal(error.to_string()));
    }
  }

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
      .map(|asset| {
        let mut inscription = Brc30Inscription::from(asset);

        let tick_info = &index
          .brc30_tick_info(&asset.tick_id.hex().to_string())
          .unwrap()
          .unwrap();

        let inscription_number = &index
          .get_inscription_entry(asset.inscription_id)
          .unwrap()
          .unwrap();

        inscription.set_tick_name(tick_info.name.as_str().to_string());
        inscription.set_inscription_number(inscription_number.number as u64);
        inscription
      })
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
    inscriptions: all
      .iter()
      .map(|asset| {
        let mut inscription = Brc30Inscription::from(asset);

        let tick_info = &index
          .brc30_tick_info(&asset.tick_id.hex().to_string())
          .unwrap()
          .unwrap();

        let inscription_number = &index
          .get_inscription_entry(asset.inscription_id)
          .unwrap()
          .unwrap();

        inscription.set_tick_name(tick_info.name.as_str().to_string());
        inscription.set_inscription_number(inscription_number.number as u64);
        inscription
      })
      .collect(),
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
    events: all_receipt
      .iter()
      .map(|receipt| {
        let mut event = Brc30Event::from(receipt);
        let pid = match event.clone() {
          Brc30Event::DeployPool(d) => d.pid,
          _ => "".to_string(),
        };

        if pid.len() > 0 {
          let parts: Vec<&str> = pid.split("#").collect();
          let tick_info = &index
            .brc30_tick_info(&parts[0].to_string())
            .unwrap()
            .unwrap();

          event.set_earn(
            tick_info.tick_id.hex().to_string(),
            tick_info.name.as_str().to_string(),
          );

          let pool_info = &index.brc30_pool_info(&pid).unwrap().unwrap();
          event.set_only(if pool_info.only { 1 } else { 0 });
        }

        event
      })
      .collect(),
    txid: txid.to_string(),
  })))
}

//  /brc30/debug/tx/:txid/events
pub(crate) async fn brc30_debug_txid_events(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<Vec<BRC30Receipt>> {
  log::debug!("rpc: get brc30_txid_events: {}", txid);
  let txid = Txid::from_str(&txid).unwrap();

  let all_receipt = index.brc30_txid_events(&txid)?;

  log::debug!("rpc: get brc30_txid_events: {:?}", all_receipt);

  Ok(Json(ApiResponse::ok(all_receipt)))
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
        events: receipt
          .iter()
          .map(|receipt| {
            let mut event = Brc30Event::from(receipt);
            let pid = match event.clone() {
              Brc30Event::DeployPool(d) => d.pid,
              _ => "".to_string(),
            };

            if pid.len() > 0 {
              let parts: Vec<&str> = pid.split("#").collect();
              let tick_info = &index
                .brc30_tick_info(&parts[0].to_string())
                .unwrap()
                .unwrap();

              event.set_earn(
                tick_info.tick_id.hex().to_string(),
                tick_info.name.as_str().to_string(),
              );

              let pool_info = &index.brc30_pool_info(&pid).unwrap().unwrap();
              event.set_only(if pool_info.only { 1 } else { 0 });
            }

            event
          })
          .collect(),
        txid: tx_id.to_string(),
      })
      .collect(),
  })))
}

pub(crate) async fn brc30_debug_block_events(
  Extension(index): Extension<Arc<Index>>,
  Path(num): Path<String>,
) -> ApiResult<BRC30BlockEvents> {
  log::debug!("rpc: get brc30_block_events: {}", num);

  let block = index
    .get_block_by_height(num.parse::<u64>().unwrap())?
    .ok_or_api_not_found("block not found")?;

  let hash = bitcoin::BlockHash::from_str(&block.block_hash().to_string().as_str())
    .map_err(|e| ApiError::bad_request(e.to_string()))?;
  let block_event = index
    .brc30_block_events(&hash)?
    .ok_or_api_not_found("block not found")?;

  log::debug!("rpc: get block hash: {:?}", hash.to_string());
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
