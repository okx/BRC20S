use {super::*, crate::okx::datastore::brc20s, axum::Json};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::TickInfo)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TickInfo {
  /// Ticker.
  #[schema(value_type = brc20s::Tick)]
  pub tick: Tick,
  /// The inscription id.
  pub inscription_id: String,
  /// The inscription number.
  pub inscription_number: i64,
  /// The minted amount.
  #[schema(format = "uint64")]
  pub minted: String,
  /// The total supply.
  #[schema(format = "uint64")]
  pub supply: String,
  /// The decimal.
  pub decimal: u8,
  /// The deployer.
  pub deployer: ScriptPubkey,
  /// The transaction id.
  pub txid: String,
  /// The height of the block that the ticker deployed.
  #[schema(format = "uint64")]
  pub deploy_height: u64,
  /// The timestamp of the block that the ticker deployed.
  #[schema(format = "uint64")]
  pub deploy_blocktime: u32,
}

impl TickInfo {
  pub fn set_inscription_number(&mut self, inscription_number: i64) {
    self.inscription_number = inscription_number;
  }
}

impl From<&brc20s::TickInfo> for TickInfo {
  fn from(tick_info: &brc20s::TickInfo) -> Self {
    let tick = Tick {
      id: tick_info.tick_id.hex(),
      name: tick_info.name.as_str().to_string(),
    };

    Self {
      tick,
      inscription_id: tick_info.inscription_id.to_string(),
      inscription_number: 0,
      minted: tick_info.circulation.to_string(),
      supply: tick_info.supply.to_string(),
      decimal: tick_info.decimal,
      deployer: tick_info.deployer.clone().into(),
      txid: tick_info.inscription_id.txid.to_string(),
      deploy_height: tick_info.deploy_block,
      deploy_blocktime: tick_info.deploy_block_time,
    }
  }
}

// brc20s/tick/:tickId
#[utoipa::path(
  get,
  path = "/api/v1/brc20s/tick/{tick_id}",
  operation_id = "get ticker info",
  params(
    ("tick_id" = String, Path, description = "The ticker ID", min_length = 10, max_length = 10, example = "a12345678f")
),
  responses(
    (status = 200, description = "Obtain matching BRC20S ticker by query.", body = BRC20STick),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
pub(crate) async fn brc20s_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tick_id): Path<String>,
) -> ApiResult<TickInfo> {
  log::debug!("rpc: get brc20s_tick_info: {}", tick_id);

  let tick_id = brc20s::TickId::from_str(tick_id.as_str())
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
  brc20s_tick.set_inscription_number(inscription_number.number);

  Ok(Json(ApiResponse::ok(brc20s_tick)))
}

// /brc20s/tick/:tickId
pub(crate) async fn brc20s_debug_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tick_id): Path<String>,
) -> ApiResult<brc20s::TickInfo> {
  log::debug!("rpc: get brc20s_debug_tick_info: {}", tick_id);

  let tick_id = brc20s::TickId::from_str(&tick_id)
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::AllTickInfo)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AllTickInfo {
  #[schema(value_type = Vec<brc20s::TickInfo>)]
  pub tokens: Vec<TickInfo>,
  pub total: usize,
}

// brc20s/tick
#[utoipa::path(
  get,
  path = "/api/v1/brc20s/tick",
  operation_id = "get all tickers info",
  params(
    Pagination
),
  responses(
    (status = 200, description = "Obtain matching all BRC20S tickers.", body = BRC20SAllTick),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
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
        brc20s_tick.set_inscription_number(inscription_number.number);
        brc20s_tick
      })
      .collect(),
    total,
  })))
}
