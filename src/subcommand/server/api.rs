use super::error::ApiError;
use super::*;
use axum::Json;

const ERR_TICK_LENGTH: &str = "tick must be 4 bytes length";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickInfo {
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: String,
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
      inscription_number: tick_info.inscription_number.to_string(),
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
          inscription_number: event.inscription_number.to_string(),
          old_satpoint: event.old_satpoint,
          new_satpoint: event.new_satpoint,
          supply: deploy_event.supply.to_string(),
          limit_per_mint: deploy_event.limit_per_mint.to_string(),
          decimal: deploy_event.decimal as u64,
          msg_sender: event.from.to_string(),
          deploy_by: event.to.to_string(),
          valid: true,
          msg: "ok".to_string(),
          event: String::from("deploy"),
        }),
        brc20::BRC20Event::Mint(mint_event) => Self::Mint(MintEvent {
          tick: std::str::from_utf8(mint_event.tick.as_bytes())
            .unwrap()
            .to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: event.inscription_number.to_string(),
          old_satpoint: event.old_satpoint,
          new_satpoint: event.new_satpoint,
          amount: mint_event.amount.to_string(),
          msg_sender: event.from.to_string(),
          to: event.to.to_string(),
          valid: true,
          msg: mint_event.msg.clone().unwrap_or("ok".to_string()),
          event: String::from("mint"),
        }),
        brc20::BRC20Event::TransferPhase1(trans1) => {
          Self::InscribeTransfer(InscribeTransferEvent {
            tick: std::str::from_utf8(trans1.tick.as_bytes())
              .unwrap()
              .to_string(),
            inscription_id: event.inscription_id.to_string(),
            inscription_number: event.inscription_number.to_string(),
            old_satpoint: event.old_satpoint,
            new_satpoint: event.new_satpoint,
            amount: trans1.amount.to_string(),
            msg_sender: event.from.to_string(),
            owner: event.to.to_string(),
            valid: true,
            msg: "ok".to_string(),
            event: String::from("inscribeTransfer"),
          })
        }
        brc20::BRC20Event::TransferPhase2(trans2) => Self::Transfer(TransferEvent {
          tick: std::str::from_utf8(trans2.tick.as_bytes())
            .unwrap()
            .to_string(),
          inscription_id: event.inscription_id.to_string(),
          inscription_number: event.inscription_number.to_string(),
          old_satpoint: event.old_satpoint,
          new_satpoint: event.new_satpoint,
          amount: trans2.amount.to_string(),
          from: event.from.to_string(),
          to: event.to.to_string(),
          valid: true,
          msg: trans2.msg.clone().unwrap_or("ok".to_string()),
          event: String::from("transfer"),
        }),
      },
      Err(err) => Self::Error(ErrorEvent {
        inscription_id: event.inscription_id.to_string(),
        inscription_number: event.inscription_number.to_string(),
        old_satpoint: event.old_satpoint,
        new_satpoint: event.new_satpoint,
        valid: false,
        from: event.from.to_string(),
        to: event.to.to_string(),
        msg: err.to_string(),
        event: match event.op {
          brc20::EventType::Deploy => "deploy",
          brc20::EventType::Mint => "mint",
          brc20::EventType::TransferPhase1 => "inscribeTransfer",
          brc20::EventType::TransferPhase2 => "transfer",
        }
        .to_string(),
      }),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
#[serde(rename_all = "camelCase")]
pub enum TxEvent {
  Deploy(DeployEvent),
  Mint(MintEvent),
  InscribeTransfer(InscribeTransferEvent),
  Transfer(TransferEvent),
  Error(ErrorEvent),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
  pub inscription_id: String,
  pub inscription_number: String,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub from: String,
  pub to: String,
  pub valid: bool,
  pub msg: String,
  pub event: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployEvent {
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: String,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub supply: String,
  pub limit_per_mint: String,
  pub decimal: u64,
  pub msg_sender: String,
  pub deploy_by: String,
  pub valid: bool,
  pub msg: String,
  pub event: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintEvent {
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: String,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub amount: String,
  pub msg_sender: String,
  pub to: String,
  pub valid: bool,
  pub msg: String,
  pub event: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscribeTransferEvent {
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: String,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub amount: String,
  pub msg_sender: String,
  pub owner: String,
  pub valid: bool,
  pub msg: String,
  pub event: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferEvent {
  pub tick: String,
  pub inscription_id: String,
  pub inscription_number: String,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub amount: String,
  pub from: String,
  pub to: String,
  pub valid: bool,
  pub msg: String,
  pub event: String,
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
  pub number: String,
  pub amount: String,
  pub tick: String,
  pub owner: String,
}

impl From<&brc20::TransferableLog> for TransferableInscription {
  fn from(trans: &brc20::TransferableLog) -> Self {
    Self {
      id: trans.inscription_id.to_string(),
      number: trans.inscription_number.to_string(),
      amount: trans.amount.to_string(),
      tick: trans.tick.as_str().to_string(),
      owner: trans.owner.to_string(),
    }
  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeightInfo<T: Serialize> {
  pub ord_height: Option<u64>,
  pub btc_chain_info: Option<T>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HeightInfoQuery {
  btc: Option<bool>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdInscription {
  pub id: String,
  pub number: String,
  pub content_type: Option<String>,
  pub content: Option<String>,
  pub owner: String,
  pub genesis_height: u64,
  pub location: String,
  pub sat: Option<u64>,
}

pub(crate) async fn node_info(
  Extension(index): Extension<Arc<Index>>,
  Query(query): Query<HeightInfoQuery>,
) -> Json<ApiResponse<HeightInfo<bitcoincore_rpc::json::GetBlockchainInfoResult>>> {
  log::debug!("rpc: get node_info");
  let (ord_height, btc_info) = match index.height_btc(query.btc.unwrap_or_default()) {
    Ok(height) => height,
    Err(err) => {
      return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string())));
    }
  };
  let mut height_info = HeightInfo {
    ord_height: None,
    btc_chain_info: btc_info,
  };
  if !ord_height.is_none() {
    height_info.ord_height = Some(ord_height.unwrap().0);
  }
  return Json(ApiResponse::ok(height_info));
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

  Json(ApiResponse::ok(tick_info.into()))
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

  Json(ApiResponse::ok(AllTickInfo {
    tokens: all_tick_info.iter().map(|t| t.into()).collect(),
  }))
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

  Json(ApiResponse::ok(TxEvents {
    txid: txid.to_string(),
    events: tx_events.iter().map(|e| e.into()).collect(),
  }))
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

  Json(ApiResponse::ok(BlockEvents {
    block: block_events
      .iter()
      .map(|(txid, events)| TxEvents {
        txid: txid.to_string(),
        events: events.iter().map(|e| e.into()).collect(),
      })
      .collect(),
  }))
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

  Json(ApiResponse::ok(TransferableInscriptions {
    inscriptions: transferable.iter().map(|trans| trans.into()).collect(),
  }))
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

  Json(ApiResponse::ok(TransferableInscriptions {
    inscriptions: transferable.iter().map(|trans| trans.into()).collect(),
  }))
}

fn ord_get_inscription_by_id(
  index: Arc<Index>,
  id: InscriptionId,
) -> Json<ApiResponse<OrdInscription>> {
  let inscription_data = match index.get_inscription_all_data_by_id(id) {
    Ok(Some(inscription_data)) => inscription_data,
    Ok(None) => {
      return Json(ApiResponse::api_err(&ApiError::not_found(
        "inscription not found",
      )))
    }
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };

  Json(ApiResponse::ok(OrdInscription {
    id: id.to_string(),
    number: inscription_data.entry.number.to_string(),
    content_type: inscription_data
      .inscription
      .content_type()
      .map(|c| String::from(c)),
    content: inscription_data.inscription.body().map(|c| hex::encode(c)),
    owner: brc20::ScriptKey::from_script(
      &inscription_data.tx.output[0].script_pubkey,
      index.get_chain_network(),
    )
    .to_string(),
    genesis_height: inscription_data.entry.height,
    location: inscription_data.sat_point.to_string(),
    sat: inscription_data.entry.sat.map(|s| s.0),
  }))
}

pub(crate) async fn ord_inscription_id(
  Extension(index): Extension<Arc<Index>>,
  Path(id): Path<String>,
) -> Json<ApiResponse<OrdInscription>> {
  log::debug!("rpc: get ord_inscription_id: {}", id);
  let id = match InscriptionId::from_str(&id) {
    Ok(id) => id,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::BadRequest(err.to_string()))),
  };

  return ord_get_inscription_by_id(index, id);
}

pub(crate) async fn ord_inscription_number(
  Extension(index): Extension<Arc<Index>>,
  Path(number): Path<u64>,
) -> Json<ApiResponse<OrdInscription>> {
  log::debug!("rpc: get ord_inscription_number: {}", number);
  let id = match index.get_inscription_id_by_inscription_number(number) {
    Ok(Some(id)) => id,
    Ok(None) => {
      return Json(ApiResponse::api_err(&ApiError::not_found(
        "inscription not found",
      )))
    }
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };

  return ord_get_inscription_by_id(index, id);
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutPointData {
  pub txid: String,
  pub script_pub_key: String,
  pub address: Option<String>,
  pub value: u64,
  pub inscription_digest: Vec<InscriptionDigest>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscriptionDigest {
  pub id: String,
  pub number: u64,
  pub location: String,
}

pub(crate) async fn ord_outpoint(
  Extension(index): Extension<Arc<Index>>,
  Path(outpoint): Path<OutPoint>,
) -> Json<ApiResponse<OutPointData>> {
  log::debug!("rpc: get ord_outpoint: {}", outpoint);

  let inscription_ids = match index.get_inscriptions_on_output(outpoint) {
    Ok(out_point) => out_point,
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };
  if inscription_ids.is_empty() {
    return Json(ApiResponse::api_err(&ApiError::not_found(
      "inscription not found",
    )));
  }

  let tx = match index.get_transaction(outpoint.txid) {
    Ok(Some(tx)) => tx,
    Ok(None) => {
      return Json(ApiResponse::api_err(&ApiError::not_found(
        "transaction not found",
      )))
    }
    Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
  };

  let vout = match tx.output.get(outpoint.vout as usize) {
    Some(vout) => vout,
    None => return Json(ApiResponse::api_err(&ApiError::not_found("vout not found"))),
  };

  let mut inscription_digests = Vec::with_capacity(inscription_ids.len());
  for id in &inscription_ids {
    let ins_data = match index.get_inscription_entry(id.clone()) {
      Ok(Some(ins_data)) => ins_data,
      Ok(None) => {
        return Json(ApiResponse::api_err(&ApiError::not_found(
          "inscription not found",
        )))
      }
      Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
    };
    let satpoint = match index.get_inscription_satpoint_by_id(id.clone()) {
      Ok(Some(satpoint)) => satpoint,
      Ok(None) => {
        return Json(ApiResponse::api_err(&ApiError::not_found(
          "inscription not found",
        )))
      }
      Err(err) => return Json(ApiResponse::api_err(&ApiError::Internal(err.to_string()))),
    };
    inscription_digests.push(InscriptionDigest {
      id: id.to_string(),
      number: ins_data.number,
      location: satpoint.to_string(),
    });
  }

  Json(ApiResponse::ok(OutPointData {
    txid: outpoint.txid.to_string(),
    script_pub_key: vout.script_pubkey.asm(),
    address: match brc20::ScriptKey::from_script(&vout.script_pubkey, index.get_chain_network()) {
      brc20::ScriptKey::Address(address) => Some(address.to_string()),
      _ => None,
    },
    value: vout.value,
    inscription_digest: inscription_digests,
  }))
}
