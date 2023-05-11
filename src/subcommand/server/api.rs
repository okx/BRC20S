use super::error::ApiError;
use super::*;
use axum::Json;
use log::log;

const ERR_TICK_LENGTH: &str = "tick must be 4 bytes length";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickInfo {
  pub tick: String,
  pub inscription_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscription_number: Option<String>,
  pub supply: String,
  pub limit_per_mint: String,
  pub minted: String,
  pub decimal: u64,
  pub deploy_by: String,
  pub txid: String,
  pub deploy_height: u64,
  pub deploy_blocktime: u64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllTickInfo {
  pub tokens: Vec<TickInfo>,
}

impl From<&brc20::TokenInfo> for TickInfo {
  fn from(tick_info: &brc20::TokenInfo) -> Self {
    Self {
      tick: std::str::from_utf8(tick_info.tick.as_bytes())
        .unwrap()
        .to_string(),
      inscription_id: tick_info.inscription_id.to_string(),
      inscription_number: None,
      supply: tick_info.supply.to_string(),
      limit_per_mint: tick_info.limit_per_mint.to_string(),
      minted: tick_info.minted.to_string(),
      decimal: tick_info.decimal as u64,
      deploy_by: tick_info.deploy_by.to_string(),
      txid: tick_info.inscription_id.txid.to_string(),
      deploy_height: tick_info.deployed_number,
      deploy_blocktime: tick_info.deployed_timestamp as u64,
    }
  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
  pub tick: String,
  pub available_balance: String,
  pub transferable_balance: String,
  pub overall_balance: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllBalance {
  pub balance: Vec<Balance>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxEvents {
  pub events: Vec<TxEvent>,
  pub txid: String,
}

impl From<&brc20::ActionReceipt> for TxEvent {
  fn from(event: &brc20::ActionReceipt) -> Self {
    match &event.result {
      Ok(result) => match result {
        brc20::BRC20Event::Deploy(deploy_event) => Self::Deploy(DeployEvent {
          tick: std::str::from_utf8(deploy_event.tick.as_bytes())
            .unwrap()
            .to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: None,
          supply: deploy_event.supply.to_string(),
          limit_per_mint: deploy_event.limit_per_mint.to_string(),
          decimal: deploy_event.decimal as u64,
          msg_sender: event.from.to_string(),
          deploy_by: event.to.to_string(),
          valid: true,
          msg: "ok".to_string(),
        }),
        brc20::BRC20Event::Mint(mint_event) => Self::Mint(MintEvent {
          tick: std::str::from_utf8(mint_event.tick.as_bytes())
            .unwrap()
            .to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: None,
          amount: mint_event.amount.to_string(),
          msg_sender: event.from.to_string(),
          to: event.to.to_string(),
          valid: true,
          msg: mint_event.msg.clone().unwrap_or("ok".to_string()),
        }),
        brc20::BRC20Event::TransferPhase1(trans1) => {
          Self::InscribeTransfer(InscribeTransferEvent {
            tick: std::str::from_utf8(trans1.tick.as_bytes())
              .unwrap()
              .to_string(),
            inscription_id: event.inscription_id.to_string(),
            inscription_number: None,
            amount: trans1.amount.to_string(),
            msg_sender: event.from.to_string(),
            owner: event.to.to_string(),
            valid: true,
            msg: "ok".to_string(),
          })
        }
        brc20::BRC20Event::TransferPhase2(trans2) => Self::Transfer(TransferEvent {
          tick: std::str::from_utf8(trans2.tick.as_bytes())
            .unwrap()
            .to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: None,
          amount: trans2.amount.to_string(),
          from: event.from.to_string(),
          to: event.to.to_string(),
          valid: true,
          msg: trans2.msg.clone().unwrap_or("ok".to_string()),
        }),
      },
      Err(err) => Self::Error(ErrorEvent {
        inscription_id: event.inscription_id.to_string(),
        inscription_number: None,
        valid: false,
        msg: err.to_string(),
      }),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
#[serde(rename_all = "camelCase")]
pub enum TxEvent {
  Deploy(DeployEvent),
  Mint(MintEvent),
  InscribeTransfer(InscribeTransferEvent),
  Transfer(TransferEvent),
  Error(ErrorEvent),
}

impl TxEvent {
  pub fn set_inscription_number(&mut self, number: Option<String>) {
    match self {
      TxEvent::Deploy(event) => {
        event.inscription_number = number;
      }
      TxEvent::Mint(event) => {
        event.inscription_number = number;
      }
      TxEvent::InscribeTransfer(event) => {
        event.inscription_number = number;
      }
      TxEvent::Transfer(event) => {
        event.inscription_number = number;
      }
      TxEvent::Error(event) => {
        event.inscription_number = number;
      }
    }
  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
  pub inscription_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscription_number: Option<String>,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployEvent {
  pub tick: String,
  pub inscription_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscription_number: Option<String>,
  pub supply: String,
  pub limit_per_mint: String,
  pub decimal: u64,
  pub msg_sender: String,
  pub deploy_by: String,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintEvent {
  pub tick: String,
  pub inscription_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscription_number: Option<String>,
  pub amount: String,
  pub msg_sender: String,
  pub to: String,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscribeTransferEvent {
  pub tick: String,
  pub inscription_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscription_number: Option<String>,
  pub amount: String,
  pub msg_sender: String,
  pub owner: String,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferEvent {
  pub tick: String,
  pub inscription_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscription_number: Option<String>,
  pub amount: String,
  pub from: String,
  pub to: String,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockEvents {
  pub block: Vec<TxEvents>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferableInscriptions {
  pub inscriptions: Vec<TransferableInscription>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferableInscription {
  pub id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub number: Option<String>,
  pub amount: String,
  pub tick: String,
}

pub(crate) async fn brc20_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tick): Path<String>,
) -> Json<ApiResponse<TickInfo>> {
  log::debug!("rpc: get brc20_tick_info: {}", tick);
  if tick.as_bytes().len() != 4 {
    return Json(ApiResponse::api_err(&ApiError::BadRequest(
      ERR_TICK_LENGTH.to_string(),
    )));
  }
  let tick = tick.to_lowercase();

  let tick_info = match index.brc20_get_tick_info(&tick) {
    Ok(tick_info) => tick_info,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };

  log::debug!("rpc: get brc20_tick_info: {:?} {:?}", tick, tick_info);

  if tick_info.is_none() {
    return Json(ApiResponse::api_err(&ApiError::not_found("tick not found")));
  }

  let tick_info = &tick_info.unwrap();

  if tick_info.tick != brc20::Tick::from_str(&tick).unwrap() {
    return Json(ApiResponse::api_err(&ApiError::internal("db: not match")));
  }

  let mut result: TickInfo = tick_info.into();
  let inscription = match index.get_inscription_entry(tick_info.inscription_id) {
    Ok(inscription) => inscription,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };
  if !inscription.is_none() {
    let inscription = inscription.unwrap();
    result.inscription_number = Some(inscription.number.to_string());
  }

  Json(ApiResponse::ok(result))
}

pub(crate) async fn brc20_all_tick_info(
  Extension(index): Extension<Arc<Index>>,
) -> Json<ApiResponse<AllTickInfo>> {
  log::debug!("rpc: get brc20_all_tick_info");
  let all_tick_info = match index.brc20_get_all_tick_info() {
    Ok(all_tick_info) => all_tick_info,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };
  log::debug!("rpc: get brc20_all_tick_info: {:?}", all_tick_info);

  let mut result = AllTickInfo { tokens: vec![] };

  for ref tick in all_tick_info {
    let mut json_tick: TickInfo = tick.into();
    let inscription = match index.get_inscription_entry(tick.inscription_id) {
      Ok(inscription) => inscription,
      Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
    };
    if !inscription.is_none() {
      let inscription = inscription.unwrap();
      json_tick.inscription_number = Some(inscription.number.to_string());
      result.tokens.push(json_tick);
    }
  }

  Json(ApiResponse::ok(result))
}

pub(crate) async fn brc20_balance(
  Extension(index): Extension<Arc<Index>>,
  Path((tick, address)): Path<(String, String)>,
) -> Json<ApiResponse<Balance>> {
  log::debug!("rpc: get brc20_balance: {} {}", tick, address);
  if tick.as_bytes().len() != 4 {
    return Json(ApiResponse::api_err(&ApiError::BadRequest(
      ERR_TICK_LENGTH.to_string(),
    )));
  }
  let tick = tick.to_lowercase();

  let address: bitcoin::Address = match address.parse() {
    Ok(address) => address,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::BadRequest(err.to_string()))),
  };

  let balance = match index.brc20_get_balance_by_address(&tick, &address) {
    Ok(balance) => balance,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };

  if balance.is_none() {
    return Json(ApiResponse::api_err(&ApiError::not_found(
      "balance not found",
    )));
  }

  let balance = balance.unwrap();

  let available_balance = balance.overall_balance - balance.transferable_balance;
  if available_balance > balance.overall_balance {
    return Json(ApiResponse::api_err(&ApiError::internal("balance error")));
  }

  log::debug!("rpc: get brc20_balance: {} {} {:?}", tick, address, balance);

  Json(ApiResponse::ok(Balance {
    tick,
    available_balance: available_balance.to_string(),
    transferable_balance: balance.transferable_balance.to_string(),
    overall_balance: balance.overall_balance.to_string(),
  }))
}

pub(crate) async fn brc20_all_balance(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> Json<ApiResponse<AllBalance>> {
  log::debug!("rpc: get brc20_all_balance: {}", address);
  let address: bitcoin::Address = match address.parse() {
    Ok(address) => address,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::BadRequest(err.to_string()))),
  };

  let all_balance = match index.brc20_get_all_balance_by_address(&address) {
    Ok(balance) => balance,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };

  log::debug!("rpc: get brc20_all_balance: {} {:?}", address, all_balance);

  Json(ApiResponse::ok(AllBalance {
    balance: all_balance
      .iter()
      .map(|(tick, bal)| Balance {
        tick: std::str::from_utf8(tick.as_bytes()).unwrap().to_string(),
        available_balance: (bal.overall_balance - bal.transferable_balance).to_string(),
        transferable_balance: bal.transferable_balance.to_string(),
        overall_balance: bal.overall_balance.to_string(),
      })
      .collect(),
  }))
}

pub(crate) async fn brc20_tx_events(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> Json<ApiResponse<TxEvents>> {
  log::debug!("rpc: get brc20_tx_events: {}", txid);
  let txid = match bitcoin::Txid::from_str(&txid) {
    Ok(txid) => txid,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::BadRequest(err.to_string()))),
  };
  let tx_events = match index.brc20_get_tx_events_by_txid(&txid) {
    Ok(tx_events) => tx_events,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };
  if tx_events.is_none() {
    return Json(ApiResponse::api_err(&ApiError::not_found(
      "tx events not found",
    )));
  }
  let tx_events = tx_events.unwrap();
  log::debug!("rpc: get brc20_tx_events: {} {:?}", txid, tx_events);

  let mut result = TxEvents {
    txid: txid.to_string(),
    events: Vec::with_capacity(tx_events.len()),
  };
  for event in &tx_events {
    let mut json_event: TxEvent = event.into();
    let inscription = match index.get_inscription_entry(event.inscription_id) {
      Ok(inscription) => inscription,
      Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
    };
    if !inscription.is_none() {
      let inscription = inscription.unwrap();
      json_event.set_inscription_number(Some(inscription.number.to_string()));
    }
    result.events.push(json_event);
  }
  Json(ApiResponse::ok(result))
}

pub(crate) async fn brc20_block_events(
  Extension(index): Extension<Arc<Index>>,
  Path(block_hash): Path<String>,
) -> Json<ApiResponse<BlockEvents>> {
  log::debug!("rpc: get brc20_block_events: {}", block_hash);
  let blockhash = match bitcoin::BlockHash::from_str(&block_hash) {
    Ok(blockhash) => blockhash,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::BadRequest(err.to_string()))),
  };

  let block_events = match index.brc20_get_block_events_by_blockhash(blockhash) {
    Ok(block_events) => block_events,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };
  if block_events.is_none() {
    return Json(ApiResponse::api_err(&ApiError::not_found(
      "block not found",
    )));
  }
  let block_events = block_events.unwrap();
  log::debug!(
    "rpc: get brc20_block_events: {} {:?}",
    block_hash,
    block_events
  );

  let mut result = BlockEvents {
    block: Vec::with_capacity(block_events.len()),
  };

  for (txid, events) in block_events {
    let mut tx_events = TxEvents {
      txid: txid.to_string(),
      events: Vec::with_capacity(events.len()),
    };
    for event in &events {
      let mut json_event: TxEvent = event.into();
      let inscription = match index.get_inscription_entry(event.inscription_id) {
        Ok(inscription) => inscription,
        Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
      };
      if !inscription.is_none() {
        let inscription = inscription.unwrap();
        json_event.set_inscription_number(Some(inscription.number.to_string()));
      }
      tx_events.events.push(json_event);
    }
    result.block.push(tx_events);
  }

  Json(ApiResponse::ok(result))
}

pub(crate) async fn brc20_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path((tick, address)): Path<(String, String)>,
) -> Json<ApiResponse<TransferableInscriptions>> {
  log::debug!("rpc: get brc20_transferable: {} {}", tick, address);
  if tick.as_bytes().len() != 4 {
    return Json(ApiResponse::api_err(&ApiError::BadRequest(
      ERR_TICK_LENGTH.to_string(),
    )));
  }
  let tick = tick.to_lowercase();

  let address: bitcoin::Address = match address.parse() {
    Ok(address) => address,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::BadRequest(err.to_string()))),
  };

  let transferable = match index.brc20_get_tick_transferable_by_address(&tick, &address) {
    Ok(balance) => balance,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };
  log::debug!(
    "rpc: get brc20_transferable: {} {} {:?}",
    tick,
    address,
    transferable
  );

  let mut result = TransferableInscriptions {
    inscriptions: Vec::with_capacity(transferable.len()),
  };
  for trans in &transferable {
    let mut ins = TransferableInscription {
      id: trans.inscription_id.to_string(),
      number: None,
      amount: trans.amount.to_string(),
      tick: tick.clone(),
    };
    let inscription = match index.get_inscription_entry(trans.inscription_id) {
      Ok(inscription) => inscription,
      Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
    };
    if !inscription.is_none() {
      let inscription = inscription.unwrap();
      ins.number = Some(inscription.number.to_string());
    }
    result.inscriptions.push(ins);
  }

  Json(ApiResponse::ok(result))
}

pub(crate) async fn brc20_all_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> Json<ApiResponse<TransferableInscriptions>> {
  log::debug!("rpc: get brc20_all_transferable: {}", address);
  let address: bitcoin::Address = match address.parse() {
    Ok(address) => address,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::BadRequest(err.to_string()))),
  };

  let transferable = match index.brc20_get_all_transferable_by_address(&address) {
    Ok(balance) => balance,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };
  log::debug!(
    "rpc: get brc20_all_transferable: {} {:?}",
    address,
    transferable
  );

  let mut result = TransferableInscriptions {
    inscriptions: Vec::with_capacity(transferable.len()),
  };
  for trans in &transferable {
    let mut ins = TransferableInscription {
      id: trans.inscription_id.to_string(),
      number: None,
      amount: trans.amount.to_string(),
      tick: std::str::from_utf8(trans.tick.as_bytes())
        .unwrap()
        .to_string(),
    };
    let inscription = match index.get_inscription_entry(trans.inscription_id) {
      Ok(inscription) => inscription,
      Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
    };
    if !inscription.is_none() {
      let inscription = inscription.unwrap();
      ins.number = Some(inscription.number.to_string());
    }
    result.inscriptions.push(ins);
  }

  Json(ApiResponse::ok(result))
}
