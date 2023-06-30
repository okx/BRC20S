pub(crate) mod brc20;
pub(crate) mod brc30;
pub(crate) mod execute_manager;
pub(crate) mod message;
pub(crate) mod protocol_manager;
pub(crate) mod resolve_manager;

#[cfg(test)]
#[macro_use]
mod test;

pub use self::protocol_manager::{BlockContext, ProtocolManager};
use self::{
  execute_manager::CallManager,
  message::{Message, Receipt},
  protocol_manager::ProtocolKind,
  resolve_manager::MsgResolveManager,
};
