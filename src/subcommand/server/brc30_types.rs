use super::error::ApiError;
use super::*;
use crate::okx::datastore::{ScriptKey, BRC30};
use axum::Json;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BRC30TickInfo {
  pub tick: Tick,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub minted: String,
  pub supply: String,
  pub decimal: i64,
  pub deployer: Deployer,
  pub txid: String,
  pub deploy_height: i64,
  pub deploy_blocktime: i64,
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
  pub only: i64,
  pub acc_per_share: i64,
  pub latest_update_block: i64,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub deployer: Deployer,
  pub deploy_height: i64,
  pub deploy_blocktime: i64,
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deployer {
  pub address: String,
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
  pub latest_update_block: i64,
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
  pub inscription_number: i64,
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
  pub decimal: Option<i64>,
  pub inscription_number: i64,
  pub inscription_id: String,
  pub old_satpoint: String,
  pub new_satpoint: String,
  pub from: From,
  pub to: To,
  pub valid: bool,
  pub msg: String,
  pub pid: String,
  pub stake: Stake,
  pub earn: Earn,
  pub pool: String,
  pub erate: String,
  pub only: i64,
  pub dmax: String,
  pub amount: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct From {
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
