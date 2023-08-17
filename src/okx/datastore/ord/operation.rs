use crate::{InscriptionId, SatPoint};
use bitcoin::Txid;
use serde::{Deserialize, Serialize};

// collect the inscription operation.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InscriptionOp {
  pub txid: Txid,
  pub action: Action,
  pub inscription_number: Option<i64>,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
}

// the act of marking an inscription.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Action {
  New { cursed: bool, unbound: bool },
  Transfer,
}
