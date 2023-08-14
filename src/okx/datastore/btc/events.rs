use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Receipt {
  pub from: ScriptKey,
  pub result: Result<Event, BTCError>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Event {
  Transfer(TransferEvent),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransferEvent {
  pub amt: u128,
  pub msg: Option<String>,
}
