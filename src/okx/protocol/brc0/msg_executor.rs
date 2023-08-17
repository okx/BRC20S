use super::*;
use crate::okx::datastore::ord as ord_store;

use crate::{
  okx::{
    datastore::brc0::{Event, EvmEvent, Receipt},
    protocol::{
      brc0::{Message, Operation},
      utils, BlockContext,
    },
  },
  Result,
};
use anyhow::anyhow;
use bitcoin::Network;

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionMessage {
  pub(self) txid: Txid,
  pub(self) inscription_id: InscriptionId,
  pub(self) inscription_number: i64,
  pub(self) old_satpoint: SatPoint,
  pub(self) new_satpoint: SatPoint,
  pub(self) from: ScriptKey,
  pub(self) to: Option<ScriptKey>,
  pub(self) op: Operation,
}

impl ExecutionMessage {
  pub fn from_message<O: ord_store::OrdDataStoreReadOnly>(
    ord_store: &O,
    msg: &Message,
    network: Network,
  ) -> Result<Self> {
    Ok(Self {
      txid: msg.txid,
      inscription_id: msg.inscription_id,
      inscription_number: utils::get_inscription_number_by_id(msg.inscription_id, ord_store)?,
      old_satpoint: msg.old_satpoint,
      new_satpoint: msg
        .new_satpoint
        .ok_or(anyhow!("new satpoint cannot be None"))?,
      from: utils::get_script_key_on_satpoint(msg.old_satpoint, ord_store, network)?,
      to: if msg.sat_in_outputs {
        Some(utils::get_script_key_on_satpoint(
          msg.new_satpoint.unwrap(),
          ord_store,
          network,
        )?)
      } else {
        None
      },
      op: msg.op.clone(),
    })
  }
}

pub fn execute(context: BlockContext, msg: &ExecutionMessage) -> Result<Option<Receipt>> {
  log::debug!("BRC0 execute message: {:?}", msg);
  let _event = match &msg.op {
    Operation::Evm(evm) => process_deploy(context, msg, evm.clone()),
  };

  Ok(None)
}

fn process_deploy(
  _context: BlockContext,
  _msg: &ExecutionMessage,
  evm: Evm,
) -> Result<Event, Error> {
  // TODO send okbc proposal tx
  println!("-----------{}", evm.d);

  Ok(Event::Evm(EvmEvent {
    txhash: "".to_string(),
  }))
}
