use bitcoin::Txid;
use serde::{Deserialize, Serialize, Serializer};

// TODO - this is a temporary solution to the problem of
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct InscriptionOperation {
  pub txid: Txid,
}
