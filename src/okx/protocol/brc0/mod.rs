use crate::{
  okx::datastore::{brc0::OperationType, ScriptKey},
  InscriptionId, Result, SatPoint,
};
use bitcoin::Txid;

mod error;
mod msg_executor;
mod msg_resolver;
mod operation;
mod params;
mod rpc;

pub(crate) use self::{
    error::JSONError,
    operation::{deserialize_brc0_operation, Operation},
    rpc::{RpcRequest, ZeroData, BRCZeroTx, ZeroTestTx, ZeroTestData, RpcResponse},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
  pub txid: Txid,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  // `new_satpoint` may be none when the transaction is not yet confirmed and the sat has not been bound to the current outputs.
  pub new_satpoint: Option<SatPoint>,
  pub op: Operation,
  pub sat_in_outputs: bool,
  pub btc_fee: u128,
}
