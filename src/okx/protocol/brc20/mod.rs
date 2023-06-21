use crate::{
  okx::datastore::{brc20::BRC20OperationType, ScriptKey},
  InscriptionId, Result, SatPoint,
};
use bitcoin::Txid;

mod error;
mod msg_executor;
mod msg_resolver;
mod num;
mod operation;
mod params;

use self::error::Error;
pub(crate) use self::{
  error::JSONError,
  msg_executor::execute,
  msg_resolver::resolve_message,
  num::Num,
  operation::{
    deserialize_brc20_operation, BRC20Operation, Deploy as BRC20Deploy, Mint as BRC20Mint,
    Transfer as BRC20Transfer,
  },
};

pub struct BRC20Message {
  pub txid: Txid,
  pub block_height: u64,
  pub block_time: u32,
  pub inscription_id: InscriptionId,
  pub inscription_number: i64,
  pub from: ScriptKey,
  pub to: ScriptKey,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub op: BRC20Operation,
}
