use super::*;
use crate::InscriptionId;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct TransferableLog {
  pub inscription_id: InscriptionId,
  pub amount: u128,
  pub tick: Tick,
  pub owner: ScriptKey,
}
