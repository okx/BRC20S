use super::{super::error::*, *};
use crate::{InscriptionId, SatPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ActionReceipt {
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub result: Result<BRC20Event, BRC20Error>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum BRC20Event {
  Deploy(DeployEvent),
  Mint(MintEvent),
  TransferPhase1(TransferPhase1Event),
  TransferPhase2(TransferPhase2Event),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DeployEvent {
  pub supply: u128,
  pub limit_per_mint: u128,
  pub decimal: u8,
  pub tick: Tick,
  pub deploy: ScriptKey,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct MintEvent {
  pub tick: Tick,
  pub to: ScriptKey,
  pub amount: u128,
  pub msg: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TransferPhase1Event {
  pub tick: Tick,
  pub owner: ScriptKey,
  pub amount: u128,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TransferPhase2Event {
  pub tick: Tick,
  pub from: ScriptKey,
  pub to: ScriptKey,
  pub amount: u128,
  pub msg: Option<String>,
}
