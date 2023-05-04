use super::TickType;
use crate::brc20::custom_serde::TickSerde;
use crate::brc20::num::Num;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Mint {
  #[serde(rename = "tick", with = "TickSerde")]
  pub tick: TickType,
  #[serde(rename = "amt")]
  pub amount: Num,
}
