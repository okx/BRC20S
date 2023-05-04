use crate::brc20::num::Num;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
  #[error("invalid brc20 number: {0}")]
  InvalidNum(String),

  #[error("{op} overflow: original: {org}, other: {other}")]
  Overflow{op: &'static str, org: Num, other: Num},
}
