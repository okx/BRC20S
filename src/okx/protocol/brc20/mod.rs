use crate::{
  okx::datastore::{
    brc20::{BRC20Error, OperationType},
    ScriptKey,
  },
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
  msg_executor::{execute, BRC20ExecutionMessage},
  msg_resolver::resolve_message,
  num::Num,
  operation::{
    deserialize_brc20_operation, Deploy as BRC20Deploy, Mint as BRC20Mint, Operation,
    Transfer as BRC20Transfer,
  },
};

#[derive(Debug, Clone)]
pub struct Message {
  pub txid: Txid,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
  pub op: Operation,
}
