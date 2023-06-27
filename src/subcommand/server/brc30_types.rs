use super::*;
use crate::okx::datastore::brc30;
use crate::okx::datastore::brc30::BRC30Event;
use std::convert::From;

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
  pub only: u64,
  pub acc_per_share: String,
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
      acc_per_share: pool_info.acc_reward_per_share.to_string(),
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
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
  pub pid: String,
  pub staked: String,
  pub reward: String,
  pub reward_debt: String,
  pub latest_update_block: u64,
}

impl From<&brc30::UserInfo> for UserInfo {
  fn from(user_info: &brc30::UserInfo) -> Self {
    Self {
      pid: user_info.pid.as_str().to_string(),
      staked: user_info.staked.to_string(),
      reward: user_info.reward.to_string(),
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
  pub inscriptions: Vec<Brc30Inscription>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brc30Inscription {
  pub tick: Tick,
  pub inscription_id: String,
  pub inscription_number: u64,
  pub amount: String,
  pub owner: String,
}

impl Brc30Inscription {
  pub fn set_tick_name(&mut self, name: String) {
    self.tick.name = name;
  }

  pub fn set_inscription_number(&mut self, inscription_number: u64) {
    self.inscription_number = inscription_number;
  }
}

impl From<&brc30::TransferableAsset> for Brc30Inscription {
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
pub struct Events {
  pub events: Vec<Brc30Event>,
  pub txid: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum Brc30Event {
  DeployTick(DeployTickEvent),
  DeployPool(DeployPoolEvent),
  Deposit(DepositEvent),
  Withdraw(WithdrawEvent),
  PassiveWithdraw(PassiveWithdrawEvent),
  Mint(Brc30MintEvent),
  InscribeTransfer(Brc30InscribeTransferEvent),
  Transfer(Brc30TransferEvent),
  Error(Brc30ErrorEvent),
}

impl Brc30Event {
  pub fn set_only(&mut self, only: i64) {
    match self {
      Self::DeployPool(d) => {
        d.only = only;
      }
      _ => {}
    }
  }

  pub fn set_earn(&mut self, id: String, name: String) {
    match self {
      Self::DeployPool(d) => {
        d.earn.id = id;
        d.earn.name = name;
      }
      _ => {}
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployTickEvent {
  #[serde(rename = "type")]
  pub type_field: String,
  pub tick: Tick,
  pub supply: String,
  pub decimal: u8,
  pub inscription_number: i64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployPoolEvent {
  #[serde(rename = "type")]
  pub type_field: String,
  pub pid: String,
  pub stake: Stake,
  pub earn: Earn,
  pub pool: String,
  pub erate: String,
  pub only: i64,
  pub dmax: String,
  pub inscription_number: i64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositEvent {
  #[serde(rename = "type")]
  pub type_field: String,
  pub pid: String,
  pub amount: String,
  pub inscription_number: i64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawEvent {
  #[serde(rename = "type")]
  pub type_field: String,
  pub pid: String,
  pub amount: String,
  pub inscription_number: i64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassiveWithdraw {
  pub pid: String,
  pub amount: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassiveWithdrawEvent {
  #[serde(rename = "type")]
  pub type_field: String,
  pub passive_withdraw: Vec<PassiveWithdraw>,
  pub inscription_number: i64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brc30MintEvent {
  #[serde(rename = "type")]
  pub type_field: String,
  pub pid: String,
  pub amount: String,
  pub inscription_number: i64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brc30InscribeTransferEvent {
  #[serde(rename = "type")]
  pub type_field: String,
  pub tick: Tick,
  pub amount: String,
  pub inscription_number: i64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brc30TransferEvent {
  #[serde(rename = "type")]
  pub type_field: String,
  pub tick: Tick,
  pub amount: String,
  pub inscription_number: i64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brc30ErrorEvent {
  #[serde(rename = "type")]
  pub type_field: String,
  pub pid: String,
  pub inscription_number: i64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: ScriptPubkey,
  pub to: ScriptPubkey,
  pub valid: bool,
  pub msg: String,
}

impl From<&brc30::BRC30Receipt> for Brc30Event {
  fn from(receipt: &brc30::BRC30Receipt) -> Self {
    match { receipt.result.clone() } {
      Ok(a) => match a {
        BRC30Event::DeployTick(d) => Self::DeployTick(DeployTickEvent {
          type_field: String::from("deployTick"),
          tick: Tick {
            id: d.tick_id.to_lowercase().hex(),
            name: d.name.as_str().to_string(),
          },
          supply: d.supply.to_string(),
          decimal: d.decimal,
          inscription_number: receipt.inscription_number,
          inscription_id: receipt.inscription_id.to_string(),
          old_satpoint: receipt.old_satpoint.to_string(),
          new_satpoint: receipt.new_satpoint.to_string(),
          from: receipt.from.clone().into(),
          to: receipt.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
        }),
        BRC30Event::DeployPool(d) => Self::DeployPool(DeployPoolEvent {
          type_field: String::from("deployPool"),
          pid: d.pid.as_str().to_string(),
          stake: Stake {
            type_field: d.stake.to_type(),
            tick: d.stake.to_string(),
          },
          earn: Earn {
            id: "".to_string(),
            name: "".to_string(),
          },
          pool: d.ptype.to_string(),
          erate: d.erate.to_string(),
          only: 0,
          dmax: d.dmax.to_string(),
          inscription_number: receipt.inscription_number,
          inscription_id: receipt.inscription_id.to_string(),
          old_satpoint: receipt.old_satpoint.to_string(),
          new_satpoint: receipt.new_satpoint.to_string(),
          from: receipt.from.clone().into(),
          to: receipt.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
        }),

        BRC30Event::Deposit(d) => Self::Deposit(DepositEvent {
          type_field: String::from("deposit"),
          pid: d.pid.as_str().to_string(),
          amount: d.amt.to_string(),
          inscription_number: receipt.inscription_number,
          inscription_id: receipt.inscription_id.to_string(),
          old_satpoint: receipt.old_satpoint.to_string(),
          new_satpoint: receipt.new_satpoint.to_string(),
          from: receipt.from.clone().into(),
          to: receipt.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
        }),

        BRC30Event::Withdraw(d) => Self::Withdraw(WithdrawEvent {
          type_field: String::from("withdraw"),
          pid: d.pid.as_str().to_string(),
          amount: d.amt.to_string(),
          inscription_number: receipt.inscription_number,
          inscription_id: receipt.inscription_id.to_string(),
          old_satpoint: receipt.old_satpoint.to_string(),
          new_satpoint: receipt.new_satpoint.to_string(),
          from: receipt.from.clone().into(),
          to: receipt.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
        }),

        BRC30Event::PassiveWithdraw(d) => Self::PassiveWithdraw(PassiveWithdrawEvent {
          type_field: String::from("passiveWithdraw"),
          passive_withdraw: d
            .pid
            .iter()
            .map(|(x, y)| PassiveWithdraw {
              pid: x.as_str().to_string(),
              amount: y.to_string(),
            })
            .collect(),
          inscription_number: receipt.inscription_number,
          inscription_id: receipt.inscription_id.to_string(),
          old_satpoint: receipt.old_satpoint.to_string(),
          new_satpoint: receipt.new_satpoint.to_string(),
          from: receipt.from.clone().into(),
          to: receipt.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
        }),

        BRC30Event::Mint(d) => Self::Mint(Brc30MintEvent {
          type_field: String::from("mint"),
          pid: d.tick_id.to_lowercase().hex().to_string(),
          amount: d.amt.to_string(),
          inscription_number: receipt.inscription_number,
          inscription_id: receipt.inscription_id.to_string(),
          old_satpoint: receipt.old_satpoint.to_string(),
          new_satpoint: receipt.new_satpoint.to_string(),
          from: receipt.from.clone().into(),
          to: receipt.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
        }),

        BRC30Event::InscribeTransfer(d) => Self::InscribeTransfer(Brc30InscribeTransferEvent {
          type_field: String::from("inscribeTransfer"),
          tick: Tick {
            id: d.tick_id.to_lowercase().hex(),
            name: d.tick_id.to_lowercase().hex(),
          },
          amount: d.amt.to_string(),
          inscription_number: receipt.inscription_number,
          inscription_id: receipt.inscription_id.to_string(),
          old_satpoint: receipt.old_satpoint.to_string(),
          new_satpoint: receipt.new_satpoint.to_string(),
          from: receipt.from.clone().into(),
          to: receipt.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
        }),

        BRC30Event::Transfer(d) => Self::Transfer(Brc30TransferEvent {
          type_field: String::from("transfer"),
          tick: Tick {
            id: d.tick_id.to_lowercase().hex(),
            name: d.tick_id.to_lowercase().hex(),
          },
          amount: d.amt.to_string(),
          inscription_number: receipt.inscription_number,
          inscription_id: receipt.inscription_id.to_string(),
          old_satpoint: receipt.old_satpoint.to_string(),
          new_satpoint: receipt.new_satpoint.to_string(),
          from: receipt.from.clone().into(),
          to: receipt.to.clone().into(),
          valid: true,
          msg: "ok".to_string(),
        }),
      },
      Err(e) => Self::Error(Brc30ErrorEvent {
        type_field: receipt.op.to_string(),
        pid: "".to_string(),
        inscription_number: receipt.inscription_number,
        inscription_id: receipt.inscription_id.to_string(),
        old_satpoint: receipt.old_satpoint.to_string(),
        new_satpoint: receipt.new_satpoint.to_string(),
        from: receipt.from.clone().into(),
        to: receipt.to.clone().into(),
        valid: false,
        msg: e.to_string(),
      }),
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BRC30BlockEvents {
  pub block: Vec<Events>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserReward {
  #[serde(rename = "user_reward")]
  pub user_reward: String,
  #[serde(rename = "block_num")]
  pub block_num: String,
}
