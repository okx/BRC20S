use super::{brc20s_types::*, *};
use crate::okx::datastore::{
  brc20,
  brc20s::{self, Pid, PledgedTick, Receipt, TickId},
};
use axum::Json;

// brc20s/tick
pub(crate) async fn brc20s_all_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Query(page): Query<Pagination>,
) -> ApiResult<AllTickInfo> {
  log::debug!("rpc: get brc20s_all_tick_info");

  let (all_tick_info, total) = index.brc20s_all_tick_info(page.start.unwrap_or(0), page.limit)?;
  log::debug!("rpc: get brc20s_all_tick_info: {:?}", all_tick_info);

  Ok(Json(ApiResponse::ok(AllTickInfo {
    tokens: all_tick_info
      .iter()
      .map(|tick_info| {
        let inscription_number = &index
          .get_inscription_entry(tick_info.inscription_id)
          .unwrap()
          .unwrap();

        let mut brc20s_tick = TickInfo::from(tick_info);
        brc20s_tick.set_inscription_number(inscription_number.number as u64);
        brc20s_tick
      })
      .collect(),
    total,
  })))
}

// brc20s/tick/:tickId
pub(crate) async fn brc20s_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tick_id): Path<String>,
) -> ApiResult<TickInfo> {
  log::debug!("rpc: get brc20s_tick_info: {}", tick_id.to_string());

  let tick_id = TickId::from_str(tick_id.as_str())
    .map_err(|_| ApiError::bad_request(BRC20SError::IncorrectTickIdFormat))?;

  let tick_info = &index
    .brc20s_tick_info(&tick_id)?
    .ok_or_api_not_found(BRC20SError::TickIdNotFound)?;

  log::debug!("rpc: get brc20s_tick_info: {:?} {:?}", tick_id, tick_info);

  if tick_info.tick_id != tick_id {
    return Err(ApiError::internal("db: not match"));
  }

  let inscription_number = &index
    .get_inscription_entry(tick_info.inscription_id)
    .unwrap()
    .unwrap();

  let mut brc20s_tick = TickInfo::from(tick_info);
  brc20s_tick.set_inscription_number(inscription_number.number as u64);

  Ok(Json(ApiResponse::ok(brc20s_tick)))
}

// /brc20s/tick/:tickId
pub(crate) async fn brc20s_debug_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tick_id): Path<String>,
) -> ApiResult<brc20s::TickInfo> {
  log::debug!("rpc: get brc20s_debug_tick_info: {}", tick_id.to_string());

  let tick_id = TickId::from_str(&tick_id)
    .map_err(|_| ApiError::bad_request(BRC20SError::IncorrectTickIdFormat))?;

  let tick_info = index
    .brc20s_tick_info(&tick_id)?
    .ok_or_api_not_found(BRC20SError::TickIdNotFound)?;

  log::debug!(
    "rpc: get brc20s_debug_tick_info: {:?} {:?}",
    tick_id,
    tick_info
  );

  Ok(Json(ApiResponse::ok(tick_info)))
}

// brc20s/pool
pub(crate) async fn brc20s_all_pool_info(
  Extension(index): Extension<Arc<Index>>,
  Query(page): Query<Pagination>,
) -> ApiResult<AllPoolInfo> {
  log::debug!("rpc: get brc20s_all_pool_info");
  let (all_pool_info, total) = index.brc20s_all_pool_info(page.start.unwrap_or(0), page.limit)?;
  log::debug!("rpc: get brc20s_all_pool_info: {:?}", all_pool_info);
  Ok(Json(ApiResponse::ok(AllPoolInfo {
    tokens: all_pool_info
      .iter()
      .map(|pool| {
        let tick_id = TickId::from(pool.pid.clone());
        let tick_info = &index.brc20s_tick_info(&tick_id).unwrap().unwrap();

        let inscription_number = &index
          .get_inscription_entry(pool.inscription_id)
          .unwrap()
          .unwrap();

        let mut pool_result = Pool::from(pool);
        pool_result.set_earn(
          tick_info.tick_id.hex().to_string(),
          tick_info.name.as_str().to_string(),
        );
        pool_result.set_inscription_num(inscription_number.number as u64);
        pool_result.set_deployer(tick_info.deployer.clone().into());
        pool_result
      })
      .collect(),
    total,
  })))
}

// brc20s/pool/:pid
pub(crate) async fn brc20s_pool_info(
  Extension(index): Extension<Arc<Index>>,
  Path(pid): Path<String>,
) -> ApiResult<Pool> {
  log::debug!("rpc: get brc20s_pool_info: {}", pid);

  let pid =
    Pid::from_str(&pid).map_err(|_| ApiError::bad_request(BRC20SError::IncorrectPidFormat))?;

  let pool_info = &index
    .brc20s_pool_info(&pid)?
    .ok_or_api_not_found(BRC20SError::PoolInfoNotFound)?;

  log::debug!(
    "rpc: get brc20s_pool_info: {:?} {:?}",
    pid.as_str(),
    pool_info
  );

  if pool_info.pid != pid {
    return Err(ApiError::internal("db: not match"));
  }

  let tick_id = TickId::from(pid.clone());

  let tick_info = &index
    .brc20s_tick_info(&tick_id)?
    .ok_or_api_not_found(BRC20SError::TickIdNotFound)?;

  let inscription_number = &index
    .get_inscription_entry(pool_info.inscription_id)
    .unwrap()
    .unwrap();

  let mut pool = Pool::from(pool_info);
  pool.set_earn(
    tick_info.tick_id.hex().to_string(),
    tick_info.name.as_str().to_string(),
  );
  pool.set_inscription_num(inscription_number.number as u64);
  pool.set_deployer(tick_info.deployer.clone().into());

  Ok(Json(ApiResponse::ok(pool)))
}

pub(crate) async fn brc20s_debug_pool_info(
  Extension(index): Extension<Arc<Index>>,
  Path(pid): Path<String>,
) -> ApiResult<brc20s::PoolInfo> {
  log::debug!("rpc: get brc20s_debug_pool_info: {}", pid);

  let pid =
    Pid::from_str(&pid).map_err(|_| ApiError::bad_request(BRC20SError::IncorrectPidFormat))?;

  let pool_info = index
    .brc20s_pool_info(&pid)?
    .ok_or_api_not_found(BRC20SError::PoolInfoNotFound)?;

  log::debug!(
    "rpc: get brc20s_debug_pool_info: {:?} {:?}",
    pid.as_str(),
    pool_info
  );

  Ok(Json(ApiResponse::ok(pool_info)))
}

pub(crate) async fn brc20s_debug_stake_info(
  Extension(index): Extension<Arc<Index>>,
  Path((address, tick)): Path<(String, String)>,
) -> ApiResult<brc20s::StakeInfo> {
  log::debug!("rpc: get brc20s_debug_stake_info: {},{}", address, tick);

  let address: bitcoin::Address = address.parse().map_err(ApiError::bad_request)?;

  let stake_info = index
    .brc20s_stake_info(&address, &PledgedTick::from_str(tick.as_str()))?
    .ok_or_api_not_found(BRC20SError::StakeInfoNotFound)?;

  log::debug!("rpc: get brc20s_debug_stake_info: {:?}", stake_info);

  Ok(Json(ApiResponse::ok(stake_info)))
}

// brc20s/pool/:pid/address/:address/userinfo
pub(crate) async fn brc20s_userinfo(
  Extension(index): Extension<Arc<Index>>,
  Path((pid, address)): Path<(String, String)>,
) -> ApiResult<UserInfo> {
  log::debug!("rpc: get brc20s_userinfo: {}, {}", pid, address);

  let pid =
    Pid::from_str(&pid).map_err(|_| ApiError::bad_request(BRC20SError::IncorrectPidFormat))?;
  let address: bitcoin::Address = address.parse().map_err(ApiError::bad_request)?;
  let user_info = &index
    .brc20s_user_info(&pid, &address)?
    .ok_or_api_not_found(BRC20SError::UserInfoNotFound)?;

  log::debug!(
    "rpc: get brc20s_userinfo: {:?} {:?}",
    pid.as_str(),
    user_info
  );

  if user_info.pid != pid {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(user_info.into())))
}

pub(crate) async fn brc20s_user_pending_reward(
  Extension(index): Extension<Arc<Index>>,
  Path((pid, address)): Path<(String, String)>,
) -> ApiResult<UserReward> {
  log::debug!("rpc: get brc20s_user_pending_reward: {}, {}", pid, address);

  let pid =
    Pid::from_str(&pid).map_err(|_| ApiError::bad_request(BRC20SError::IncorrectPidFormat))?;
  let address: bitcoin::Address = address.parse().map_err(ApiError::bad_request)?;
  let (user_reward, block) = &index.brc20s_user_pending_reward(&pid, &address)?;

  log::debug!(
    "rpc: get brc20s_user_pending_reward: {:?}, {:?}, {:?}",
    pid.as_str(),
    user_reward,
    block,
  );

  Ok(Json(ApiResponse::ok(UserReward {
    pending_reward: user_reward.clone().unwrap(),
    block_num: block.clone().unwrap(),
  })))
}

// brc20s/debug/pool/:pid/address/:address/userinfo
pub(crate) async fn brc20s_debug_userinfo(
  Extension(index): Extension<Arc<Index>>,
  Path((pid, address)): Path<(String, String)>,
) -> ApiResult<brc20s::UserInfo> {
  log::debug!("rpc: get brc20s_debug_userinfo: {}, {}", pid, address);

  let pid =
    Pid::from_str(&pid).map_err(|_| ApiError::bad_request(BRC20SError::IncorrectPidFormat))?;
  let address: bitcoin::Address = address.parse().map_err(ApiError::bad_request)?;
  let user_info = index
    .brc20s_user_info(&pid, &address)?
    .ok_or_api_not_found(BRC20SError::UserInfoNotFound)?;

  log::debug!(
    "rpc: get brc20s_debug_userinfo: {:?} {:?}",
    pid.as_str(),
    user_info
  );

  if user_info.pid != pid {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(user_info)))
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

  let tick_id = TickId::from_str(&tick_id)
    .map_err(|_| ApiError::bad_request(BRC20SError::IncorrectTickIdFormat))?;
  let address: bitcoin::Address = address.parse().map_err(ApiError::bad_request)?;
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

  let tick_id = TickId::from_str(&tick_id)
    .map_err(|_| ApiError::bad_request(BRC20SError::IncorrectTickIdFormat))?;

  let address: bitcoin::Address = address.parse().map_err(ApiError::bad_request)?;
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

  let address: bitcoin::Address = address.parse().map_err(ApiError::bad_request)?;

  let all_balance = index.brc20s_all_balance(&address)?;

  log::debug!("rpc: get brc20s_all_balance: {} {:?}", address, all_balance);

  Ok(Json(ApiResponse::ok(AllBalance {
    balance: all_balance
      .iter()
      .map(|(tick_id, balance)| {
        let mut balance_result = Balance::from(balance);

        let tick_info = &index.brc20s_tick_info(&tick_id).unwrap().unwrap();

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

// brc20s/tick/:tickId/address/:address/transferable
pub(crate) async fn brc20s_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path((tick_id, address)): Path<(String, String)>,
) -> ApiResult<Transferable> {
  log::debug!("rpc: get brc20s_transferable: {},{}", tick_id, address);

  let tick_id = TickId::from_str(&tick_id)
    .map_err(|_| ApiError::bad_request(BRC20SError::IncorrectTickIdFormat))?;

  let address: bitcoin::Address = address.parse().map_err(ApiError::bad_request)?;
  let all_transfer = &index.brc20s_tickid_transferable(&tick_id, &address)?;

  log::debug!(
    "rpc: get brc20s_transferable: {:?} {:?}",
    tick_id.hex(),
    all_transfer
  );

  Ok(Json(ApiResponse::ok(Transferable {
    inscriptions: all_transfer
      .iter()
      .map(|asset| {
        let mut inscription = brc20s_types::Inscription::from(asset);

        let tick_info = &index.brc20s_tick_info(&asset.tick_id).unwrap().unwrap();

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

// brc20s/address/:address/transferable
pub(crate) async fn brc20s_all_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<Transferable> {
  log::debug!("rpc: get brc20s_all_transferable: {}", address);

  let address: bitcoin::Address = address.parse().map_err(ApiError::bad_request)?;

  let all = index.brc20s_all_transferable(&address)?;

  log::debug!("rpc: get brc20s_all_transferable: {} {:?}", address, all);

  Ok(Json(ApiResponse::ok(Transferable {
    inscriptions: all
      .iter()
      .map(|asset| {
        let mut inscription = brc20s_types::Inscription::from(asset);

        let tick_info = &index.brc20s_tick_info(&asset.tick_id).unwrap().unwrap();

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

// brc20s/tx/:txid/receipts
pub(crate) async fn brc20s_txid_receipts(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<TxReceipts> {
  log::debug!("rpc: get brc20s_txid_receipts: {}", txid);
  let txid = Txid::from_str(&txid).unwrap();

  let all_receipt = index
    .brc20s_txid_receipts(&txid)?
    .ok_or_api_not_found(BRC20SError::ReceiptsNotFound)?;

  log::debug!("rpc: get brc20s_txid_receipts: {:?}", all_receipt);

  let mut receipts = Vec::new();
  for receipt in all_receipt.iter() {
    match brc20s_types::Receipt::from(receipt, index.clone()) {
      Ok(receipt) => {
        receipts.push(receipt);
      }
      Err(_) => {
        return Err(ApiError::internal("failed to get transaction receipts"));
      }
    }
  }

  Ok(Json(ApiResponse::ok(TxReceipts {
    receipts,
    txid: txid.to_string(),
  })))
}

// brc20s/debug/tx/:txid/receipts
pub(crate) async fn brc20s_debug_txid_receipts(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<Vec<Receipt>> {
  log::debug!("rpc: get brc20s_debug_txid_receipts: {}", txid);
  let txid = Txid::from_str(&txid).unwrap();

  let all_receipt = index
    .brc20s_txid_receipts(&txid)?
    .ok_or_api_not_found(BRC20SError::ReceiptsNotFound)?;

  log::debug!("rpc: get brc20s_debug_txid_receipts: {:?}", all_receipt);

  Ok(Json(ApiResponse::ok(all_receipt)))
}

// brc20s/block/:blockhash/receipts
pub(crate) async fn brc20s_block_receipts(
  Extension(index): Extension<Arc<Index>>,
  Path(block_hash): Path<String>,
) -> ApiResult<BlockReceipts> {
  log::debug!("rpc: get brc20s_block_receipts: {}", block_hash);

  let block_hash = bitcoin::BlockHash::from_str(&block_hash).map_err(ApiError::bad_request)?;
  let block_receipts = index
    .brc20s_block_receipts(&block_hash)?
    .ok_or_api_not_found(BRC20SError::BlockReceiptsNotFound)?;

  log::debug!("rpc: get brc20s_block_receipts: {:?}", block_receipts);

  let mut api_block_receipts = Vec::new();
  for (txid, tx_receipts) in block_receipts.iter() {
    let mut api_tx_receipts = Vec::new();
    for receipt in tx_receipts.iter() {
      match brc20s_types::Receipt::from(receipt, index.clone()) {
        Ok(receipt) => {
          api_tx_receipts.push(receipt);
        }
        Err(error) => {
          return Err(ApiError::internal(format!(
            "failed to get transaction receipts for {txid}, error: {error}"
          )));
        }
      }
    }
    api_block_receipts.push(TxReceipts {
      receipts: api_tx_receipts,
      txid: txid.to_string(),
    });
  }

  Ok(Json(ApiResponse::ok(BlockReceipts {
    block: api_block_receipts,
  })))
}

// brc20s/stake/:address/:tick?tick_type=0
pub(crate) async fn brc20s_stake_info(
  Extension(index): Extension<Arc<Index>>,
  Path((address, tick)): Path<(String, String)>,
) -> ApiResult<StakedInfo> {
  log::debug!(
    "rpc: get brc20s_stake_info: tick:{}, address:{}",
    tick,
    address
  );

  let tick = brc20::Tick::from_str(&tick)
    .map_err(|_| ApiError::bad_request(brc20_types::BRC20Error::IncorrectTickFormat))?;

  let tick = index
    .brc20_get_tick_info(&tick)?
    .ok_or_api_not_found(brc20_types::BRC20Error::TickNotFound)?
    .tick;

  let address: bitcoin::Address = address.parse().map_err(ApiError::bad_request)?;

  let stake_info = index
    .brc20s_stake_info(&address, &PledgedTick::BRC20Tick(tick.clone()))?
    .ok_or_api_not_found(BRC20SError::StakeInfoNotFound)?;

  log::debug!("rpc: get brc20s_stake_info: {:?}", stake_info);

  let mut result = StakedInfo::from(&stake_info);
  result.tick = tick.to_string();

  Ok(Json(ApiResponse::ok(result)))
}
