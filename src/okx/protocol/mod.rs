pub(crate) mod brc0;
pub(crate) mod brc20;
pub(crate) mod brc20s;
pub(crate) mod execute_manager;
pub(crate) mod message;
pub(crate) mod protocol_manager;
pub(crate) mod resolve_manager;
mod utils;

pub use self::protocol_manager::{BlockContext, ProtocolManager};
use self::{
  execute_manager::CallManager,
  message::{Message, Receipt, BrcZeroMsg, MsgInscription, InscriptionContext},
  protocol_manager::ProtocolKind,
  resolve_manager::MsgResolveManager,
};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum JSONError {
  #[error("invalid content type")]
  InvalidContentType,

  #[error("unsupported content type")]
  UnSupportContentType,

  #[error("invalid json string")]
  InvalidJson,

  #[error("not brc0 json")]
  NotBRC0Json,
}