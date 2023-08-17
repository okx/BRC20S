use crate::okx::datastore::brc0::BRC0Error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("brc0 error: {0}")]
  BRC0Error(BRC0Error),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum JSONError {
  #[error("invalid content type")]
  InvalidContentType,

  #[error("unsupport content type")]
  UnSupportContentType,

  #[error("invalid json string")]
  InvalidJson,

  #[error("not brc0 json")]
  NotBRC0Json,

  #[error("parse operation json error: {0}")]
  ParseOperationJsonError(String),
}
