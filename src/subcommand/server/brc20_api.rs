use super::{error::ApiError, types::ScriptPubkey, *};
use crate::okx::datastore::brc20::redb as brc20_db;
use crate::okx::{
  datastore::{
    brc20::{self},
    ScriptKey,
  },
  protocol::brc20 as brc20_protocol,
};
use axum::Json;

pub(super) const ERR_TICK_LENGTH: &str = "tick must be 4 bytes length";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickInfo {
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub supply: String,
  pub limit_per_mint: String,
  pub minted: String,
  pub decimal: u64,
  pub deploy_by: ScriptPubkey,
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
      inscription_number: tick_info.inscription_number,
      supply: tick_info.supply.to_string(),
      limit_per_mint: tick_info.limit_per_mint.to_string(),
      minted: tick_info.minted.to_string(),
      decimal: tick_info.decimal as u64,
      deploy_by: tick_info.deploy_by.clone().into(),
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

impl From<&brc20::Receipt> for TxEvent {
  fn from(event: &brc20::Receipt) -> Self {
    match &event.result {
      Ok(result) => match result {
        brc20::Event::Deploy(deploy_event) => Self::Deploy(DeployEvent {
          tick: std::str::from_utf8(deploy_event.tick.as_bytes())
            .unwrap()
            .to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: event.inscription_number,
          old_satpoint: event.old_satpoint,
          new_satpoint: event.new_satpoint,
          supply: deploy_event.supply.to_string(),
          limit_per_mint: deploy_event.limit_per_mint.to_string(),
          decimal: deploy_event.decimal as u64,
          from: event.from.clone().into(),
          to: event.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
          event: String::from("deploy"),
        }),
        brc20::Event::Mint(mint_event) => Self::Mint(MintEvent {
          tick: std::str::from_utf8(mint_event.tick.as_bytes())
            .unwrap()
            .to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: event.inscription_number,
          old_satpoint: event.old_satpoint,
          new_satpoint: event.new_satpoint,
          amount: mint_event.amount.to_string(),
          from: event.from.clone().into(),
          to: event.to.clone().into(),
          valid: true,
          msg: mint_event.msg.clone().unwrap_or("ok".to_string()),
          event: String::from("mint"),
        }),
        brc20::Event::InscribeTransfer(trans1) => Self::InscribeTransfer(InscribeTransferEvent {
          tick: std::str::from_utf8(trans1.tick.as_bytes())
            .unwrap()
            .to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: event.inscription_number,
          old_satpoint: event.old_satpoint,
          new_satpoint: event.new_satpoint,
          amount: trans1.amount.to_string(),
          from: event.from.clone().into(),
          to: event.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
          event: String::from("inscribeTransfer"),
        }),
        brc20::Event::Transfer(trans2) => Self::Transfer(TransferEvent {
          tick: std::str::from_utf8(trans2.tick.as_bytes())
            .unwrap()
            .to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: event.inscription_number,
          old_satpoint: event.old_satpoint,
          new_satpoint: event.new_satpoint,
          amount: trans2.amount.to_string(),
          from: event.from.clone().into(),
          to: event.to.clone().into(),
          valid: true,
          msg: trans2.msg.clone().unwrap_or("ok".to_string()),
          event: String::from("transfer"),
        }),
      },
      Err(err) => Self::Error(ErrorEvent {
        inscription_id: event.inscription_id.to_string(),
        inscription_number: event.inscription_number,
        old_satpoint: event.old_satpoint,
        new_satpoint: event.new_satpoint,
        valid: false,
        from: event.from.clone().into(),
        to: event.to.clone().into(),
        msg: err.to_string(),
        event: match event.op {
          brc20::OperationType::Deploy => "deploy",
          brc20::OperationType::Mint => "mint",
          brc20::OperationType::InscribeTransfer => "inscribeTransfer",
          brc20::OperationType::Transfer => "transfer",
        }
        .to_string(),
      }),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum TxEvent {
  Deploy(DeployEvent),
  Mint(MintEvent),
  InscribeTransfer(InscribeTransferEvent),
  Transfer(TransferEvent),
  Error(ErrorEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub supply: String,
  pub limit_per_mint: String,
  pub decimal: u64,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub amount: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscribeTransferEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub amount: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferEvent {
  #[serde(rename = "type")]
  pub event: String,
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub amount: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
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
  pub inscription_id: String,
  pub inscription_number: i64,
  pub amount: String,
  pub tick: String,
  pub owner: String,
}

impl From<&brc20::TransferableLog> for TransferableInscription {
  fn from(trans: &brc20::TransferableLog) -> Self {
    Self {
      inscription_id: trans.inscription_id.to_string(),
      inscription_number: trans.inscription_number,
      amount: trans.amount.to_string(),
      tick: trans.tick.as_str().to_string(),
      owner: trans.owner.to_string(),
    }
  }
}

pub(crate) async fn brc20_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tick): Path<String>,
) -> ApiResult<TickInfo> {
  log::debug!("rpc: get brc20_tick_info: {}", tick);
  if tick.as_bytes().len() != 4 {
    return Err(ApiError::bad_request(brc20_api::ERR_TICK_LENGTH));
  }
  let tick = tick.to_lowercase();

  let tick_info = &index
    .brc20_get_tick_info(&tick)?
    .ok_or_api_not_found("tick not found")?;

  log::debug!("rpc: get brc20_tick_info: {:?} {:?}", tick, tick_info);

  if tick_info.tick != brc20::Tick::from_str(&tick).unwrap() {
    return Err(ApiError::internal("db: not match"));
  }

  Ok(Json(ApiResponse::ok(tick_info.into())))
}

pub(crate) async fn brc20_all_tick_info(
  Extension(index): Extension<Arc<Index>>,
) -> ApiResult<AllTickInfo> {
  log::debug!("rpc: get brc20_all_tick_info");
  let all_tick_info = index.brc20_get_all_tick_info()?;
  log::debug!("rpc: get brc20_all_tick_info: {:?}", all_tick_info);

  Ok(Json(ApiResponse::ok(AllTickInfo {
    tokens: all_tick_info.iter().map(|t| t.into()).collect(),
  })))
}

pub(crate) async fn brc20_balance(
  Extension(index): Extension<Arc<Index>>,
  Path((tick, address)): Path<(String, String)>,
) -> ApiResult<Balance> {
  log::debug!("rpc: get brc20_balance: {} {}", tick, address);
  if tick.as_bytes().len() != 4 {
    return Err(ApiError::BadRequest(brc20_api::ERR_TICK_LENGTH.to_string()));
  }
  let tick = tick.to_lowercase();

  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;

  let balance = index
    .brc20_get_balance_by_address(&tick, &address)?
    .ok_or_api_not_found("balance not found")?;

  let available_balance = balance.overall_balance - balance.transferable_balance;
  if available_balance > balance.overall_balance {
    return Err(ApiError::internal("balance error"));
  }

  log::debug!("rpc: get brc20_balance: {} {} {:?}", tick, address, balance);

  Ok(Json(ApiResponse::ok(Balance {
    tick,
    available_balance: available_balance.to_string(),
    transferable_balance: balance.transferable_balance.to_string(),
    overall_balance: balance.overall_balance.to_string(),
  })))
}

pub(crate) async fn brc20_all_balance(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<AllBalance> {
  log::debug!("rpc: get brc20_all_balance: {}", address);

  let address: bitcoin::Address = address
    .parse()
    .map_err(|e: bitcoin::util::address::Error| ApiError::bad_request(e.to_string()))?;

  let all_balance = index.brc20_get_all_balance_by_address(&address)?;

  log::debug!("rpc: get brc20_all_balance: {} {:?}", address, all_balance);

  Ok(Json(ApiResponse::ok(AllBalance {
    balance: all_balance
      .iter()
      .map(|(tick, bal)| Balance {
        tick: std::str::from_utf8(tick.as_bytes()).unwrap().to_string(),
        available_balance: (bal.overall_balance - bal.transferable_balance).to_string(),
        transferable_balance: bal.transferable_balance.to_string(),
        overall_balance: bal.overall_balance.to_string(),
      })
      .collect(),
  })))
}

pub(crate) async fn brc20_tx(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<TxInscriptionInfo> {
  log::debug!("rpc: get brc20_tx: {}", txid);
  let txid = bitcoin::Txid::from_str(&txid).map_err(|e| ApiError::bad_request(e.to_string()))?;

  let tx_info = get_operations_by_txid(&index, &txid)?;

  if tx_info.inscriptions.is_empty() {
    return Err(ApiError::not_found("brc20 operation not found"));
  }

  log::debug!("rpc: get brc20_tx: {} {:?}", txid, tx_info);
  Ok(Json(ApiResponse::ok(tx_info)))
}

pub(crate) async fn brc20_tx_events(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<TxEvents> {
  log::debug!("rpc: get brc20_tx_events: {}", txid);
  let txid = bitcoin::Txid::from_str(&txid).map_err(|e| ApiError::bad_request(e.to_string()))?;
  let tx_events = index
    .brc20_get_tx_events_by_txid(&txid)?
    .ok_or_api_not_found("tx events not found")?;

  log::debug!("rpc: get brc20_tx_events: {} {:?}", txid, tx_events);

  Ok(Json(ApiResponse::ok(TxEvents {
    txid: txid.to_string(),
    events: tx_events.iter().map(|e| e.into()).collect(),
  })))
}

pub(crate) async fn brc20_block_events(
  Extension(index): Extension<Arc<Index>>,
  Path(block_hash): Path<String>,
) -> ApiResult<BlockEvents> {
  log::debug!("rpc: get brc20_block_events: {}", block_hash);

  let blockhash =
    bitcoin::BlockHash::from_str(&block_hash).map_err(|e| ApiError::bad_request(e.to_string()))?;

  let block_events = index
    .brc20_get_block_events_by_blockhash(blockhash)?
    .ok_or_api_not_found("block not found")?;

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

pub(crate) async fn brc20_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path((tick, address)): Path<(String, String)>,
) -> ApiResult<TransferableInscriptions> {
  log::debug!("rpc: get brc20_transferable: {} {}", tick, address);
  if tick.as_bytes().len() != 4 {
    return Err(ApiError::bad_request(brc20_api::ERR_TICK_LENGTH));
  }
  let tick = tick.to_lowercase();

  let address: bitcoin::Address = address
    .parse()
    .map_err(|err: bitcoin::util::address::Error| ApiError::bad_request(err.to_string()))?;

  let transferable = index.brc20_get_tick_transferable_by_address(&tick, &address)?;
  log::debug!(
    "rpc: get brc20_transferable: {} {} {:?}",
    tick,
    address,
    transferable
  );

  Ok(Json(ApiResponse::ok(TransferableInscriptions {
    inscriptions: transferable.iter().map(|trans| trans.into()).collect(),
  })))
}

pub(crate) async fn brc20_all_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<TransferableInscriptions> {
  log::debug!("rpc: get brc20_all_transferable: {}", address);
  let address: bitcoin::Address = address
    .parse()
    .map_err(|err: bitcoin::util::address::Error| ApiError::bad_request(err.to_string()))?;

  let transferable = index.brc20_get_all_transferable_by_address(&address)?;
  log::debug!(
    "rpc: get brc20_all_transferable: {} {:?}",
    address,
    transferable
  );

  Ok(Json(ApiResponse::ok(TransferableInscriptions {
    inscriptions: transferable.iter().map(|trans| trans.into()).collect(),
  })))
}

pub(super) fn get_operations_by_txid(
  index: &Arc<Index>,
  txid: &bitcoin::Txid,
) -> Result<TxInscriptionInfo> {
  let mut brc20_operation_infos = Vec::new();

  let tx_result = index
    .get_transaction_info(txid)?
    .ok_or(anyhow!("can't get transaction info: {txid}"))?;

  // get inscription operations
  let operations = ord_api::get_ord_operations_by_txid(index, txid)?;

  // get new inscriptions
  let new_inscriptions = Inscription::from_transaction(&tx_result.transaction()?)
    .into_iter()
    .map(|i| i.inscription)
    .collect();

  let rtx = index.begin_read()?.0;
  let brc20_store = brc20_db::DataStoreReader::new(&rtx);
  for operation in operations {
    match brc20_protocol::resolve_message(&brc20_store, &new_inscriptions, &operation)? {
      None => continue,
      Some(msg) => brc20_operation_infos.push(InscriptionInfo {
        action: match msg.op {
          brc20_protocol::Operation::Transfer(_) => ActionType::Transfer,
          _ => ActionType::Inscribe,
        },
        inscription_number: index
          .get_inscription_entry(msg.inscription_id)?
          .map(|entry| entry.number),
        inscription_id: msg.inscription_id.to_string(),
        from: index
          .get_outpoint_entry(&msg.old_satpoint.outpoint)?
          .map(|txout| {
            ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()).into()
          })
          .ok_or(anyhow!("outpoint not found {}", msg.old_satpoint.outpoint))?,
        to: match msg.new_satpoint {
          Some(satpoint) => match index.get_outpoint_entry(&satpoint.outpoint) {
            Ok(Some(txout)) => {
              Some(ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()).into())
            }
            Ok(None) => return Err(anyhow!("outpoint not found {}", satpoint.outpoint)),
            Err(e) => return Err(e),
          },
          None => None,
        },
        old_satpoint: msg.old_satpoint.to_string(),
        new_satpoint: msg.new_satpoint.map(|v| v.to_string()),
        operation: Some(RawOperation::Brc20Operation(msg.op.into())),
      }),
    };
  }
  // if the transaction is not confirmed, try to parsing protocol
  Ok(TxInscriptionInfo {
    txid: txid.to_string(),
    blockhash: tx_result.blockhash.map(|v| v.to_string()),
    confirmed: tx_result.blockhash.is_some(),
    inscriptions: brc20_operation_infos,
  })
}
