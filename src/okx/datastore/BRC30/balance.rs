use super::TickId;
use crate::okx::datastore::ScriptKey;
use crate::okx::datastore::BRC20::{ActionReceipt, Tick, TokenInfo, TransferableLog};
use crate::InscriptionId;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct Balance {
  pub tick_id: TickId,
  pub overall_balance: u128,
  pub transferable_balance: u128,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TransferableAsset {
  pub inscription_id: InscriptionId,
  pub amount: u128,
  pub tick_id: TickId,
  pub owner: ScriptKey,
}
