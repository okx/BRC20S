#[derive(Debug, PartialEq, thiserror::Error)]
pub enum JSONError {
  #[error("invalid json string")]
  InvalidJson,

  #[error("not brc0 json")]
  NotBRC0Json,

  #[error("parse operation json error: {0}")]
  ParseOperationJsonError(String),
}
