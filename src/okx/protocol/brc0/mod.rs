mod error;
mod rpc;

pub(crate) use self::{
  error::JSONError,
  rpc::{BRCZeroTx, RpcParams, RpcRequest},
};
