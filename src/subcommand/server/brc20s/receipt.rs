use {super::*, crate::okx::datastore::brc20s, axum::Json};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
  pub op: OperationType,
  pub inscription_number: Option<i64>,
  pub inscription_id: Option<InscriptionId>,
  pub old_satpoint: Option<SatPoint>,
  pub new_satpoint: Option<SatPoint>,
  pub from: ScriptPubkey,
  pub to: Option<ScriptPubkey>,
  pub valid: bool,
  pub msg: String,
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
        _ => Some(receipt.inscription_id),
      },
      old_satpoint: match receipt.op {
        brc20s::OperationType::PassiveUnStake => None,
        _ => Some(receipt.old_satpoint),
      },
      new_satpoint: match receipt.op {
        brc20s::OperationType::PassiveUnStake => None,
        _ => Some(receipt.new_satpoint),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Event {
  DeployTick(DeployTickEvent),
  DeployPool(DeployPoolEvent),
  Deposit(DepositEvent),
  Withdraw(WithdrawEvent),
  PassiveWithdraw(PassiveWithdrawEvent),
  Mint(MintEvent),
  InscribeTransfer(InscribeTransferEvent),
  Transfer(TransferEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployTickEvent {
  tick: Tick,
  supply: String,
  decimal: u8,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployPoolEvent {
  pid: String,
  stake: Stake,
  earn: Earn,
  pool: String,
  erate: String,
  only: u8,
  dmax: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositEvent {
  pid: String,
  amount: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawEvent {
  pid: String,
  amount: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassiveWithdrawEvent {
  pid: String,
  amount: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintEvent {
  pid: String,
  amount: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscribeTransferEvent {
  tick: Tick,
  amount: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferEvent {
  tick: Tick,
  amount: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  msg: Option<String>,
  from: ScriptPubkey,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxReceipts {
  pub receipts: Vec<Receipt>,
  pub txid: String,
}

// brc20s/tx/:txid/receipts
pub(crate) async fn brc20s_txid_receipts(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<TxReceipts> {
  log::debug!("rpc: get brc20s_txid_receipts: {}", txid);
  let txid = Txid::from_str(&txid).unwrap();

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
  let txid = Txid::from_str(&txid).unwrap();

  let all_receipt = index
    .brc20s_txid_receipts(&txid)?
    .ok_or_api_not_found(BRC20SError::ReceiptsNotFound)?;

  log::debug!("rpc: get brc20s_debug_txid_receipts: {:?}", all_receipt);

  Ok(Json(ApiResponse::ok(all_receipt)))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockReceipts {
  pub block: Vec<TxReceipts>,
}

// brc20s/block/:blockhash/receipts
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
