use super::*;
use crate::okx::datastore::brc30;
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
  pub deployer: String,
  pub txid: String,
  pub deploy_height: u64,
  pub deploy_blocktime: u64,
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
      inscription_number: 0, // TODO inscription_number
      minted: tick_info.minted.to_string(),
      supply: tick_info.supply.to_string(),
      decimal: tick_info.decimal as u64,
      deployer: tick_info.deployer.to_string(),
      txid: tick_info.inscription_id.txid.to_string(),
      deploy_height: tick_info.deploy_block,
      deploy_blocktime: 0, // TODO  add deploy_blocktime
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
  pub deployer: String,
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

  pub fn set_deploy(&mut self, deployer: String, deploy_height: u64, deploy_blocktime: u64) {
    self.deployer = deployer;
    self.deploy_height = deploy_height;
    self.deploy_blocktime = deploy_blocktime;
  }
}

impl From<&brc30::PoolInfo> for BRC30Pool {
  fn from(pool_info: &brc30::PoolInfo) -> Self {
    let stake = Stake {
      type_field: pool_info.ptype.to_string(),
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
      only: if pool_info.only { 0 } else { 1 },
      acc_per_share: pool_info.acc_reward_per_share.to_string(),
      latest_update_block: pool_info.last_update_block,
      inscription_id: pool_info.inscription_id.to_string(),
      inscription_number: 0,
      deployer: "".to_string(),
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
  pub reward_debt: String,
  pub latest_update_block: u64,
}

impl From<&brc30::UserInfo> for UserInfo {
  fn from(user_info: &brc30::UserInfo) -> Self {
    Self {
      pid: user_info.pid.as_str().to_string(),
      staked: user_info.staked.to_string(),
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
  pub claimable: String,
}

impl BRC30Balance {
  pub fn set_claimable(&mut self, claimable: u128) {
    self.claimable = claimable.to_string();
  }

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
      claimable: "0".to_string(),
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
  pub inscriptions: Vec<Inscription>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inscription {
  pub tick: Tick,
  pub inscription_id: String,
  pub inscription_number: u64,
  pub amount: String,
  pub owner: String,
}

impl From<&brc30::TransferableAsset> for Inscription {
  fn from(asset: &brc30::TransferableAsset) -> Self {
    let tick = Tick {
      id: asset.tick_id.to_lowercase().hex(),
      name: "".to_string(), //TODO
    };

    Self {
      tick,
      inscription_id: asset.inscription_id.to_string(),
      inscription_number: 0,   //TODO
      amount: "0".to_string(), //TODO
      owner: asset.owner.to_string(),
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Events {
  pub events: Vec<Event>,
  pub txid: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
  #[serde(rename = "type")]
  pub type_field: String,
  pub tick: Tick,
  pub supply: String,
  pub decimal: u64,
  pub inscription_number: u64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: String,
  pub to: String,
  pub valid: bool,
  pub msg: String,
  pub pid: String,
  pub stake: Stake,
  pub earn: Earn,
  pub pool: String,
  pub erate: String,
  pub only: u64,
  pub dmax: String,
  pub amount: String,
}

impl From<&brc30::BRC30Receipt> for Event {
  fn from(_receipt: &brc30::BRC30Receipt) -> Self {
    // TODO
    let earn = Earn {
      id: "".to_string(),
      name: "".to_string(),
    };

    let stake = Stake {
      type_field: "".to_string(),
      tick: "".to_string(),
    };

    let tick = Tick {
      id: "".to_string(),
      name: "".to_string(),
    };

    Self {
      type_field: "".to_string(),
      tick,
      supply: "".to_string(),
      decimal: 0,
      inscription_number: 0,
      inscription_id: "".to_string(),
      old_satpoint: "".to_string(),
      new_satpoint: "".to_string(),
      from: "".to_string(),
      to: "".to_string(),
      valid: false,
      msg: "".to_string(),
      pid: "".to_string(),
      stake,
      earn,
      pool: "".to_string(),
      erate: "".to_string(),
      only: 0,
      dmax: "".to_string(),
      amount: "".to_string(),
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BRC30BlockEvents {
  pub block: Vec<Events>,
}
