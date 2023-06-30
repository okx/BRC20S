use crate::inscription_id::InscriptionId;
use crate::okx::protocol::brc30::operation::BRC30Operation;
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

pub use self::{
  error::{BRC30Error, Error},
  msg_executor::{execute, BRC30ExecutionMessage},
  msg_resolver::resolve_message,
  num::Num,
  operation::{
    deserialize_brc30_operation, Deploy, Mint, Operation, PassiveUnStake, Stake, Transfer, UnStake,
  },
};

pub struct BRC30Message {
  pub txid: Txid,
  pub inscription_id: InscriptionId,
  pub commit_input_satpoint: Option<SatPoint>,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub op: BRC30Operation,
}
