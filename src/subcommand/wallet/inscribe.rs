use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub commit: Txid,
  pub inscription: InscriptionId,
  pub parent: Option<InscriptionId>,
  pub reveal: Txid,
  pub total_fees: u64,
}
