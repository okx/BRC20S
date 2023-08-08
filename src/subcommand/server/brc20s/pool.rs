use {
  super::*,
  axum::Json,
  brc20s::{Pid, PoolInfo, TickId},
};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Pool {
  pub pid: String,
  pub stake: Stake,
  pub earn: Earn,
  pub pool: String,
  pub erate: String,
  pub staked: String,
  pub minted: String,
  pub dmax: String,
  pub only: u8,
  pub acc_reward_per_share: String,
  pub latest_update_block: u64,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub deployer: ScriptPubkey,
  pub deploy_height: u64,
  pub deploy_blocktime: u64,
  pub txid: String,
}

impl Pool {
  pub fn set_earn(&mut self, earn_id: String, earn_name: String) {
    self.earn.id = earn_id;
    self.earn.name = earn_name;
  }

  pub fn set_inscription_num(&mut self, inscription_number: i64) {
    self.inscription_number = inscription_number
  }

  pub fn set_deployer(&mut self, deployer: ScriptPubkey) {
    self.deployer = deployer;
  }
}

impl From<&PoolInfo> for Pool {
  fn from(pool_info: &PoolInfo) -> Self {
    let stake = Stake {
      type_field: pool_info.stake.to_type(),
      tick: pool_info.stake.to_string(),
    };

    let earn = Earn {
      id: "".to_string(),
      name: "".to_string(),
    };

    Self {
      pid: pool_info.pid.as_str().to_string(),
      stake,
      earn,
      pool: pool_info.ptype.to_string(),
      staked: pool_info.staked.to_string(),
      erate: pool_info.erate.to_string(),
      minted: pool_info.minted.to_string(),
      dmax: pool_info.dmax.to_string(),
      only: if pool_info.only { 1 } else { 0 },
      acc_reward_per_share: pool_info.acc_reward_per_share.to_string(),
      latest_update_block: pool_info.last_update_block,
      inscription_id: pool_info.inscription_id.to_string(),
      inscription_number: 0,
      deployer: ScriptPubkey::default(),
      deploy_height: pool_info.deploy_block,
      deploy_blocktime: u64::from(pool_info.deploy_block_time),
      txid: pool_info.inscription_id.txid.to_string(),
    }
  }
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

  let tick_id = TickId::from(pid);

  let tick_info = &index
    .brc20s_tick_info(&tick_id)?
    .ok_or_api_not_found(BRC20SError::TickIdNotFound)?;

  let inscription_number = &index
    .get_inscription_entry(pool_info.inscription_id)
    .unwrap()
    .unwrap();

  let mut pool = Pool::from(pool_info);
  pool.set_earn(tick_info.tick_id.hex(), tick_info.name.as_str().to_string());
  pool.set_inscription_num(inscription_number.number);
  pool.set_deployer(tick_info.deployer.clone().into());

  Ok(Json(ApiResponse::ok(pool)))
}

pub(crate) async fn brc20s_debug_pool_info(
  Extension(index): Extension<Arc<Index>>,
  Path(pid): Path<String>,
) -> ApiResult<PoolInfo> {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AllPoolInfo {
  pub pools: Vec<Pool>,
  pub total: usize,
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
    pools: all_pool_info
      .iter()
      .map(|pool| {
        let tick_id = TickId::from(pool.pid.clone());
        let tick_info = &index.brc20s_tick_info(&tick_id).unwrap().unwrap();

        let inscription_number = &index
          .get_inscription_entry(pool.inscription_id)
          .unwrap()
          .unwrap();

        let mut pool_result = Pool::from(pool);
        pool_result.set_earn(tick_info.tick_id.hex(), tick_info.name.as_str().to_string());
        pool_result.set_inscription_num(inscription_number.number);
        pool_result.set_deployer(tick_info.deployer.clone().into());
        pool_result
      })
      .collect(),
    total,
  })))
}

// /brc20s/pool/:tick_id
pub(crate) async fn brc20s_all_pools_by_tid(
  Extension(index): Extension<Arc<Index>>,
  Path(tick_id): Path<String>,
) -> ApiResult<AllPoolInfo> {
  log::debug!("rpc: get brc20s_all_pools_by_tid: {}", tick_id);

  let tick_id = TickId::from_str(&tick_id)
    .map_err(|_| ApiError::bad_request(BRC20SError::IncorrectTickIdFormat))?;
  let all_pool_info = index.brc20s_all_pools_by_tid(&tick_id)?;

  let _ = index
    .brc20s_tick_info(&tick_id)?
    .ok_or_api_not_found(BRC20SError::TickIdNotFound)?;

  log::debug!("rpc: get brc20s_all_pools_by_tid: {:?}", all_pool_info);
  Ok(Json(ApiResponse::ok(AllPoolInfo {
    pools: all_pool_info
      .iter()
      .map(|pool| {
        let tick_id = TickId::from(pool.pid.clone());
        let tick_info = &index.brc20s_tick_info(&tick_id).unwrap().unwrap();

        let inscription_number = &index
          .get_inscription_entry(pool.inscription_id)
          .unwrap()
          .unwrap();

        let mut pool_result = Pool::from(pool);
        pool_result.set_earn(tick_info.tick_id.hex(), tick_info.name.as_str().to_string());
        pool_result.set_inscription_num(inscription_number.number);
        pool_result.set_deployer(tick_info.deployer.clone().into());
        pool_result
      })
      .collect(),
    total: all_pool_info.len(),
  })))
}
