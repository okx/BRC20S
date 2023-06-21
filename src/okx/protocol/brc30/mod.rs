use crate::inscription_id::InscriptionId;
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::brc30::operation::BRC30Operation;
use crate::SatPoint;
use bitcoin::Txid;

pub mod error;
mod hash;
pub mod msg_executor;
pub mod msg_resolver;
pub mod num;
pub mod operation;
pub mod params;
mod util;

pub use self::{
  error::{BRC30Error, Error},
  msg_executor::execute,
  msg_resolver::resolve_message,
  num::Num,
  operation::{
    deserialize_brc30_operation, Deploy, Mint, Operation, PassiveUnStake, Stake, Transfer, UnStake,
  },
};

pub struct BRC30Message {
  pub txid: Txid,
  pub block_height: u64,
  pub block_time: u32,
  pub inscription_id: InscriptionId,
  pub inscription_number: i64,
  pub commit_from: Option<ScriptKey>,
  pub from: ScriptKey,
  pub to: ScriptKey,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub op: BRC30Operation,
}
