use crate::brc20::{ActionReceipt, ScriptKey, Tick, TokenInfo, TransferableLog};
use crate::brc30;
use crate::InscriptionId;
use brc30::TickId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Balance {
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
