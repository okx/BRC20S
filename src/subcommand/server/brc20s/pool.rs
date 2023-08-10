use {
  super::*,
  axum::Json,
  brc20s::{Pid, PoolInfo, TickId},
};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::Pool)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Pool {
  /// Pool id.
  pub pid: String,
  /// Stake ticker info.
  #[schema(value_type = brc20s::Stake)]
  pub stake: Stake,
  /// Earn ticker info.
  #[schema(value_type = brc20s::Earn)]
  pub earn: Earn,
  /// Pool type. Such as "pool", "fixed".
  pub pool: String,
  /// Mining rate.
  pub erate: String,
  /// The amount of the ticker that has been staked.
  pub staked: String,
  /// The amount of the ticker that has been minted.
  pub minted: String,
  /// The total supply of the ticker.
  pub dmax: String,
  /// Whether the pool is exclusive.
  pub only: u8,
  /// The accumulated reward per share.
  pub acc_reward_per_share: String,
  /// The latest update block number.
  #[schema(format = "uint64")]
  pub latest_update_block: u64,
  /// Inscription ID of the ticker deployed.
  pub inscription_id: String,
  /// Inscription number of the ticker deployed.
  pub inscription_number: i64,
  /// The deployer of the ticker deployed.
  pub deployer: ScriptPubkey,
  /// The height of the block that the ticker deployed.
  #[schema(format = "uint64")]
  pub deploy_height: u64,
  /// The timestamp of the block that the ticker deployed.
  #[schema(format = "uint32")]
  pub deploy_blocktime: u32,
  /// A hex encoded 32 byte transaction ID that the ticker deployed.
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
      deploy_blocktime: pool_info.deploy_block_time,
      txid: pool_info.inscription_id.txid.to_string(),
    }
  }
}

// brc20s/pool/:pid
#[utoipa::path(
  get,
  path = "/api/v1/brc20s/pool/{pid}",
  operation_id = "get the pool infomation by pid",
  params(
      ("pid" = String, Path, description = "Pool ID", min_length = 13, max_length = 13, example= "a01234567f#0f"),
),
  responses(
    (status = 200, description = "Obtain pool infomation by pid", body = BRC20SPool),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::AllPoolInfo)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AllPoolInfo {
  #[schema(value_type = Vec<brc20s::Pool>)]
  pub pools: Vec<Pool>,
  pub total: usize,
}

// brc20s/pool
#[utoipa::path(
  get,
  path = "/api/v1/brc20s/pool",
  operation_id = "get the all of pool infomations",
  params(
    Pagination
),
  responses(
    (status = 200, description = "Obtain all of pool infomations", body = BRC20SAllPool),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
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
#[utoipa::path(
  get,
  path = "/api/v1/brc20s/pool/{tid}",
  operation_id = "get the pool infomation by ticker id",
  params(
      ("tid" = String, Path, description = "Ticker ID", min_length = 10, max_length = 10, example= "a01234567f"),
),
  responses(
    (status = 200, description = "Obtain pool infomation by ticker ID", body = BRC20SAllPool),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
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
