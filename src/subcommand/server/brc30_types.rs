use super::{types::ScriptPubkey, *};
use crate::okx::datastore::brc30;
use std::{convert::From, vec};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BRC30TickInfo {
  pub tick: Tick,
  pub inscription_id: String,
  pub inscription_number: u64,
  pub minted: String,
  pub supply: String,
  pub decimal: u64,
  pub deployer: ScriptPubkey,
  pub txid: String,
  pub deploy_height: u64,
  pub deploy_blocktime: u64,
}

impl BRC30TickInfo {
  pub fn set_inscription_number(&mut self, inscription_number: u64) {
    self.inscription_number = inscription_number;
  }

  pub fn set_deploy_blocktime(&mut self, deploy_blocktime: u64) {
    self.deploy_blocktime = deploy_blocktime;
  }
}

impl From<&brc30::TickInfo> for BRC30TickInfo {
  fn from(tick_info: &brc30::TickInfo) -> Self {
    let tick = Tick {
      id: tick_info.tick_id.to_lowercase().hex(),
      name: tick_info.name.as_str().to_string(),
    };

    Self {
      tick,
      inscription_id: tick_info.inscription_id.to_string(),
      inscription_number: 0,
      minted: tick_info.circulation.to_string(),
      supply: tick_info.supply.to_string(),
      decimal: tick_info.decimal as u64,
      deployer: tick_info.deployer.clone().into(),
      txid: tick_info.inscription_id.txid.to_string(),
      deploy_height: tick_info.deploy_block,
      deploy_blocktime: 0,
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tick {
  pub id: String,
  pub name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllBRC30TickInfo {
  pub tokens: Vec<BRC30TickInfo>,
  pub total: usize,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BRC30Pool {
  pub pid: String,
  pub stake: Stake,
  pub earn: Earn,
  pub pool: String,
  pub erate: String,
  pub staked: String,
  pub minted: String,
  pub dmax: String,
  pub only: u8,
  pub acc_reward_per_share: String,
  pub latest_update_block: u64,
  pub inscription_id: String,
  pub inscription_number: u64,
  pub deployer: ScriptPubkey,
  pub deploy_height: u64,
  pub deploy_blocktime: u64,
  pub txid: String,
}

impl BRC30Pool {
  pub fn set_earn(&mut self, earn_id: String, earn_name: String) {
    self.earn.id = earn_id;
    self.earn.name = earn_name;
  }

  pub fn set_inscription_num(&mut self, inscription_number: u64) {
    self.inscription_number = inscription_number
  }

  pub fn set_deploy(&mut self, deployer: ScriptPubkey, deploy_height: u64, deploy_blocktime: u64) {
    self.deployer = deployer;
    self.deploy_height = deploy_height;
    self.deploy_blocktime = deploy_blocktime;
  }
}

impl From<&brc30::PoolInfo> for BRC30Pool {
  fn from(pool_info: &brc30::PoolInfo) -> Self {
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
      deploy_height: 0,
      deploy_blocktime: 0,
      txid: pool_info.inscription_id.txid.to_string(),
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stake {
  #[serde(rename = "type")]
  pub type_field: String,
  pub tick: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Earn {
  pub id: String,
  pub name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllBRC30PoolInfo {
  pub tokens: Vec<BRC30Pool>,
  pub total: usize,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
  pub pid: String,
  pub staked: String,
  pub minted: String,
  pub pending_reward: String,
  pub reward_debt: String,
  pub latest_update_block: u64,
}

impl From<&brc30::UserInfo> for UserInfo {
  fn from(user_info: &brc30::UserInfo) -> Self {
    Self {
      pid: user_info.pid.as_str().to_string(),
      staked: user_info.staked.to_string(),
      minted: user_info.minted.to_string(),
      pending_reward: user_info.pending_reward.to_string(),
      reward_debt: user_info.reward_debt.to_string(),
      latest_update_block: user_info.latest_updated_block,
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BRC30Balance {
  pub tick: Tick,
  pub transferable: String,
  pub overall: String,
}

impl BRC30Balance {
  pub fn set_tick_name(&mut self, name: String) {
    self.tick.name = name;
  }
}

impl From<&brc30::Balance> for BRC30Balance {
  fn from(balance: &brc30::Balance) -> Self {
    let tick = Tick {
      id: balance.tick_id.to_lowercase().hex(),
      name: "".to_string(),
    };

    Self {
      tick,
      transferable: balance.transferable_balance.to_string(),
      overall: balance.overall_balance.to_string(),
    }
  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllBRC30Balance {
  pub balance: Vec<BRC30Balance>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transferable {
  pub inscriptions: Vec<BRC30Inscription>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BRC30Inscription {
  pub tick: Tick,
  pub inscription_id: String,
  pub inscription_number: u64,
  pub amount: String,
  pub owner: String,
}

impl BRC30Inscription {
  pub fn set_tick_name(&mut self, name: String) {
    self.tick.name = name;
  }

  pub fn set_inscription_number(&mut self, inscription_number: u64) {
    self.inscription_number = inscription_number;
  }
}

impl From<&brc30::TransferableAsset> for BRC30Inscription {
  fn from(asset: &brc30::TransferableAsset) -> Self {
    let tick = Tick {
      id: asset.tick_id.to_lowercase().hex(),
      name: "".to_string(),
    };

    Self {
      tick,
      inscription_id: asset.inscription_id.to_string(),
      inscription_number: 0,
      amount: asset.amount.to_string(),
      owner: asset.owner.to_string(),
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxReceipts {
  pub receipts: Vec<BRC30Receipt>,
  pub txid: String,
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
impl From<brc30::BRC30OperationType> for OperationType {
  fn from(op_type: brc30::BRC30OperationType) -> Self {
    match op_type {
      brc30::BRC30OperationType::Deploy => Self::Deploy,
      brc30::BRC30OperationType::Mint => Self::Mint,
      brc30::BRC30OperationType::Stake => Self::Deposit,
      brc30::BRC30OperationType::UnStake => Self::Withdraw,
      brc30::BRC30OperationType::PassiveUnStake => Self::PassiveWithdraw,
      brc30::BRC30OperationType::InscribeTransfer => Self::InscribeTransfer,
      brc30::BRC30OperationType::Transfer => Self::Transfer,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BRC30Receipt {
  pub op: OperationType,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscription_number: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscription_id: Option<InscriptionId>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub old_satpoint: Option<SatPoint>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub new_satpoint: Option<SatPoint>,
  pub from: ScriptPubkey,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub to: Option<ScriptPubkey>,
  pub valid: bool,
  pub msg: String,
  pub events: Vec<BRC30Event>,
}

impl BRC30Receipt {
  pub(super) fn from(receipt: &brc30::BRC30Receipt, index: Arc<Index>) -> Result<Self> {
    let mut result = Self {
      op: receipt.op.clone().into(),
      inscription_number: match receipt.op {
        brc30::BRC30OperationType::PassiveUnStake => None,
        _ => Some(receipt.inscription_number),
      },
      inscription_id: match receipt.op {
        brc30::BRC30OperationType::PassiveUnStake => None,
        _ => Some(receipt.inscription_id),
      },
      old_satpoint: match receipt.op {
        brc30::BRC30OperationType::PassiveUnStake => None,
        _ => Some(receipt.old_satpoint),
      },
      new_satpoint: match receipt.op {
        brc30::BRC30OperationType::PassiveUnStake => None,
        _ => Some(receipt.new_satpoint),
      },
      from: receipt.from.clone().into(),
      to: match receipt.op {
        brc30::BRC30OperationType::PassiveUnStake => None,
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
          brc30::BRC30Event::DeployTick(deploy_tick) => {
            BRC30Event::DeployTick(DeployTickEvent::new(deploy_tick, receipt.to.clone().into()))
          }
          brc30::BRC30Event::DeployPool(deploy_pool) => BRC30Event::DeployPool(
            DeployPoolEvent::new(deploy_pool, receipt.to.clone().into(), index.clone())?,
          ),
          brc30::BRC30Event::Deposit(deposit) => {
            BRC30Event::Deposit(DepositEvent::new(deposit, receipt.to.clone().into()))
          }
          brc30::BRC30Event::Withdraw(withdraw) => {
            BRC30Event::Withdraw(WithdrawEvent::new(withdraw, receipt.to.clone().into()))
          }
          brc30::BRC30Event::PassiveWithdraw(passive_withdraw) => BRC30Event::PassiveWithdraw(
            PassiveWithdrawEvent::new(passive_withdraw, receipt.from.clone().into()),
          ),
          brc30::BRC30Event::Mint(mint) => {
            BRC30Event::Mint(MintEvent::new(mint, receipt.to.clone().into()))
          }
          brc30::BRC30Event::InscribeTransfer(inscribe_transfer) => {
            BRC30Event::InscribeTransfer(InscribeTransferEvent::new(
              inscribe_transfer,
              receipt.to.clone().into(),
              index.clone(),
            )?)
          }
          brc30::BRC30Event::Transfer(transfer) => BRC30Event::Transfer(TransferEvent::new(
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
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum BRC30Event {
  DeployTick(DeployTickEvent),
  DeployPool(DeployPoolEvent),
  Deposit(DepositEvent),
  Withdraw(WithdrawEvent),
  PassiveWithdraw(PassiveWithdrawEvent),
  Mint(MintEvent),
  InscribeTransfer(InscribeTransferEvent),
  Transfer(TransferEvent),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployTickEvent {
  tick: Tick,
  supply: String,
  decimal: u8,
  deployer: ScriptPubkey,
}

impl DeployTickEvent {
  pub(super) fn new(event: brc30::DeployTickEvent, deployer: ScriptPubkey) -> Self {
    Self {
      tick: Tick {
        id: event.tick_id.to_lowercase().hex(),
        name: event.name.as_str().to_string(),
      },
      supply: event.supply.to_string(),
      decimal: event.decimal,
      deployer,
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    event: brc30::DeployPoolEvent,
    deployer: ScriptPubkey,
    index: Arc<Index>,
  ) -> Result<Self> {
    let parts: Vec<&str> = event.pid.as_str().split("#").collect();
    let tick_info = index
      .brc30_tick_info(&parts[0].to_string())?
      .ok_or(anyhow!("tick not found, pid: {}", event.pid.as_str()))?;

    Ok(Self {
      pid: event.pid.as_str().to_string(),
      stake: Stake {
        type_field: event.stake.to_type(),
        tick: event.stake.to_string(),
      },
      earn: Earn {
        id: tick_info.tick_id.hex().to_string(),
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositEvent {
  pid: String,
  amount: String,
  owner: ScriptPubkey,
}

impl DepositEvent {
  pub(super) fn new(event: brc30::DepositEvent, owner: ScriptPubkey) -> Self {
    Self {
      pid: event.pid.as_str().to_string(),
      amount: event.amt.to_string(),
      owner,
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawEvent {
  pid: String,
  amount: String,
  owner: ScriptPubkey,
}

impl WithdrawEvent {
  pub(super) fn new(event: brc30::WithdrawEvent, owner: ScriptPubkey) -> Self {
    Self {
      pid: event.pid.as_str().to_string(),
      amount: event.amt.to_string(),
      owner,
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassiveWithdrawEvent {
  pid: String,
  amount: String,
  owner: ScriptPubkey,
}

impl PassiveWithdrawEvent {
  pub(super) fn new(event: brc30::PassiveWithdrawEvent, owner: ScriptPubkey) -> Self {
    Self {
      pid: event.pid.as_str().to_string(),
      amount: event.amt.to_string(),
      owner,
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintEvent {
  pid: String,
  amount: String,
  owner: ScriptPubkey,
}

impl MintEvent {
  pub(super) fn new(event: brc30::MintEvent, owner: ScriptPubkey) -> Self {
    Self {
      pid: event.pid.as_str().to_string(),
      amount: event.amt.to_string(),
      owner,
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscribeTransferEvent {
  tick: Tick,
  amount: String,
  owner: ScriptPubkey,
}

impl InscribeTransferEvent {
  pub(super) fn new(
    event: brc30::InscribeTransferEvent,
    owner: ScriptPubkey,
    index: Arc<Index>,
  ) -> Result<Self> {
    let tick_info = index
      .brc30_tick_info(&event.tick_id.hex())?
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    event: brc30::TransferEvent,
    from: ScriptPubkey,
    to: ScriptPubkey,
    index: Arc<Index>,
  ) -> Result<Self> {
    let tick_info = index
      .brc30_tick_info(&event.tick_id.hex())?
      .ok_or(anyhow!("tick not found, tid: {}", event.tick_id.hex()))?;
    Ok(Self {
      tick: Tick {
        id: event.tick_id.hex(),
        name: tick_info.name.as_str().to_string(),
      },
      amount: event.amt.to_string(),
      msg: event.msg.clone(),
      from,
      to,
    })
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BRC30BlockReceipts {
  pub block: Vec<TxReceipts>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserReward {
  #[serde(rename = "pending_reward")]
  pub pending_reward: String,
  #[serde(rename = "block_num")]
  pub block_num: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakedInfo {
  #[serde(rename = "type")]
  pub type_field: String,
  pub tick: String,
  #[serde(rename = "max_share")]
  pub max_share: String,
  #[serde(rename = "total_only")]
  pub total_only: String,
  #[serde(rename = "staked_pids")]
  pub staked_pids: Vec<StakedPid>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakedPid {
  pub pid: String,
  pub only: bool,
  pub stake: String,
}

impl From<&brc30::StakeInfo> for StakedInfo {
  fn from(stake: &brc30::StakeInfo) -> Self {
    Self {
      type_field: "BRC20".to_string(),
      tick: "".to_string(),
      max_share: stake.max_share.to_string(),
      total_only: stake.total_only.to_string(),
      staked_pids: stake
        .pool_stakes
        .iter()
        .rev()
        .map(|(a, b, c)| StakedPid {
          pid: a.as_str().to_string(),
          only: *b,
          stake: c.to_string(),
        })
        .collect(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{okx::datastore::ScriptKey, txid, InscriptionId, SatPoint};
  use bitcoin::{Address, Network};
  use std::str::FromStr;

  #[test]
  fn serialize_brc30_receipts() {
    let receipt = BRC30Receipt {
      inscription_id: Some(InscriptionId {
        txid: txid(1),
        index: 0xFFFFFFFF,
      }),
      inscription_number: Some(10),
      op: brc30::BRC30OperationType::Deploy.into(),
      old_satpoint: Some(
        SatPoint::from_str(
          "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
        )
        .unwrap(),
      ),
      new_satpoint: Some(
        SatPoint::from_str(
          "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
        )
        .unwrap(),
      ),
      from: ScriptKey::from_script(
        &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .script_pubkey(),
        Network::Bitcoin,
      )
      .into(),
      to: Some(
        ScriptKey::from_script(
          &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
            .unwrap()
            .script_pubkey(),
          Network::Bitcoin,
        )
        .into(),
      ),
      valid: true,
      msg: "ok".to_string(),
      events: vec![
        BRC30Event::DeployTick(DeployTickEvent {
          tick: Tick {
            id: "aabbccddee".to_string(),
            name: "abcdef".to_string(),
          },
          supply: "1000000".to_string(),
          decimal: 18,
          deployer: ScriptKey::from_script(
            &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
              .unwrap()
              .script_pubkey(),
            Network::Bitcoin,
          )
          .into(),
        }),
        BRC30Event::DeployPool(DeployPoolEvent {
          pid: "aabbccddee#1f".to_string(),
          stake: Stake {
            type_field: brc30::PledgedTick::BRC30Tick(
              brc30::TickId::from_str("aabbccddee").unwrap(),
            )
            .to_type(),
            tick: "aabbccddee".to_string(),
          },
          earn: Earn {
            id: "aabbccddee".to_string(),
            name: "abcdef".to_string(),
          },
          pool: "pool".to_string(),
          erate: "1000000".to_string(),
          only: 0,
          dmax: "10000".to_string(),
          deployer: ScriptKey::from_script(
            &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
              .unwrap()
              .script_pubkey(),
            Network::Bitcoin,
          )
          .into(),
        }),
      ],
    };
    pretty_assert_eq!(
      serde_json::to_string_pretty(&receipt).unwrap(),
      r#"{
  "op": "deploy",
  "inscriptionNumber": 10,
  "inscriptionId": "1111111111111111111111111111111111111111111111111111111111111111i4294967295",
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "valid": true,
  "msg": "ok",
  "events": [
    {
      "type": "deployTick",
      "tick": {
        "id": "aabbccddee",
        "name": "abcdef"
      },
      "supply": "1000000",
      "decimal": 18,
      "deployer": {
        "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
      }
    },
    {
      "type": "deployPool",
      "pid": "aabbccddee#1f",
      "stake": {
        "type": "BRC20-S",
        "tick": "aabbccddee"
      },
      "earn": {
        "id": "aabbccddee",
        "name": "abcdef"
      },
      "pool": "pool",
      "erate": "1000000",
      "only": 0,
      "dmax": "10000",
      "deployer": {
        "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
      }
    }
  ]
}"#
    )
  }
}
