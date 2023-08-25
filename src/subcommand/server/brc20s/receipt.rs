use {super::*, crate::okx::datastore::brc20s, axum::Json};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::Receipt)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
  /// Operation type.
  #[schema(value_type = brc20s::OperationType)]
  pub op: OperationType,
  /// THe inscription number.
  pub inscription_number: Option<i64>,
  /// The inscription id.
  pub inscription_id: Option<String>,
  /// The inscription satpoint of the transaction input.
  pub old_satpoint: Option<String>,
  /// The inscription satpoint of the transaction output.
  pub new_satpoint: Option<String>,
  /// The message sender which is an address or script pubkey hash.
  pub from: ScriptPubkey,
  /// The message receiver which is an address or script pubkey hash.
  pub to: Option<ScriptPubkey>,
  /// Whether the receipt is valid.
  pub valid: bool,
  /// The message of the receipt.
  pub msg: String,
  /// The events of the receipt.
  ///
  #[schema(value_type = Vec<brc20s::Event>)]
  pub events: Vec<Event>,
}

impl Receipt {
  pub(super) fn from(receipt: &brc20s::Receipt, index: Arc<Index>) -> Result<Self> {
    let mut result = Self {
      op: receipt.op.clone().into(),
      inscription_number: match receipt.op {
        brc20s::OperationType::PassiveUnStake => None,
        _ => Some(receipt.inscription_number),
      },
      inscription_id: match receipt.op {
        brc20s::OperationType::PassiveUnStake => None,
        _ => Some(receipt.inscription_id.to_string()),
      },
      old_satpoint: match receipt.op {
        brc20s::OperationType::PassiveUnStake => None,
        _ => Some(receipt.old_satpoint.to_string()),
      },
      new_satpoint: match receipt.op {
        brc20s::OperationType::PassiveUnStake => None,
        _ => Some(receipt.new_satpoint.to_string()),
      },
      from: receipt.from.clone().into(),
      to: match receipt.op {
        brc20s::OperationType::PassiveUnStake => None,
        _ => Some(receipt.clone().to.into()),
      },
      valid: receipt.result.is_ok(),
      msg: match &receipt.result {
        Ok(_) => "ok".to_string(),
        Err(e) => e.to_string(),
      },
      events: vec![],
    };

    if let Ok(events) = receipt.result.clone() {
      let mut receipt_events = Vec::new();
      for event in events.into_iter() {
        receipt_events.push(match event {
          brc20s::Event::DeployTick(deploy_tick) => {
            Event::DeployTick(DeployTickEvent::new(deploy_tick, receipt.to.clone().into()))
          }
          brc20s::Event::DeployPool(deploy_pool) => Event::DeployPool(DeployPoolEvent::new(
            deploy_pool,
            receipt.to.clone().into(),
            index.clone(),
          )?),
          brc20s::Event::Deposit(deposit) => {
            Event::Deposit(DepositEvent::new(deposit, receipt.to.clone().into()))
          }
          brc20s::Event::Withdraw(withdraw) => {
            Event::Withdraw(WithdrawEvent::new(withdraw, receipt.to.clone().into()))
          }
          brc20s::Event::PassiveWithdraw(passive_withdraw) => Event::PassiveWithdraw(
            PassiveWithdrawEvent::new(passive_withdraw, receipt.from.clone().into()),
          ),
          brc20s::Event::Mint(mint) => Event::Mint(MintEvent::new(mint, receipt.to.clone().into())),
          brc20s::Event::InscribeTransfer(inscribe_transfer) => {
            Event::InscribeTransfer(InscribeTransferEvent::new(
              inscribe_transfer,
              receipt.to.clone().into(),
              index.clone(),
            )?)
          }
          brc20s::Event::Transfer(transfer) => Event::Transfer(TransferEvent::new(
            transfer,
            receipt.from.clone().into(),
            receipt.to.clone().into(),
            index.clone(),
          )?),
        });
      }
      result.events = receipt_events;
    }
    Ok(result)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::OperationType)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
  Deploy,
  Mint,
  Deposit,
  Withdraw,
  PassiveWithdraw,
  InscribeTransfer,
  Transfer,
}
impl From<brc20s::OperationType> for OperationType {
  fn from(op_type: brc20s::OperationType) -> Self {
    match op_type {
      brc20s::OperationType::Deploy => Self::Deploy,
      brc20s::OperationType::Mint => Self::Mint,
      brc20s::OperationType::Stake => Self::Deposit,
      brc20s::OperationType::UnStake => Self::Withdraw,
      brc20s::OperationType::PassiveUnStake => Self::PassiveWithdraw,
      brc20s::OperationType::InscribeTransfer => Self::InscribeTransfer,
      brc20s::OperationType::Transfer => Self::Transfer,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::Event)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Event {
  /// The deployed tick event.
  #[schema(value_type = brc20s::DeployTickEvent)]
  DeployTick(DeployTickEvent),
  /// The deployed pool event.
  #[schema(value_type = brc20s::DeployPoolEvent)]
  DeployPool(DeployPoolEvent),
  /// The deposit event.
  #[schema(value_type = brc20s::DepositEvent)]
  Deposit(DepositEvent),
  /// The withdraw event.
  #[schema(value_type = brc20s::WithdrawEvent)]
  Withdraw(WithdrawEvent),
  /// The passive withdraw event.
  #[schema(value_type = brc20s::PassiveWithdrawEvent)]
  PassiveWithdraw(PassiveWithdrawEvent),
  /// The mint event.
  #[schema(value_type = brc20s::MintEvent)]
  Mint(MintEvent),
  /// The pretransfer event.
  #[schema(value_type = brc20s::InscribeTransferEvent)]
  InscribeTransfer(InscribeTransferEvent),
  /// The transfer event.
  #[schema(value_type = brc20s::TransferEvent)]
  Transfer(TransferEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::DeployTickEvent)]
#[serde(rename_all = "camelCase")]
pub struct DeployTickEvent {
  /// The ticker info.
  #[schema(value_type = brc20s::Tick)]
  tick: Tick,
  /// The total supply of the ticker.
  #[schema(format = "uint64")]
  supply: String,
  /// The decimal of the ticker.
  decimal: u8,
  /// The deployer of the ticker deployed.
  deployer: ScriptPubkey,
}

impl DeployTickEvent {
  pub(super) fn new(event: brc20s::DeployTickEvent, deployer: ScriptPubkey) -> Self {
    Self {
      tick: Tick {
        id: event.tick_id.hex(),
        name: event.name.as_str().to_string(),
      },
      supply: event.supply.to_string(),
      decimal: event.decimal,
      deployer,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::DeployPoolEvent)]
#[serde(rename_all = "camelCase")]
pub struct DeployPoolEvent {
  /// The pool id.
  pid: String,
  /// The pledge ticker info.
  #[schema(value_type = brc20s::Stake)]
  stake: Stake,
  /// The earn ticker info.
  #[schema(value_type = brc20s::Earn)]
  earn: Earn,
  /// Pool type. Such as "pool", "fixed".
  pool: String,
  /// Mining rate.
  erate: String,
  /// Whether the pool is exclusive.
  only: u8,
  /// The max amount of the pool.
  #[schema(format = "uint64")]
  dmax: String,
  /// The deployer of the pool deployed.
  deployer: ScriptPubkey,
}

impl DeployPoolEvent {
  pub(super) fn new(
    event: brc20s::DeployPoolEvent,
    deployer: ScriptPubkey,
    index: Arc<Index>,
  ) -> Result<Self> {
    let tick_id = brc20s::TickId::from(event.pid.clone());
    let tick_info = index
      .brc20s_tick_info(&tick_id)?
      .ok_or(anyhow!("tick not found, pid: {}", event.pid.as_str()))?;

    Ok(Self {
      pid: event.pid.as_str().to_string(),
      stake: Stake {
        type_field: event.stake.to_type(),
        tick: event.stake.to_string(),
      },
      earn: Earn {
        id: tick_info.tick_id.hex(),
        name: tick_info.name.as_str().to_string(),
      },
      pool: event.ptype.to_string(),
      erate: event.erate.to_string(),
      only: event.only.into(),
      dmax: event.dmax.to_string(),
      deployer,
    })
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::DepositEvent)]
#[serde(rename_all = "camelCase")]
pub struct DepositEvent {
  /// The pool id.
  pid: String,
  /// The amount of the deposit.
  #[schema(format = "uint64")]
  amount: String,
  /// The owner of the deposit.
  owner: ScriptPubkey,
}

impl DepositEvent {
  pub(super) fn new(event: brc20s::DepositEvent, owner: ScriptPubkey) -> Self {
    Self {
      pid: event.pid.as_str().to_string(),
      amount: event.amt.to_string(),
      owner,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::WithdrawEvent)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawEvent {
  /// The pool id.
  pid: String,
  /// The amount of the withdraw.
  #[schema(format = "uint64")]
  amount: String,
  /// The owner of the withdraw.
  owner: ScriptPubkey,
}

impl WithdrawEvent {
  pub(super) fn new(event: brc20s::WithdrawEvent, owner: ScriptPubkey) -> Self {
    Self {
      pid: event.pid.as_str().to_string(),
      amount: event.amt.to_string(),
      owner,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::PassiveWithdrawEvent)]
#[serde(rename_all = "camelCase")]
pub struct PassiveWithdrawEvent {
  /// The pool id.
  pid: String,
  /// The amount of the passive withdraw.
  #[schema(format = "uint64")]
  amount: String,
  /// The owner of the passive withdraw.
  owner: ScriptPubkey,
}

impl PassiveWithdrawEvent {
  pub(super) fn new(event: brc20s::PassiveWithdrawEvent, owner: ScriptPubkey) -> Self {
    Self {
      pid: event.pid.as_str().to_string(),
      amount: event.amt.to_string(),
      owner,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::MintEvent)]
#[serde(rename_all = "camelCase")]
pub struct MintEvent {
  /// The pool id.
  pid: String,
  /// The amount of the mint.
  #[schema(format = "uint64")]
  amount: String,
  /// The owner of the mint.
  owner: ScriptPubkey,
}

impl MintEvent {
  pub(super) fn new(event: brc20s::MintEvent, owner: ScriptPubkey) -> Self {
    Self {
      pid: event.pid.as_str().to_string(),
      amount: event.amt.to_string(),
      owner,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::InscribeTransferEvent)]
#[serde(rename_all = "camelCase")]
pub struct InscribeTransferEvent {
  /// The pool id.
  #[schema(value_type = brc20s::Tick)]
  tick: Tick,
  /// The amount of the transfer.
  amount: String,
  /// The owner of the transfer.
  owner: ScriptPubkey,
}

impl InscribeTransferEvent {
  pub(super) fn new(
    event: brc20s::InscribeTransferEvent,
    owner: ScriptPubkey,
    index: Arc<Index>,
  ) -> Result<Self> {
    let tick_info = index
      .brc20s_tick_info(&event.tick_id)?
      .ok_or(anyhow!("tick not found, tid: {}", event.tick_id.hex()))?;

    Ok(Self {
      tick: Tick {
        id: event.tick_id.hex(),
        name: tick_info.name.as_str().to_string(),
      },
      amount: event.amt.to_string(),
      owner,
    })
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::TransferEvent)]
#[serde(rename_all = "camelCase")]
pub struct TransferEvent {
  /// The pool id.
  #[schema(value_type = brc20s::Tick)]
  tick: Tick,
  /// The amount of the transfer.
  #[schema(format = "uint64")]
  amount: String,
  /// The message of the transfer.
  msg: Option<String>,
  /// The message sender which is an address or script pubkey hash.
  from: ScriptPubkey,
  /// The message receiver which is an address or script pubkey hash.
  to: ScriptPubkey,
}

impl TransferEvent {
  pub(super) fn new(
    event: brc20s::TransferEvent,
    from: ScriptPubkey,
    to: ScriptPubkey,
    index: Arc<Index>,
  ) -> Result<Self> {
    let tick_info = index
      .brc20s_tick_info(&event.tick_id)?
      .ok_or(anyhow!("tick not found, tid: {}", event.tick_id.hex()))?;
    Ok(Self {
      tick: Tick {
        id: event.tick_id.hex(),
        name: tick_info.name.as_str().to_string(),
      },
      amount: event.amt.to_string(),
      msg: event.msg,
      from,
      to,
    })
  }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::TxReceipts)]
#[serde(rename_all = "camelCase")]
pub struct TxReceipts {
  #[schema(value_type = Vec<brc20s::Receipt>)]
  pub receipts: Vec<Receipt>,
  pub txid: String,
}

// brc20s/tx/:txid/receipts
/// Get the transaction receipts by txid.
///
/// Get all receipts of the transaction.
#[utoipa::path(
  get,
  path = "/api/v1/brc20s/tx/{txid}/receipts",
  params(
      ("txid" = String, Path, description = "transaction ID")
),
  responses(
    (status = 200, description = "Obtain transaction receipts by txid", body = BRC20STxReceipts),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
pub(crate) async fn brc20s_txid_receipts(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<TxReceipts> {
  log::debug!("rpc: get brc20s_txid_receipts: {}", txid);
  let txid = Txid::from_str(&txid).map_err(ApiError::bad_request)?;

  let all_receipt = index
    .brc20s_txid_receipts(&txid)?
    .ok_or_api_not_found(BRC20SError::ReceiptsNotFound)?;

  log::debug!("rpc: get brc20s_txid_receipts: {:?}", all_receipt);

  let mut receipts = Vec::new();
  for receipt in all_receipt.iter() {
    match Receipt::from(receipt, index.clone()) {
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
) -> ApiResult<Vec<brc20s::Receipt>> {
  log::debug!("rpc: get brc20s_debug_txid_receipts: {}", txid);
  let txid = Txid::from_str(&txid).map_err(ApiError::bad_request)?;

  let all_receipt = index
    .brc20s_txid_receipts(&txid)?
    .ok_or_api_not_found(BRC20SError::ReceiptsNotFound)?;

  log::debug!("rpc: get brc20s_debug_txid_receipts: {:?}", all_receipt);

  Ok(Json(ApiResponse::ok(all_receipt)))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::BlockReceipts)]
#[serde(rename_all = "camelCase")]
pub struct BlockReceipts {
  #[schema(value_type = Vec<brc20s::TxReceipts>)]
  pub block: Vec<TxReceipts>,
}

// brc20s/block/:blockhash/receipts
/// Get the block receipts by blockhash.
///
/// Get all receipts of the block.
#[utoipa::path(
  get,
  path = "/api/v1/brc20s/block/{blockhash}/receipts",
  params(
      ("blockhash" = String, Path, description = "block hash")
),
  responses(
    (status = 200, description = "Obtain block receipts by block hash", body = BRC20SBlockReceipts),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
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
      match Receipt::from(receipt, index.clone()) {
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
