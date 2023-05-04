use serde::{Deserialize, Serialize};
use crate::brc20::num::Num;
use crate::brc20::custom_serde::TickSerde;
use super::TickType;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Mint {
  #[serde(rename="tick", with="TickSerde")]
  pub tick: TickType,
  #[serde(rename="amt")]
  pub amount: Num,
}
