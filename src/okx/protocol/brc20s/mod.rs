use crate::inscription_id::InscriptionId;
use crate::okx::protocol::brc20s::operation::Operation;
use crate::SatPoint;
use bitcoin::Txid;

pub mod error;
pub mod hash;
pub mod msg_executor;
pub mod msg_resolver;
pub mod num;
pub mod operation;
pub mod params;
mod util;
#[cfg(test)]
#[macro_use]
mod test;
mod vesion;

pub use self::{
  error::{BRC20SError, Error},
  msg_executor::{execute, BRC20SExecutionMessage},
  num::Num,
  operation::{Deploy, Mint, PassiveUnStake, RawOperation, Stake, Transfer, UnStake},
};
pub(crate) use self::{msg_resolver::resolve_message, operation::deserialize_brc20s_operation};

#[derive(Debug, Clone)]
pub struct Message {
  pub txid: Txid,
  pub inscription_id: InscriptionId,
  pub commit_input_satpoint: Option<SatPoint>,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub op: Operation,
}
