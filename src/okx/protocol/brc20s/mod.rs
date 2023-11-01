use crate::inscription_id::InscriptionId;
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
mod version;

pub(crate) use self::operation::deserialize_brc20s_operation;
pub use self::{
  error::{BRC20SError, Error},
  msg_executor::{execute, ExecutionMessage},
  num::Num,
  operation::{Deploy, Mint, Operation, PassiveUnStake, RawOperation, Stake, Transfer, UnStake},
  version::get_config_by_network,
};
#[derive(Debug, Clone)]
pub struct Message {
  pub txid: Txid,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  // `new_satpoint` may be none when the transaction is not yet confirmed and the sat has not been bound to the current outputs.
  pub new_satpoint: Option<SatPoint>,
  // The validity of access control for `deploy`, `deposit`, `withdraw`, and `mint` operations requires this value.
  pub commit_input_satpoint: Option<SatPoint>,
  pub op: Operation,
  pub sat_in_outputs: bool,
}
