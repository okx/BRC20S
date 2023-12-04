use std::collections::HashMap;
use super::{brc20_types::*, *};
use crate::okx::{
  datastore::{
    brc20::{redb as brc20_db, Tick},
    ScriptKey,
  },
  protocol::brc20 as brc20_protocol,
};
use axum::Json;
use bitcoin::address::Address;

pub(crate) async fn brc20_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tick): Path<String>,
) -> ApiResult<TickInfo> {
  log::debug!("rpc: get brc20_tick_info: {}", tick);
  let tick =
    Tick::from_str(&tick).map_err(|_| ApiError::bad_request(BRC20Error::IncorrectTickFormat))?;
  let tick_info = &index
    .brc20_get_tick_info(&tick)?
    .ok_or_api_not_found("tick not found")?;

  log::debug!("rpc: get brc20_tick_info: {:?} {:?}", tick, tick_info);

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

  let tick =
    Tick::from_str(&tick).map_err(|_| ApiError::bad_request(BRC20Error::IncorrectTickFormat))?;

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let balance = index
    .brc20_get_balance_by_address(&tick, &address)?
    .ok_or_api_not_found(BRC20Error::BalanceNotFound)?;

  let available_balance = balance.overall_balance - balance.transferable_balance;

  log::debug!("rpc: get brc20_balance: {} {} {:?}", tick, address, balance);

  Ok(Json(ApiResponse::ok(Balance {
    tick: balance.tick.to_string(),
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

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let all_balance = index.brc20_get_all_balance_by_address(&address)?;

  log::debug!("rpc: get brc20_all_balance: {} {:?}", address, all_balance);

  Ok(Json(ApiResponse::ok(AllBalance {
    balance: all_balance
      .iter()
      .map(|bal| Balance {
        tick: bal.tick.to_string(),
        available_balance: (bal.overall_balance - bal.transferable_balance).to_string(),
        transferable_balance: bal.transferable_balance.to_string(),
        overall_balance: bal.overall_balance.to_string(),
      })
      .collect(),
  })))
}

pub(crate) async fn brc20_acc_count(
  Extension(index): Extension<Arc<Index>>,
) -> ApiResult<u64> {
  log::debug!("rpc: get brc20_acc_count");
  let count = index.brc20_get_acc_count()?;
  log::debug!("rpc: get brc20_acc_count: {:?}", count);

  Ok(Json(ApiResponse::ok(count)))
}

pub(crate) async fn brc20_all_acc_balance(
  Extension(index): Extension<Arc<Index>>,
  Query(page): Query<Pagination>,
) -> ApiResult<AllAccBalances> {
  log::debug!("rpc: get brc20_all_acc_balance");
  let all_balances = index.brc20_get_all_acc_balance(page.start.unwrap_or(0), page.limit)?;
  log::debug!("rpc: get brc20_all_acc_balance: {:?}", all_balances);

  let mut balances = HashMap::new();
  for (addr, bals) in &all_balances {
    for bal in bals.iter() {
      balances.entry(addr.to_string()).or_insert(Vec::new()).push(Balance {
        tick: bal.tick.to_string(),
        available_balance: (bal.overall_balance - bal.transferable_balance).to_string(),
        transferable_balance: bal.transferable_balance.to_string(),
        overall_balance: bal.overall_balance.to_string(),
      });
    }
  }

  Ok(Json(ApiResponse::ok(AllAccBalances {
    balances
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
    return Err(ApiError::not_found(BRC20Error::OperationNotFound));
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
    .ok_or_api_not_found(BRC20Error::EventsNotFound)?;

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

pub(crate) async fn brc20_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path((tick, address)): Path<(String, String)>,
) -> ApiResult<TransferableInscriptions> {
  log::debug!("rpc: get brc20_transferable: {} {}", tick, address);
  if tick.as_bytes().len() != 4 {
    return Err(ApiError::bad_request(BRC20Error::IncorrectTickFormat));
  }
  let tick = tick.to_lowercase();

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

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

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

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

fn get_operations_by_txid(index: &Arc<Index>, txid: &bitcoin::Txid) -> Result<TxInscriptionInfo> {
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
    .collect::<Vec<Inscription>>();

  let rtx = index.begin_read()?.0;
  let brc20_store = brc20_db::DataStoreReader::new(&rtx);
  for operation in operations {
    match brc20_protocol::Message::resolve(&brc20_store, &new_inscriptions, &operation)? {
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
