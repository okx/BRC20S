use {super::*, crate::okx::datastore::brc20 as brc20_store, axum::Json, utoipa::ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum TxEvent {
  Deploy(DeployEvent),
  Mint(MintEvent),
  InscribeTransfer(InscribeTransferEvent),
  Transfer(TransferEvent),
  Error(ErrorEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeployEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub supply: String,
  pub limit_per_mint: String,
  pub decimal: u8,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MintEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub amount: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InscribeTransferEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub amount: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransferEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub amount: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

impl From<&brc20_store::Receipt> for TxEvent {
  fn from(event: &brc20_store::Receipt) -> Self {
    match &event.result {
      Ok(brc20_store::Event::Deploy(deploy_event)) => Self::Deploy(DeployEvent {
        tick: deploy_event.tick.to_string(),
        inscription_id: event.inscription_id.to_string(),
        inscription_number: event.inscription_number,
        old_satpoint: event.old_satpoint.to_string(),
        new_satpoint: event.new_satpoint.to_string(),
        supply: deploy_event.supply.to_string(),
        limit_per_mint: deploy_event.limit_per_mint.to_string(),
        decimal: deploy_event.decimal,
        from: event.from.clone().into(),
        to: event.to.clone().into(),
        valid: true,
        msg: "ok".to_string(),
        event: "deploy".to_string(),
      }),
      Ok(brc20_store::Event::Mint(mint_event)) => Self::Mint(MintEvent {
        tick: mint_event.tick.to_string(),
        inscription_id: event.inscription_id.to_string(),
        inscription_number: event.inscription_number,
        old_satpoint: event.old_satpoint.to_string(),
        new_satpoint: event.new_satpoint.to_string(),
        amount: mint_event.amount.to_string(),
        from: event.from.clone().into(),
        to: event.to.clone().into(),
        valid: true,
        msg: mint_event.msg.clone().unwrap_or("ok".to_string()),
        event: "mint".to_string(),
      }),
      Ok(brc20_store::Event::InscribeTransfer(trans1)) => {
        Self::InscribeTransfer(InscribeTransferEvent {
          tick: trans1.tick.to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: event.inscription_number,
          old_satpoint: event.old_satpoint.to_string(),
          new_satpoint: event.new_satpoint.to_string(),
          amount: trans1.amount.to_string(),
          from: event.from.clone().into(),
          to: event.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
          event: "inscribeTransfer".to_string(),
        })
      }
      Ok(brc20_store::Event::Transfer(trans2)) => Self::Transfer(TransferEvent {
        tick: trans2.tick.to_string(),
        inscription_id: event.inscription_id.to_string(),
        inscription_number: event.inscription_number,
        old_satpoint: event.old_satpoint.to_string(),
        new_satpoint: event.new_satpoint.to_string(),
        amount: trans2.amount.to_string(),
        from: event.from.clone().into(),
        to: event.to.clone().into(),
        valid: true,
        msg: trans2.msg.clone().unwrap_or("ok".to_string()),
        event: "transfer".to_string(),
      }),
      Err(err) => Self::Error(ErrorEvent {
        inscription_id: event.inscription_id.to_string(),
        inscription_number: event.inscription_number,
        old_satpoint: event.old_satpoint.to_string(),
        new_satpoint: event.new_satpoint.to_string(),
        valid: false,
        from: event.from.clone().into(),
        to: event.to.clone().into(),
        msg: err.to_string(),
        event: match event.op {
          brc20_store::OperationType::Deploy => "deploy".to_string(),
          brc20_store::OperationType::Mint => "mint".to_string(),
          brc20_store::OperationType::InscribeTransfer => "inscribeTransfer".to_string(),
          brc20_store::OperationType::Transfer => "transfer".to_string(),
        },
      }),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TxEvents {
  pub events: Vec<TxEvent>,
  pub txid: String,
}

#[utoipa::path(
    get,
    path = "/brc20/tx/{txid}/events",
    operation_id = "get transaction events by txid",
    params(
        ("txid" = String, Path, description = "transaction ID")
  ),
    responses(
      (status = 200, description = "Obtain transaction events by txid", body = BRC20TxEventsResponse),
      (status = 400, description = "Bad query.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::bad_request(BRC20Error::IncorrectTickFormat)))),
      (status = 404, description = "Ticker not found.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::not_found(BRC20Error::TickNotFound)))),
      (status = 500, description = "Internal server error.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::internal("internal error")))),
    )
  )]
pub(crate) async fn brc20_tx_events(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<TxEvents> {
  log::debug!("rpc: get brc20_tx_events: {}", txid);
  let txid = bitcoin::Txid::from_str(&txid).map_err(|e| ApiError::bad_request(e.to_string()))?;
  let tx_events = index
    .brc20_get_tx_events_by_txid(&txid)?
    .ok_or_api_not_found(BRC20Error::EventsNotFound)?;

  log::debug!("rpc: get brc20_tx_events: {} {:?}", txid, tx_events);

  Ok(Json(ApiResponse::ok(TxEvents {
    txid: txid.to_string(),
    events: tx_events.iter().map(|e| e.into()).collect(),
  })))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockEvents {
  pub block: Vec<TxEvents>,
}
#[utoipa::path(
    get,
    path = "/brc20/block/{blockhash}/events",
    operation_id = "get block events by blockhash",
    params(
        ("blockhash" = String, Path, description = "block hash")
  ),
    responses(
      (status = 200, description = "Obtain block events by block hash", body = BRC20BlockEventsResponse),
      (status = 400, description = "Bad query.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::bad_request(BRC20Error::IncorrectTickFormat)))),
      (status = 404, description = "Ticker not found.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::not_found(BRC20Error::TickNotFound)))),
      (status = 500, description = "Internal server error.", body = ApiErrorResponse, example = json!(ApiErrorResponse::api_err(&ApiError::internal("internal error")))),
    )
  )]
pub(crate) async fn brc20_block_events(
  Extension(index): Extension<Arc<Index>>,
  Path(block_hash): Path<String>,
) -> ApiResult<BlockEvents> {
  log::debug!("rpc: get brc20_block_events: {}", block_hash);

  let blockhash =
    bitcoin::BlockHash::from_str(&block_hash).map_err(|e| ApiError::bad_request(e.to_string()))?;

  let block_events = index
    .brc20_get_block_events_by_blockhash(blockhash)?
    .ok_or_api_not_found(BRC20Error::BlockNotFound)?;

  log::debug!(
    "rpc: get brc20_block_events: {} {:?}",
    block_hash,
    block_events
  );

  Ok(Json(ApiResponse::ok(BlockEvents {
    block: block_events
      .iter()
      .map(|(txid, events)| TxEvents {
        txid: txid.to_string(),
        events: events.iter().map(|e| e.into()).collect(),
      })
      .collect(),
  })))
}
