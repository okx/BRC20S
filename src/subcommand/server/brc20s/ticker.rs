use {super::*, crate::okx::datastore::brc20s, axum::Json};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TickInfo {
  pub tick: Tick,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub minted: String,
  pub supply: String,
  pub decimal: u64,
  pub deployer: ScriptPubkey,
  pub txid: String,
  pub deploy_height: u64,
  pub deploy_blocktime: u64,
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
      decimal: u64::from(tick_info.decimal),
      deployer: tick_info.deployer.clone().into(),
      txid: tick_info.inscription_id.txid.to_string(),
      deploy_height: tick_info.deploy_block,
      deploy_blocktime: u64::from(tick_info.deploy_block_time),
    }
  }
}

// brc20s/tick/:tickId
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AllTickInfo {
  pub tokens: Vec<TickInfo>,
  pub total: usize,
}

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
        brc20s_tick.set_inscription_number(inscription_number.number);
        brc20s_tick
      })
      .collect(),
    total,
  })))
}
