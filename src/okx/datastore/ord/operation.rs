use crate::{InscriptionId, SatPoint};
use bitcoin::Txid;

// collect the inscription operation.
#[derive(Debug, Clone)]
pub struct InscriptionOp {
  pub txid: Txid,
  pub action: Action,
  pub inscription_number: Option<i64>,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
}

// the act of marking an inscription.
#[derive(Debug, Clone)]
pub enum Action {
  New { cursed: bool, unbound: bool },
  Transfer,
}
