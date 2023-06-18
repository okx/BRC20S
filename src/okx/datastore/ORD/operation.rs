use crate::{InscriptionId, SatPoint};
use bitcoin::Txid;

// collect the inscription operation.
pub struct InscriptionOperation {
  pub txid: Txid,
  pub action: Action,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
}

// the act of marking an inscription.
pub enum Action {
  New,
  Transfer,
}
