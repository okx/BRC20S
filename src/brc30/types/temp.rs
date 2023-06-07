use crate::Inscription;
use crate::InscriptionId;
use crate::SatPoint;
use bitcoin::Txid;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

// TODO - this is a temporary solution to the problem of
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct InscriptionOperation {
  txid: Txid,
}
