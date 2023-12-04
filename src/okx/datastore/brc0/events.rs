use super::*;
use crate::{InscriptionId, SatPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum OperationType {
    Evm,
}
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Receipt {
    pub inscription_id: InscriptionId,
    pub inscription_number: i64,
    pub old_satpoint: SatPoint,
    pub new_satpoint: SatPoint,
    pub op: OperationType,
    pub from: ScriptKey,
    pub to: ScriptKey,
    pub result: Result<Event, BRC0Error>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Event {
    Evm(EvmEvent),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EvmEvent {
    pub txhash: String,
}
