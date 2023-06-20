use super::*;
use crate::okx::datastore::BRC30;
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

impl From<&BRC30::TickInfo> for BRC30TickInfo {
  fn from(tick_info: &BRC30::TickInfo) -> Self {
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
  pub acc_per_share: u64,
  pub latest_update_block: u64,
  pub inscription_id: String,
  pub inscription_number: u64,
  pub deployer: String,
  pub deploy_height: u64,
  pub deploy_blocktime: u64,
  pub txid: String,
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BRC30Balance {
  pub tick: Tick,
  pub transferable: String,
  pub overall: String,
  pub claimable: String,
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
  pub tick: Option<Tick>,
  pub supply: Option<String>,
  pub decimal: Option<u64>,
  pub inscription_number: u64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: AddFrom,
  pub to: To,
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddFrom {
  pub address: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct To {
  pub address: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockEvents {
  pub txid: String,
  pub events: Vec<Event>,
}
