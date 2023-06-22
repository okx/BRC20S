use crate::{InscriptionId, SatPoint};
use bitcoin::Txid;

// collect the inscription operation.
#[derive(Clone)]
pub struct InscriptionOp {
  pub txid: Txid,
  pub action: Action,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
}

// the act of marking an inscription.
#[derive(Clone)]
pub enum Action {
  New,
  Transfer,
}
