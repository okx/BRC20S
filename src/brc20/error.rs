use crate::brc20::num::Num;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
  #[error("invalid brc20 number: {0}")]
  InvalidNum(String),

  #[error("{op} overflow: original: {org}, other: {other}")]
  Overflow {
    op: &'static str,
    org: Num,
    other: Num,
  },

  #[error("invalid json string")]
  InvalidJson,

  #[error("not brc20 json")]
  NotBRC20Json,

  #[error("parse operation json error: {0}")]
  ParseOperationJsonError(String),

  #[error("invalid decimals {0}")]
  InvalidDecimals(u32),

  #[error("invalid max supply: {0}")]
  InvalidMaxSupply(Num),
}
